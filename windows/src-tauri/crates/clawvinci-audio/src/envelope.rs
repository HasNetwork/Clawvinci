// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only
// Derived from Sources/PalmierPro/Audio/AudioEnvelope.swift (GPLv3).

use serde::{Deserialize, Serialize};

/// Audio envelope representing energy levels in time-windowed hops.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioEnvelope {
    pub hop_seconds: f64,
    pub samples: Vec<f32>,
}

impl AudioEnvelope {
    pub fn new(hop_seconds: f64, samples: Vec<f32>) -> Self {
        Self {
            hop_seconds,
            samples,
        }
    }

    /// Total duration of the envelope in seconds.
    pub fn duration(&self) -> f64 {
        self.samples.len() as f64 * self.hop_seconds
    }

    /// Number of hops in this envelope.
    pub fn count(&self) -> usize {
        self.samples.len()
    }

    /// True if the envelope is empty.
    pub fn is_empty(&self) -> bool {
        self.samples.is_empty()
    }
}

pub const DEFAULT_SAMPLE_RATE: f64 = 16_000.0;
pub const DEFAULT_HOP_SECONDS: f64 = 0.0025; // 2.5 ms hops (400 hops/sec)

/// Computes a log-energy RMS envelope from mono float audio samples.
///
/// Matches PalmierPro's `AudioEnvelopeExtractor`:
/// `hop_size = max(1, round(sample_rate * hop_seconds))`
/// `log_energy = log1p(sqrt(sum_squares / count) * 100.0)`
pub fn compute_envelope(samples: &[f32], sample_rate: f64, hop_seconds: f64) -> AudioEnvelope {
    if samples.is_empty() || sample_rate <= 0.0 || hop_seconds <= 0.0 {
        return AudioEnvelope::new(hop_seconds, Vec::new());
    }

    let hop_size = ((sample_rate * hop_seconds).round() as usize).max(1);
    let estimated_hops = samples.len().div_ceil(hop_size);
    let mut envelope_samples = Vec::with_capacity(estimated_hops);

    let mut sum_squares = 0.0f32;
    let mut carry = 0usize;

    for &sample in samples {
        sum_squares += sample * sample;
        carry += 1;
        if carry == hop_size {
            envelope_samples.push(log_energy(sum_squares, hop_size));
            sum_squares = 0.0;
            carry = 0;
        }
    }

    if carry > 0 {
        envelope_samples.push(log_energy(sum_squares, carry));
    }

    AudioEnvelope::new(hop_seconds, envelope_samples)
}

#[inline]
fn log_energy(sum_squares: f32, count: usize) -> f32 {
    if count == 0 {
        return 0.0;
    }
    let rms = (sum_squares / count as f32).sqrt();
    (rms * 100.0).ln_1p()
}
