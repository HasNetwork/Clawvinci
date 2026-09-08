// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only
// Derived from Sources/PalmierPro/Audio/Analysis/VoiceActivity.swift and SpeechMaskStore.swift (GPLv3).

use crate::silence::{SilenceRemovalPlanner, SilenceRemovalSettings};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub const VAD_SAMPLE_RATE: usize = 16_000;
pub const VAD_CHUNK_SIZE: usize = 512;
pub const VAD_CHUNK_DURATION: f64 = (VAD_CHUNK_SIZE as f64) / (VAD_SAMPLE_RATE as f64); // 0.032 s = 32 ms

/// A time span indicating detected speech activity.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VadSpan {
    pub start: f64,
    pub end: f64,
}

impl VadSpan {
    pub fn new(start: f64, end: f64) -> Self {
        Self { start, end }
    }

    pub fn duration(&self) -> f64 {
        (self.end - self.start).max(0.0)
    }
}

/// Results of voice activity analysis over an audio track.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VadAnalysis {
    /// Number of 32 ms VAD cells spanning the full source duration.
    pub chunk_count: usize,
    /// Speech spans in source seconds.
    pub segments: Vec<VadSpan>,
}

impl VadAnalysis {
    pub fn new(chunk_count: usize, segments: Vec<VadSpan>) -> Self {
        Self {
            chunk_count,
            segments,
        }
    }

    /// Converts speech segments into a uniform per-cell boolean mask.
    pub fn to_mask(&self) -> Vec<bool> {
        let mut mask = vec![false; self.chunk_count];
        for span in &self.segments {
            let lo = ((span.start / VAD_CHUNK_DURATION).floor() as usize).min(self.chunk_count);
            let hi = ((span.end / VAD_CHUNK_DURATION).ceil() as usize).min(self.chunk_count);
            if lo < hi {
                for cell in lo..hi {
                    mask[cell] = true;
                }
            }
        }
        mask
    }
}

/// Standalone energy-spectral Voice Activity Detector (pure DSP).
pub struct VoiceActivityDetector;

impl VoiceActivityDetector {
    /// Analyzes 16 kHz mono float audio samples for speech activity.
    pub fn analyze(samples: &[f32]) -> VadAnalysis {
        if samples.is_empty() {
            return VadAnalysis::new(0, Vec::new());
        }

        let chunk_count = (samples.len() + VAD_CHUNK_SIZE - 1) / VAD_CHUNK_SIZE;
        let mut chunk_energies = Vec::with_capacity(chunk_count);
        let mut chunk_zcr = Vec::with_capacity(chunk_count);

        // Compute short-time energy and zero-crossing rate per 32 ms chunk
        for c in 0..chunk_count {
            let start = c * VAD_CHUNK_SIZE;
            let end = (start + VAD_CHUNK_SIZE).min(samples.len());
            let chunk = &samples[start..end];

            let mut sum_sq = 0.0f32;
            let mut zc = 0usize;

            for (i, &s) in chunk.iter().enumerate() {
                sum_sq += s * s;
                if i > 0 && ((s >= 0.0 && chunk[i - 1] < 0.0) || (s < 0.0 && chunk[i - 1] >= 0.0)) {
                    zc += 1;
                }
            }

            let rms = (sum_sq / chunk.len().max(1) as f32).sqrt();
            let zcr = zc as f32 / chunk.len().max(1) as f32;

            chunk_energies.push(rms);
            chunk_zcr.push(zcr);
        }

        // Estimate background noise floor using the 15th percentile of energies
        let mut sorted_energies = chunk_energies.clone();
        sorted_energies.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let noise_floor_idx = (sorted_energies.len() * 15 / 100).min(sorted_energies.len() - 1);
        let noise_floor = sorted_energies[noise_floor_idx].max(1e-4);

        // Adaptive speech threshold: 12 dB above noise floor or absolute floor 0.015
        let speech_threshold = (noise_floor * 3.5).max(0.012);

        // State machine classification with hangover smoothing
        let mut is_speech_raw = vec![false; chunk_count];
        for (i, &energy) in chunk_energies.iter().enumerate() {
            let zcr = chunk_zcr[i];
            // Speech typically has energy above threshold and moderate ZCR (not pure high freq hiss)
            if energy >= speech_threshold && zcr < 0.55 {
                is_speech_raw[i] = true;
            }
        }

        // Apply hangover smoothing: 2-frame onset confirmation, 5-frame tail hangover (160ms)
        let mut smoothed = vec![false; chunk_count];
        let hangover_frames = 5;
        let mut hangover = 0;

        for i in 0..chunk_count {
            if is_speech_raw[i] {
                hangover = hangover_frames;
                smoothed[i] = true;
            } else if hangover > 0 {
                hangover -= 1;
                smoothed[i] = true;
            }
        }

        // Extract continuous speech spans (discarding bursts < 100ms)
        let min_span_chunks = 3; // ~96 ms
        let mut segments = Vec::new();
        let mut i = 0;

        while i < chunk_count {
            if !smoothed[i] {
                i += 1;
                continue;
            }

            let mut j = i + 1;
            while j < chunk_count && smoothed[j] {
                j += 1;
            }

            if j - i >= min_span_chunks {
                segments.push(VadSpan::new(
                    i as f64 * VAD_CHUNK_DURATION,
                    j as f64 * VAD_CHUNK_DURATION,
                ));
            }

            i = j;
        }

        VadAnalysis::new(chunk_count, segments)
    }
}

