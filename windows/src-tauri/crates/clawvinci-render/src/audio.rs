// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only
// Derived from Sources/PalmierPro/Preview/ScrubAudioEngine.swift (GPLv3).

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;

/// High-precision monotonic clock to drive playback synchronization.
pub struct AudioClock {
    epoch: Instant,
    accumulated_seconds: Arc<AtomicU64>, // stored as f64::to_bits
    is_running: Arc<AtomicBool>,
}

impl Default for AudioClock {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioClock {
    pub fn new() -> Self {
        Self {
            epoch: Instant::now(),
            accumulated_seconds: Arc::new(AtomicU64::new(0.0f64.to_bits())),
            is_running: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn start(&mut self, start_seconds: f64) {
        self.accumulated_seconds
            .store(start_seconds.max(0.0).to_bits(), Ordering::SeqCst);
        self.epoch = Instant::now();
        self.is_running.store(true, Ordering::SeqCst);
    }

    pub fn pause(&mut self) {
        if self.is_running.swap(false, Ordering::SeqCst) {
            let elapsed = self.epoch.elapsed().as_secs_f64();
            let current = f64::from_bits(self.accumulated_seconds.load(Ordering::SeqCst));
            self.accumulated_seconds
                .store((current + elapsed).to_bits(), Ordering::SeqCst);
        }
    }

    pub fn resume(&mut self) {
        if !self.is_running.swap(true, Ordering::SeqCst) {
            self.epoch = Instant::now();
        }
    }

    pub fn seek(&mut self, time_seconds: f64) {
        self.accumulated_seconds
            .store(time_seconds.max(0.0).to_bits(), Ordering::SeqCst);
        self.epoch = Instant::now();
    }

    pub fn current_time_seconds(&self) -> f64 {
        let base = f64::from_bits(self.accumulated_seconds.load(Ordering::SeqCst));
        if self.is_running.load(Ordering::SeqCst) {
            base + self.epoch.elapsed().as_secs_f64()
        } else {
            base
        }
    }

    pub fn current_frame(&self, fps: i32) -> usize {
        let t = self.current_time_seconds();
        (t * fps.max(1) as f64).round().max(0.0) as usize
    }

    pub fn is_running(&self) -> bool {
        self.is_running.load(Ordering::SeqCst)
    }
}

/// Scrub audio synthesizer generating audio grains during interactive scrubbing and frame steps.
pub struct ScrubAudioEngine;

impl ScrubAudioEngine {
    pub const DEFAULT_SAMPLE_RATE: u32 = 48_000;
    pub const GRAIN_SAMPLES: usize = 2_400; // ~50ms at 48kHz
    pub const FADE_SAMPLES: usize = 144;   // ~3ms fade in/out

    /// Synthesizes a directional grain buffer for a single-frame step.
    pub fn synthesize_step_grain(forward: bool, sample_rate: u32) -> Vec<f32> {
        let count = (Self::GRAIN_SAMPLES as u64 * sample_rate as u64 / Self::DEFAULT_SAMPLE_RATE as u64) as usize;
        let mut buffer = Vec::with_capacity(count);

        let freq_start = if forward { 320.0 } else { 480.0 };
        let freq_end = if forward { 540.0 } else { 280.0 };

        let fade_in = (Self::FADE_SAMPLES as u64 * sample_rate as u64 / Self::DEFAULT_SAMPLE_RATE as u64) as usize;
        let fade_out = fade_in;

        for i in 0..count {
            let t = i as f32 / count as f32;
            let freq = freq_start + (freq_end - freq_start) * t;
            let phase = 2.0 * std::f32::consts::PI * freq * (i as f32 / sample_rate as f32);
            let raw_sample = phase.sin() * 0.25;

            // Envelope windowing
            let envelope = if i < fade_in {
                i as f32 / fade_in as f32
            } else if i >= count - fade_out {
                (count - 1 - i) as f32 / fade_out as f32
            } else {
                1.0
            };

            buffer.push(raw_sample * envelope);
        }

        buffer
    }

    /// Synthesizes an interactive scrub buffer proportional to velocity (delta frames).
    pub fn synthesize_scrub_burst(delta_frames: i64, sample_rate: u32) -> Vec<f32> {
        if delta_frames == 0 {
            return Vec::new();
        }
        let forward = delta_frames > 0;
        let mag = (delta_frames.abs() as f32).clamp(1.0, 10.0);
        let mut grain = Self::synthesize_step_grain(forward, sample_rate);

        // Scale amplitude by scrub velocity
        let gain = (0.2 + mag * 0.08).min(0.8);
        for s in &mut grain {
            *s *= gain;
        }

        grain
    }
}
