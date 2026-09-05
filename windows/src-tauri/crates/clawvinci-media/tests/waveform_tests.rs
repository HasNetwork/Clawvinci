// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only

use clawvinci_media::*;

#[test]
fn test_waveform_silence() {
    let samples = vec![0.0f32; 44100]; // 1 second of silence
    let data = downsample_pcm(&samples, 44100, 100);

    assert_eq!(data.buckets_per_second, 100);
    assert_eq!(data.peaks.len(), 100);
    assert_eq!(data.rms.len(), 100);

    for &peak in &data.peaks {
        assert_eq!(peak, 0.0);
    }
    for &rms in &data.rms {
        assert_eq!(rms, 0.0);
    }
}

#[test]
fn test_waveform_square_wave() {
    // 1 second of full scale square wave (+1.0 and -1.0 alternating every 100 samples)
    let mut samples = Vec::with_capacity(44100);
    for i in 0..44100 {
        let val = if (i / 100) % 2 == 0 { 1.0f32 } else { -1.0f32 };
        samples.push(val);
    }

    let data = downsample_pcm(&samples, 44100, 100);

    assert_eq!(data.peaks.len(), 100);
    for &peak in &data.peaks {
        assert!((peak - 1.0).abs() < 1e-5);
    }
    for &rms in &data.rms {
        assert!((rms - 1.0).abs() < 1e-5);
    }
}

#[test]
fn test_waveform_sine_wave() {
    // 1 second of 440 Hz Sine wave at amplitude 0.8
    let mut samples = Vec::with_capacity(44100);
    for i in 0..44100 {
        let t = i as f32 / 44100.0;
        let val = 0.8 * (2.0 * std::f32::consts::PI * 440.0 * t).sin();
        samples.push(val);
    }

    let data = downsample_pcm(&samples, 44100, 100);
    assert_eq!(data.peaks.len(), 100);

    for &peak in &data.peaks {
        assert!(peak <= 0.801 && peak >= 0.79);
    }

    // For a sine wave of amplitude A, RMS is A / sqrt(2) ≈ 0.8 / 1.4142 ≈ 0.5657
    for &rms in &data.rms {
        assert!(rms <= 0.60 && rms >= 0.53);
    }
}

#[test]
fn test_waveform_empty_samples() {
    let data = downsample_pcm(&[], 44100, 100);
    assert_eq!(data.duration_seconds, 0.0);
    assert!(data.peaks.is_empty());
    assert!(data.rms.is_empty());
}