/// Store for speech masks and derived quiet non-speech / dead-air spans.
#[derive(Debug, Default)]
pub struct SpeechMaskStore {
    speech_masks: HashMap<String, Vec<bool>>,
    quiet_non_speech_masks: HashMap<String, Vec<bool>>,
    dead_air_masks: HashMap<String, (SilenceRemovalSettings, Vec<bool>)>,
}

impl SpeechMaskStore {
    pub const SPEECH_GAP: f32 = 0.24; // ~12 dB gap
    pub const NO_SPEECH_FLOOR: f32 = 0.56; // Fallback threshold

    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert_speech_mask(&mut self, media_ref: impl Into<String>, mask: Vec<bool>) {
        let key = media_ref.into();
        self.speech_masks.insert(key.clone(), mask);
        self.quiet_non_speech_masks.remove(&key);
        self.dead_air_masks.remove(&key);
    }

    pub fn speech_mask(&self, media_ref: &str) -> Option<&Vec<bool>> {
        self.speech_masks.get(media_ref)
    }

    pub fn quiet_non_speech_mask(
        &mut self,
        media_ref: &str,
        samples: &[f32],
    ) -> Option<Vec<bool>> {
        if let Some(cached) = self.quiet_non_speech_masks.get(media_ref) {
            return Some(cached.clone());
        }

        let speech = self.speech_masks.get(media_ref)?;
        if speech.is_empty() || samples.is_empty() {
            return None;
        }

        let mask = Self::build_quiet_non_speech_mask(speech, samples);
        self.quiet_non_speech_masks.insert(media_ref.to_string(), mask.clone());
        Some(mask)
    }

    pub fn dead_air_mask(
        &mut self,
        media_ref: &str,
        samples: &[f32],
        settings: SilenceRemovalSettings,
    ) -> Option<Vec<bool>> {
        if let Some((cached_settings, mask)) = self.dead_air_masks.get(media_ref) {
            if *cached_settings == settings {
                return Some(mask.clone());
            }
        }

        let quiet_non_speech = self.quiet_non_speech_mask(media_ref, samples)?;
        let mask = SilenceRemovalPlanner::removable_mask(
            &quiet_non_speech,
            &settings,
            VAD_CHUNK_DURATION,
        );

        self.dead_air_masks
            .insert(media_ref.to_string(), (settings, mask.clone()));
        Some(mask)
    }

    pub fn invalidate(&mut self, media_ref: &str) {
        self.speech_masks.remove(media_ref);
        self.quiet_non_speech_masks.remove(media_ref);
        self.dead_air_masks.remove(media_ref);
    }

    pub fn clear(&mut self) {
        self.speech_masks.clear();
        self.quiet_non_speech_masks.clear();
        self.dead_air_masks.clear();
    }

    /// Derives a quiet non-speech mask from the speech mask and audio envelope/samples.
    pub fn build_quiet_non_speech_mask(speech: &[bool], samples: &[f32]) -> Vec<bool> {
        let n = speech.len();
        if n == 0 || samples.is_empty() {
            return Vec::new();
        }

        let cell_peak = |c: usize| -> f32 {
            let s0 = c * samples.len() / n;
            let s1 = samples.len().min((s0 + 1).max((c + 1) * samples.len() / n));
            let mut peak = 0.0f32;
            for s in &samples[s0..s1] {
                let abs = s.abs();
                if abs > peak {
                    peak = abs;
                }
            }
            peak
        };

        let median = |values: &mut [f32]| -> f32 {
            if values.is_empty() {
                return 0.0;
            }
            values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            values[values.len() / 2]
        };

        let mut speech_peaks: Vec<f32> = (0..n).filter(|&i| speech[i]).map(cell_peak).collect();
        let speech_median = median(&mut speech_peaks);

        // Quiet floor is dynamic based on speech level
        let quiet_floor = if speech_peaks.is_empty() {
            Self::NO_SPEECH_FLOOR * 0.1 // amplitude domain
        } else {
            (speech_median * 0.25).max(0.015) // at least 12 dB below median speech
        };

        let mut dead = vec![false; n];
        let mut i = 0;

        while i < n {
            if speech[i] {
                i += 1;
                continue;
            }

            let mut j = i;
            while j < n && !speech[j] {
                j += 1;
            }

            let mut non_speech_peaks: Vec<f32> = (i..j).map(cell_peak).collect();
            let non_speech_median = median(&mut non_speech_peaks);

            if non_speech_median <= quiet_floor {
                for c in i..j {
                    dead[c] = true;
                }
            }

            i = j;
        }

        dead
    }
}
