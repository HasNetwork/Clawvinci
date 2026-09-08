// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only
// Derived from Sources/PalmierPro/Audio/Beats/BeatDetector.swift and BeatStore.swift (GPLv3).

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

pub const BEAT_SAMPLE_RATE: f64 = 22050.0;
pub const BEAT_HOP: usize = 441; // 50 logit frames / second (20 ms per frame)

/// Results of beat and tempo analysis.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BeatAnalysis {
    /// Estimated tempo in beats per minute (0.0 if indeterminate).
    pub bpm: f64,
    /// Timestamps of detected beats in source seconds.
    pub beats: Vec<f64>,
    /// Timestamps of detected downbeats (bar starts) in source seconds.
    pub downbeats: Vec<f64>,
}

impl BeatAnalysis {
    pub const EMPTY: Self = Self {
        bpm: 0.0,
        beats: Vec::new(),
        downbeats: Vec::new(),
    };

    pub fn new(bpm: f64, beats: Vec<f64>, downbeats: Vec<f64>) -> Self {
        Self {
            bpm,
            beats,
            downbeats,
        }
    }
}

/// Standalone onset and beat tracking engine (pure DSP).
pub struct BeatDetector;

impl BeatDetector {
    /// Analyzes mono audio samples at 22,050 Hz for rhythmic onsets, beats, and tempo.
    pub fn detect(samples: &[f32], sample_rate: f64) -> BeatAnalysis {
        if samples.is_empty() || sample_rate <= 0.0 {
            return BeatAnalysis::EMPTY;
        }

        // Window into hop frames
        let hop_size = ((sample_rate * (BEAT_HOP as f64 / BEAT_SAMPLE_RATE)).round() as usize).max(1);
        let frame_count = samples.len().div_ceil(hop_size);
        if frame_count < 3 {
            return BeatAnalysis::EMPTY;
        }

        // Compute frame energy and high-frequency content for onset novelty
        let mut energies = Vec::with_capacity(frame_count);
        for f in 0..frame_count {
            let start = f * hop_size;
            let end = (start + hop_size).min(samples.len());
            let chunk = &samples[start..end];

            let mut sum_sq = 0.0f32;
            let mut hfc = 0.0f32;
            for (i, &s) in chunk.iter().enumerate() {
                sum_sq += s * s;
                if i > 0 {
                    let diff = s - chunk[i - 1];
                    hfc += diff * diff;
                }
            }

            let rms = (sum_sq / chunk.len().max(1) as f32).sqrt();
            let hf_energy = (hfc / chunk.len().max(1) as f32).sqrt();
            energies.push(rms * 0.5 + hf_energy * 0.5);
        }

        // Half-wave rectified first difference (spectral flux / energy novelty)
        let mut novelty = vec![0.0f32; frame_count];
        for i in 1..frame_count {
            let diff = energies[i] - energies[i - 1];
            if diff > 0.0 {
                novelty[i] = diff;
            }
        }

        // Adaptive thresholding: local moving average + dynamic offset
        let window_radius = 8; // ~160 ms context
        let mut logits = vec![-10.0f32; frame_count];
        let max_novelty = novelty.iter().copied().fold(0.0f32, f32::max);

        if max_novelty > 1e-5 {
            for i in 0..frame_count {
                let w_start = i.saturating_sub(window_radius);
                let w_end = (i + window_radius + 1).min(frame_count);
                let mean: f32 = novelty[w_start..w_end].iter().sum::<f32>() / (w_end - w_start) as f32;

                let val = novelty[i];
                if val > mean * 1.25 && val > max_novelty * 0.15 {
                    let normalized = ((val - mean) / (max_novelty - mean).max(1e-5)).clamp(0.0, 1.0);
                    // Map to logit: log(p / (1 - p))
                    let p = (normalized * 0.9 + 0.05).clamp(0.01, 0.99);
                    logits[i] = (p / (1.0 - p)).ln();
                }
            }
        }

        let beats = pick_peaks(&logits, 0.5);
        let bpm = estimate_bpm(&beats).unwrap_or(0.0);

        // Classify downbeats (bar accents): every 4th beat for 4/4 or peaks with highest energy
        let mut downbeats = Vec::new();
        if !beats.is_empty() {
            for (idx, &t) in beats.iter().enumerate() {
                if idx % 4 == 0 {
                    downbeats.push(t);
                }
            }
        }

        BeatAnalysis::new(bpm, beats, downbeats)
    }
}

/// Picks timestamps where sigmoid(logit) is a local maximum above threshold.
pub fn pick_peaks(logits: &[f32], threshold: f32) -> Vec<f64> {
    if logits.len() < 3 {
        return Vec::new();
    }

    let mut times = Vec::new();
    for i in 1..logits.len() - 1 {
        let p = 1.0 / (1.0 + (-logits[i]).exp());
        if p >= threshold && logits[i] >= logits[i - 1] && logits[i] > logits[i + 1] {
            times.push((i * BEAT_HOP) as f64 / BEAT_SAMPLE_RATE);
        }
    }
    times
}

/// Computes estimated BPM as 60.0 divided by median inter-beat interval.
pub fn estimate_bpm(beats: &[f64]) -> Option<f64> {
    if beats.len() <= 2 {
        return None;
    }

    let mut intervals = Vec::with_capacity(beats.len() - 1);
    for i in 1..beats.len() {
        let diff = beats[i] - beats[i - 1];
        if diff > 0.05 {
            intervals.push(diff);
        }
    }

    if intervals.is_empty() {
        return None;
    }

    intervals.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let median = intervals[intervals.len() / 2];

    if median > 0.0 {
        Some(60.0 / median)
    } else {
        None
    }
}

/// Cache store for beat analyses per media asset.
#[derive(Debug, Clone, Default)]
pub struct BeatStore {
    cache: Arc<RwLock<HashMap<String, BeatAnalysis>>>,
}

impl BeatStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&self, media_ref: &str) -> Option<BeatAnalysis> {
        let read = self.cache.read().ok()?;
        read.get(media_ref).cloned()
    }

    pub fn insert(&self, media_ref: impl Into<String>, analysis: BeatAnalysis) {
        if let Ok(mut write) = self.cache.write() {
            write.insert(media_ref.into(), analysis);
        }
    }

    pub fn invalidate(&self, media_ref: &str) {
        if let Ok(mut write) = self.cache.write() {
            write.remove(media_ref);
        }
    }

    pub fn clear(&self) {
        if let Ok(mut write) = self.cache.write() {
            write.clear();
        }
    }
}
