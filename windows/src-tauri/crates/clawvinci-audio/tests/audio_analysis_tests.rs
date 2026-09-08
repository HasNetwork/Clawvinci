// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only

use clawvinci_audio::beats::{estimate_bpm, pick_peaks, BeatAnalysis, BeatDetector, BeatStore};
use clawvinci_audio::envelope::{compute_envelope, DEFAULT_HOP_SECONDS, DEFAULT_SAMPLE_RATE};
use clawvinci_audio::meter::{
    AudioLevelAnalyzer, AudioMeterAnalysis, AudioMeterChannelState, AudioMeterHub,
};
use clawvinci_audio::silence::{SilenceRemovalPlanner, SilenceRemovalSettings};
use clawvinci_audio::speaker::{
    cosine_similarity, normalize_vector, turns_from_words, SpeakerRegistry, SpeakerTurn,
    TranscriptWordInfo,
};
use clawvinci_audio::sync::{AudioSyncCorrelator, AudioSyncResult};
use clawvinci_audio::vad::{
    SpeechMaskStore, VadAnalysis, VadSpan, VoiceActivityDetector, VAD_CHUNK_DURATION,
};
use std::f32::consts::PI;

#[test]
fn test_audio_envelope_extraction() {
    let sample_rate = DEFAULT_SAMPLE_RATE;
    let duration_secs = 0.5;
    let total_samples = (sample_rate * duration_secs) as usize;

    // Synthesize 440 Hz sine wave
    let mut samples = Vec::with_capacity(total_samples);
    for i in 0..total_samples {
        let t = i as f32 / sample_rate as f32;
        samples.push((2.0 * PI * 440.0 * t).sin() * 0.8);
    }

    let env = compute_envelope(&samples, sample_rate, DEFAULT_HOP_SECONDS);
    assert!(!env.is_empty());
    assert_eq!(env.count(), env.samples.len());
    assert!((env.duration() - duration_secs).abs() < 0.01);
    assert!(env.samples.iter().all(|&s| s > 0.0));

    // Zero amplitude produces zero energy
    let silence = vec![0.0f32; total_samples];
    let env_silence = compute_envelope(&silence, sample_rate, DEFAULT_HOP_SECONDS);
    assert!(env_silence.samples.iter().all(|&s| s == 0.0));
}

#[test]
fn test_audio_meter_decay_and_clipping() {
    let mut channel = AudioMeterChannelState::new();
    assert_eq!(channel.display(0.0).level_db, channel.floor_db);

    // Ingest 0 dBFS (peak = 1.0) at t = 1.0
    channel.ingest(1.0, 1.0);
    let d1 = channel.display(1.0);
    assert!((d1.level_db - 0.0).abs() < 0.1);
    assert!((d1.peak_db - 0.0).abs() < 0.1);
    assert!(d1.clipped);

    // After 1.0 second (t = 2.0):
    // Level should decay by 24 dB/s -> -24 dBFS
    // Peak should be held (peak_hold_seconds = 1.5s) -> 0 dBFS
    let d2 = channel.display(2.0);
    assert!((d2.level_db - -24.0).abs() < 0.1);
    assert!((d2.peak_db - 0.0).abs() < 0.1);

    // After 2.5 seconds (t = 3.5):
    // Peak elapsed = 3.5 - (1.0 + 1.5) = 1.0s -> decays by 18 dB/s -> -18 dBFS
    let d3 = channel.display(3.5);
    assert!((d3.peak_db - -18.0).abs() < 0.1);

    // Reset clipping
    channel.reset_clipping();
    assert!(!channel.display(3.5).clipped);

    // Hub test
    let mut hub = AudioMeterHub::new();
    let analysis = AudioMeterAnalysis::new(0.5, 0.8);
    hub.ingest(&analysis, 1.0);
    let stereo = hub.display(1.0);
    assert!(!stereo.left.clipped);
    assert!(!stereo.right.clipped);
    hub.reset();

    // Analyzer tests
    let left = vec![0.1f32, -0.7f32, 0.3f32];
    let right = vec![0.2f32, 0.4f32, -0.9f32];
    let analyzed = AudioLevelAnalyzer::analyze_floats(&left, &right, 0..3);
    assert!((analyzed.left_peak - 0.7).abs() < 1e-5);
    assert!((analyzed.right_peak - 0.9).abs() < 1e-5);

    let interleaved = vec![0.1f32, 0.2f32, -0.7f32, 0.4f32, 0.3f32, -0.9f32];
    let interl_analysis = AudioLevelAnalyzer::analyze_interleaved(&interleaved, 2);
    assert!((interl_analysis.left_peak - 0.7).abs() < 1e-5);
    assert!((interl_analysis.right_peak - 0.9).abs() < 1e-5);
}

