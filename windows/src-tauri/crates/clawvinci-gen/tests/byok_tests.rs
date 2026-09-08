// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only

use clawvinci_gen::backend::{ByokGenerationBackend, ByokProviderConfig, GenerationBackendClient};
use clawvinci_gen::error::GenError;
use clawvinci_gen::submission::BackendGenerationParams;

#[test]
fn test_byok_provider_config_and_inference() {
    let backend = ByokGenerationBackend::new();
    assert!(!backend.is_provider_configured("kling"));
    assert!(!backend.is_provider_configured("seedance"));
    assert!(!backend.is_provider_configured("fal"));

    // Provider inference
    assert_eq!(ByokGenerationBackend::infer_provider("kling-v1-standard"), "kling");
    assert_eq!(ByokGenerationBackend::infer_provider("seedance-v1-fast"), "seedance");
    assert_eq!(ByokGenerationBackend::infer_provider("nano-banana-pro"), "fal");
    assert_eq!(ByokGenerationBackend::infer_provider("elevenlabs-multilingual-v2"), "elevenlabs");
    assert_eq!(ByokGenerationBackend::infer_provider("suno-v3"), "suno");
    assert_eq!(ByokGenerationBackend::infer_provider("dall-e-3"), "openai");

    // Configure a provider
    let config = ByokProviderConfig::new("kling", Some("kling_test_key_123".into()), None);
    backend.set_provider_config(config.clone());
    assert!(backend.is_provider_configured("kling"));

    let retrieved = backend.get_provider_config("kling").expect("Must be found");
    assert_eq!(retrieved.provider, "kling");
    assert_eq!(retrieved.api_key.as_deref(), Some("kling_test_key_123"));
    assert!(retrieved.endpoint.is_none());
}

#[test]
fn test_byok_backend_missing_key_error() {
    let backend = ByokGenerationBackend::new();
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

    rt.block_on(async {
        let params = BackendGenerationParams::Video {
            prompt: "Sunset on beach".to_string(),
            duration_seconds: 5,
            aspect_ratio: "16:9".to_string(),
            resolution: "720p".to_string(),
            reference_image_urls: vec![],
            reference_video_urls: vec![],
            reference_audio_urls: vec![],
            first_frame_url: None,
            last_frame_url: None,
            source_video_url: None,
            seed: None,
        };

        // Submitting for seedance without key must fail with actionable error
        let err = backend
            .submit("seedance-v1-fast", &params, None)
            .await
            .expect_err("Should error when provider key is missing");

        match err {
            GenError::Backend(msg) => {
                assert!(msg.contains("seedance"));
                assert!(msg.contains("BYOK") || msg.contains("settings"));
            }
            other => panic!("Unexpected error variant: {other:?}"),
        }
    });
}
