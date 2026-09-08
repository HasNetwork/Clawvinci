// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only

use clawvinci_search::error::SearchError;
use clawvinci_search::transcription::{
    DeterministicLocalTranscriber, LocalWhisperEngine, LocalWhisperTranscriber,
    TranscriptCache, TranscriptionBackend, TranscriptionBackendConfig,
    TranscriptionEngineMode, TranscriptionService, WhisperAudioPreprocessor,
    WHISPER_N_MELS, WHISPER_SAMPLE_RATE,
};
use serde_json::json;
use std::f32::consts::PI;
use std::fs;
use std::sync::Arc;

#[test]
fn test_byok_config_default_and_unconfigured_error() {
    let config = TranscriptionBackendConfig::default();
    assert!(config.endpoint.is_none());
    assert!(config.api_key.is_none());
    assert!(config.model.is_none());

    let backend = TranscriptionBackend::new(config);
    assert!(!backend.is_configured());

    // Calling transcribe_audio without config should fail loudly
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

    rt.block_on(async {
        let err = backend
            .transcribe_audio(vec![0u8; 100], "test.wav", None)
            .await
            .expect_err("Should error when unconfigured");

        match err {
            SearchError::CloudTranscriptionFailed(msg) => {
                assert!(msg.contains("No BYOK transcription endpoint configured"));
            }
            other => panic!("Unexpected error variant: {other:?}"),
        }
    });
}

#[test]
fn test_openai_verbose_json_response_parsing() {
    let response_json = json!({
        "task": "transcribe",
        "language": "english",
        "duration": 4.5,
        "text": "Welcome to Clawvinci video editor.",
        "words": [
            { "word": "Welcome", "start": 0.0, "end": 0.5 },
            { "word": "to", "start": 0.5, "end": 0.7 },
            { "word": "Clawvinci", "start": 0.7, "end": 1.4 },
            { "word": "video", "start": 1.5, "end": 2.0 },
            { "word": "editor", "start": 2.0, "end": 2.6 }
        ],
        "segments": [
            {
                "id": 0,
                "seek": 0,
                "start": 0.0,
                "end": 2.6,
                "text": "Welcome to Clawvinci video editor."
            }
        ]
    });

    let result = TranscriptionBackend::parse_whisper_response(&response_json).unwrap();
    assert_eq!(result.text, "Welcome to Clawvinci video editor.");
    assert_eq!(result.language.as_deref(), Some("english"));
    assert_eq!(result.words.len(), 5);
    assert_eq!(result.words[0].text, "Welcome");
    assert_eq!(result.words[0].start, Some(0.0));
    assert_eq!(result.words[0].end, Some(0.5));
    assert_eq!(result.words[2].text, "Clawvinci");

    assert_eq!(result.segments.len(), 1);
    assert_eq!(result.segments[0].start, 0.0);
    assert_eq!(result.segments[0].end, 2.6);
}

#[test]
fn test_whisper_audio_preprocessor_resampling_and_mel_bins() {
    // 48 kHz sine wave (1 second = 48000 samples)
    let sample_rate = 48000.0;
    let num_samples = 48000;
    let mut input_audio = Vec::with_capacity(num_samples);
    for i in 0..num_samples {
        let t = i as f32 / sample_rate as f32;
        let s = (2.0 * PI * 440.0 * t).sin();
        input_audio.push(s);
    }

    let resampled = WhisperAudioPreprocessor::resample_to_16k(&input_audio, sample_rate);
    assert_eq!(resampled.len(), 16000);

    // Compute log mel-spectrogram
    let mel = WhisperAudioPreprocessor::log_mel_spectrogram(&resampled);
    assert!(!mel.is_empty());
    assert_eq!(mel[0].len(), WHISPER_N_MELS);
}

#[test]
fn test_deterministic_local_transcriber_and_engine() {
    let engine = LocalWhisperEngine::new(std::env::temp_dir());
    engine.prepare();
    assert!(engine.is_ready());

    // Generate 2 seconds of pulsed synthetic audio (speech bursts)
    let sample_rate = WHISPER_SAMPLE_RATE;
    let mut samples = vec![0.0f32; (sample_rate * 2.0) as usize];
    for (i, sample) in samples.iter_mut().enumerate() {
        let t = i as f64 / sample_rate;
        // Two 0.5s bursts of energy
        if (0.2..0.7).contains(&t) || (1.1..1.6).contains(&t) {
            *sample = (2.0 * PI * 300.0 * t as f32).sin() * 0.8;
        }
    }

    let result = engine.transcribe(&samples, sample_rate).unwrap();
    assert!(!result.text.is_empty());
    assert!(!result.segments.is_empty());
    assert!(!result.words.is_empty());

    for word in &result.words {
        assert!(word.start.is_some());
        assert!(word.end.is_some());
        assert!(word.end.unwrap() >= word.start.unwrap());
    }

    // Empty input check
    let empty_res = engine.transcribe(&[], sample_rate).unwrap();
    assert!(empty_res.words.is_empty());
    assert!(empty_res.segments.is_empty());
}

#[test]
fn test_transcription_service_cache_and_modes() {
    let temp_dir = std::env::temp_dir().join(format!("clawvinci_svc_test_{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&temp_dir).unwrap();

    let cache = Arc::new(TranscriptCache::new(temp_dir.join("cache")));
    let local_engine = Arc::new(LocalWhisperEngine::new(temp_dir.join("models")));
    local_engine.prepare();

    let service = TranscriptionService::new(
        cache.clone(),
        local_engine,
        TranscriptionBackendConfig::default(),
    );

    // Create a dummy WAV file
    let audio_file = temp_dir.join("test_sample.wav");
    let mut dummy_wav = Vec::new();
    dummy_wav.extend_from_slice(b"RIFF");
    dummy_wav.extend_from_slice(&36u32.to_le_bytes());
    dummy_wav.extend_from_slice(b"WAVE");
    dummy_wav.extend_from_slice(b"fmt ");
    dummy_wav.extend_from_slice(&16u32.to_le_bytes());
    dummy_wav.extend_from_slice(&1u16.to_le_bytes());
    dummy_wav.extend_from_slice(&1u16.to_le_bytes());
    dummy_wav.extend_from_slice(&16000u32.to_le_bytes());
    dummy_wav.extend_from_slice(&32000u32.to_le_bytes());
    dummy_wav.extend_from_slice(&2u16.to_le_bytes());
    dummy_wav.extend_from_slice(&16u16.to_le_bytes());
    dummy_wav.extend_from_slice(b"data");
    dummy_wav.extend_from_slice(&1600u32.to_le_bytes());
    dummy_wav.extend_from_slice(&vec![128u8; 1600]);
    fs::write(&audio_file, &dummy_wav).unwrap();

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

    rt.block_on(async {
        // Transcribe with Local mode
        let res = service
            .transcribe_file(&audio_file, Some(TranscriptionEngineMode::Local))
            .await
            .unwrap();

        assert_eq!(res.language.as_deref(), Some("en"));

        // Verify it was stored in disk cache
        assert!(cache.has_cached_on_disk(&audio_file, None));

        // Clear memory cache to ensure disk hit
        cache.clear_memory();
        let cached_res = service
            .transcribe_file(&audio_file, None)
            .await
            .unwrap();
        assert_eq!(cached_res.text, res.text);
    });

    let _ = fs::remove_dir_all(&temp_dir);
}