#[test]
fn test_audio_sync_cross_correlation() {
    // Generate deterministic reference pattern (400 hops)
    let n = 400;
    let mut reference = vec![0.1f32; n];
    // Add distinct pulses
    for i in (50..n).step_by(60) {
        reference[i] = 0.9;
        reference[i + 1] = 0.8;
        reference[i + 2] = 0.7;
    }

    // Target is reference delayed by +40 hops
    let lag = 40i64;
    let mut target = vec![0.1f32; 200];
    for i in 0..200 {
        let ref_idx = (i as i64 + lag) as usize;
        if ref_idx < reference.len() {
            target[i] = reference[ref_idx];
        }
    }

    let result = AudioSyncCorrelator::correlate(&reference, &target, 100, 0, 16);
    assert!(result.is_some(), "Cross-correlation should find match");
    let res = result.unwrap();
    assert_eq!(res.lag_hops, lag, "Detected lag should match exact delay");
    assert!(res.confidence > 0.95, "Confidence should be high for identical signal");

    // AudioSyncResult constructor test
    let res_manual = AudioSyncResult::new(lag, 0.98);
    assert_eq!(res_manual.lag_hops, lag);
}

#[test]
fn test_silence_removal_planner() {
    let settings = SilenceRemovalSettings::new(0.5, 0.15).unwrap();
    assert_eq!(settings.minimum_pause_seconds, 0.5);
    assert_eq!(settings.speech_padding_seconds, 0.15);

    // cell_duration = 0.032s (32 ms)
    // 0.5s min pause = ceil(0.5 / 0.032) = 16 cells
    // 0.15s padding = ceil(0.15 / 0.032) = 5 cells
    let cell_dur = 0.032;

    // Create mask:
    // 0..10: speech (false)
    // 10..15: short pause 5 cells (not removable, < 16)
    // 15..30: speech (false)
    // 30..60: long pause 30 cells (removable, >= 16)
    // 60..80: speech (false)
    let mut mask = vec![false; 80];
    for i in 10..15 {
        mask[i] = true;
    }
    for i in 30..60 {
        mask[i] = true;
    }

    let removable = SilenceRemovalPlanner::removable_mask(&mask, &settings, cell_dur);
    assert_eq!(removable.len(), 80);

    // Short pause (10..15) should NOT be marked removable
    for i in 10..15 {
        assert!(!removable[i], "Short pause cell {} should not be removable", i);
    }

    // Long pause (30..60) should have 5 cells padding on each side:
    // Start = 30 + 5 = 35
    // End = 60 - 5 = 55
    for i in 30..35 {
        assert!(!removable[i], "Padding cell {} should not be removable", i);
    }
    for i in 35..55 {
        assert!(removable[i], "Interior cell {} should be removable", i);
    }
    for i in 55..60 {
        assert!(!removable[i], "Padding cell {} should not be removable", i);
    }

    let visible_ranges = SilenceRemovalPlanner::visible_removable_ranges(
        &removable,
        0.0..80.0 * 0.032 * 30.0,
        30,
        &settings,
        cell_dur,
    );
    assert!(!visible_ranges.is_empty());
}

#[test]
fn test_voice_activity_detection() {
    let sr = 16_000;
    // 0.5s silence + 0.5s 400Hz speech-like tone + 0.5s silence
    let mut samples = vec![0.0f32; (sr as f32 * 1.5) as usize];
    let tone_start = (sr as f32 * 0.5) as usize;
    let tone_end = (sr as f32 * 1.0) as usize;

    for i in tone_start..tone_end {
        let t = (i - tone_start) as f32 / sr as f32;
        samples[i] = (2.0 * PI * 400.0 * t).sin() * 0.7;
    }

    let vad = VoiceActivityDetector::analyze(&samples);
    assert!(!vad.segments.is_empty(), "VAD should detect tone segment");
    let span = &vad.segments[0];
    assert!(span.duration() > 0.0);
    assert!(span.start >= 0.45 && span.start <= 0.55);
    assert!(span.end >= 0.95 && span.end <= 1.2); // Includes hangover

    let mask = vad.to_mask();
    assert_eq!(mask.len(), vad.chunk_count);
    let speech_cell = (0.75 / VAD_CHUNK_DURATION) as usize;
    assert!(mask[speech_cell]);

    // VadAnalysis and VadSpan manual constructor
    let manual_span = VadSpan::new(1.0, 2.0);
    let manual_analysis = VadAnalysis::new(100, vec![manual_span]);
    assert_eq!(manual_analysis.chunk_count, 100);

    // SpeechMaskStore tests
    let mut store = SpeechMaskStore::new();
    store.insert_speech_mask("asset-1", mask.clone());
    assert_eq!(store.speech_mask("asset-1"), Some(&mask));
    let quiet_mask = store.quiet_non_speech_mask("asset-1", &samples);
    assert!(quiet_mask.is_some());
    store.invalidate("asset-1");
    assert_eq!(store.speech_mask("asset-1"), None);
}

#[test]
fn test_beat_detector_and_bpm_estimation() {
    let beats = vec![0.5, 1.0, 1.5, 2.0, 2.5, 3.0];
    let bpm = estimate_bpm(&beats).unwrap();
    assert!((bpm - 120.0).abs() < 0.1, "Median interval 0.5s should yield 120 BPM");

    // Peak picking test
    let logits = vec![-5.0f32, 5.0f32, -5.0f32];
    let peaks = pick_peaks(&logits, 0.5);
    assert_eq!(peaks.len(), 1);

    // Synthesize 120 BPM click track at 22050 Hz (2 seconds = 4 clicks)
    let sr = 22050.0;
    let mut samples = vec![0.0f32; (sr * 2.0) as usize];
    for &b in &[0.25, 0.75, 1.25, 1.75] {
        let start = (b * sr) as usize;
        for i in 0..200 {
            if start + i < samples.len() {
                samples[start + i] = 0.9 * (-(i as f32) / 40.0).exp();
            }
        }
    }

    let analysis = BeatDetector::detect(&samples, sr);
    assert!(!analysis.beats.is_empty(), "Should detect clicks as beats");
    assert!(analysis.bpm > 100.0 && analysis.bpm < 140.0);

    // Store test
    let store = BeatStore::new();
    store.insert("asset-1", analysis.clone());
    assert_eq!(store.get("asset-1"), Some(analysis));
    store.invalidate("asset-1");
    assert_eq!(store.get("asset-1"), None);

    let empty = BeatAnalysis::EMPTY;
    assert_eq!(empty.bpm, 0.0);
}

#[test]
fn test_speaker_turns_and_similarity() {
    let words = vec![
        TranscriptWordInfo {
            text: "Hello".into(),
            speaker: Some("Alice".into()),
            start: Some(0.0),
            end: Some(0.5),
        },
        TranscriptWordInfo {
            text: "there".into(),
            speaker: Some("Alice".into()),
            start: Some(0.6), // < 1.0s gap
            end: Some(1.1),
        },
        TranscriptWordInfo {
            text: "Hi!".into(),
            speaker: Some("Bob".into()),
            start: Some(1.5),
            end: Some(2.0),
        },
    ];

    let turns = turns_from_words(&words);
    assert_eq!(turns.len(), 2);
    assert_eq!(turns[0].speaker, "Alice");
    assert_eq!(turns[0].start, 0.0);
    assert_eq!(turns[0].end, 1.1);
    assert!(turns[0].duration() > 1.0);
    assert_eq!(turns[1].speaker, "Bob");
    assert_eq!(turns[1].start, 1.5);
    assert_eq!(turns[1].end, 2.0);

    let manual_turn = SpeakerTurn::new("Charlie", 3.0, 4.0);
    assert_eq!(manual_turn.duration(), 1.0);

    // Vector normalization & similarity
    let norm = normalize_vector(&[3.0, 4.0]);
    assert!((norm[0] - 0.6).abs() < 1e-5);
    assert!((norm[1] - 0.8).abs() < 1e-5);

    let v1 = vec![1.0, 0.0, 0.0];
    let v2 = vec![1.0, 0.0, 0.0];
    let v3 = vec![0.0, 1.0, 0.0];
    assert!((cosine_similarity(&v1, &v2) - 1.0).abs() < 1e-5);
    assert_eq!(cosine_similarity(&v1, &v3), 0.0);

    let mut registry = SpeakerRegistry::new();
    let id1 = registry.assign_or_create(&v1);
    let id2 = registry.assign_or_create(&v2);
    let id3 = registry.assign_or_create(&v3);
    assert_eq!(id1, id2, "Similar vectors should receive same speaker ID");
    assert_ne!(id1, id3, "Orthogonal vectors should receive different speaker ID");
    assert_eq!(registry.len(), 2);
    assert!(!registry.is_empty());
}
