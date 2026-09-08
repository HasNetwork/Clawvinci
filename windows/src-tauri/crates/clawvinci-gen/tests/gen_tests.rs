// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only

use clawvinci_gen::backend::{
    BackendGenerationJob, BackendGenerationStatus, GenerationBackendClient, MockGenerationBackend,
};
use clawvinci_gen::catalog::{
    CostEstimator, ModelCatalog, ModelModality, ModelPreferences,
};
use clawvinci_gen::edit::EditActionKind;
use clawvinci_gen::error::GenError;
use clawvinci_gen::preprocessing::{ImageConverter, TrimmedSource};
use clawvinci_gen::service::GenerationService;
use clawvinci_gen::submission::{
    AudioGenerationSubmission, ImageGenerationSubmission, MusicGenerationSubmission,
    MusicMode, VideoGenerationSubmission,
};
use clawvinci_model::clip_type::ClipType;
use clawvinci_model::media_manifest::{GenerationInput, MediaManifest};
use clawvinci_model::timeline::Timeline;
use std::path::Path;
use std::sync::Arc;
use tempfile::tempdir;
use tokio_util::sync::CancellationToken;

#[test]
fn test_catalog_defaults_and_lookup() {
    let catalog = ModelCatalog::default_catalog();
    assert!(!catalog.entries.is_empty());

    let video_models = catalog.models_for_modality(ModelModality::Video);
    assert!(video_models.len() >= 2);
    assert!(video_models.iter().any(|m| m.id == "seedance-v1-fast"));
    assert!(video_models.iter().any(|m| m.id == "kling-v1"));

    let seedance = catalog.get("seedance-v1-fast").expect("seedance model found");
    assert_eq!(seedance.display_name, "Seedance Video Fast");
    let caps = seedance.video_caps().expect("video caps present");
    assert!(caps.durations.contains(&5));
    assert!(caps.durations.contains(&10));
    assert!(caps.resolutions.contains(&"720p".to_string()));
    assert!(caps.aspect_ratios.contains(&"16:9".to_string()));

    let image_models = catalog.models_for_modality(ModelModality::Image);
    assert!(image_models.iter().any(|m| m.id == "nano-banana-pro"));
    assert!(image_models.iter().any(|m| m.id == "flux-1-schnell"));

    let audio_models = catalog.models_for_modality(ModelModality::Audio);
    assert!(audio_models.iter().any(|m| m.id == "eleven-multilingual-v2"));

    let music_models = catalog.models_for_modality(ModelModality::Music);
    assert!(music_models.iter().any(|m| m.id == "suno-v3-5"));

    let upscale_models = catalog.models_for_modality(ModelModality::Upscale);
    assert!(upscale_models.iter().any(|m| m.id == "real-esrgan-4x"));
}

#[test]
fn test_model_preferences() {
    let mut prefs = ModelPreferences::default();
    assert!(prefs.is_enabled("seedance-v1-fast"));
    assert_eq!(prefs.default_for_modality(ModelModality::Video), "seedance-v1-fast");

    prefs.set_enabled("seedance-v1-fast", false);
    assert!(!prefs.is_enabled("seedance-v1-fast"));

    prefs.set_enabled("seedance-v1-fast", true);
    assert!(prefs.is_enabled("seedance-v1-fast"));
}

#[test]
fn test_cost_estimation() {
    let catalog = ModelCatalog::default_catalog();

    // 1. Video cost (Seedance)
    let seedance = catalog.get("seedance-v1-fast").unwrap();
    // 5 seconds @ 720p (rate = 2.0 credits/s) -> 10 credits
    let cost = CostEstimator::estimate_video_cost(seedance, 5, Some("720p"), true, false, false);
    assert_eq!(cost, Some(10));

    // 10 seconds @ 1080p (rate = 4.0 credits/s) without audio (0.85 discount) -> ceil(4.0 * 0.85 * 10) = 34
    let cost_no_audio = CostEstimator::estimate_video_cost(seedance, 10, Some("1080p"), false, false, false);
    assert_eq!(cost_no_audio, Some(34));

    // 5 seconds draft mode (rate = 1.0) -> 5 credits
    let cost_draft = CostEstimator::estimate_video_cost(seedance, 5, Some("720p"), true, true, false);
    assert_eq!(cost_draft, Some(5));

    // 2. Image cost (Nano Banana Pro)
    let banana = catalog.get("nano-banana-pro").unwrap();
    let img_cost = CostEstimator::estimate_image_cost(banana, Some("1024x1024"), None, 1);
    assert_eq!(img_cost, Some(1));
    let img_cost_hd = CostEstimator::estimate_image_cost(banana, Some("1920x1080"), None, 2);
    assert_eq!(img_cost_hd, Some(4));

    // 3. Speech cost (ElevenLabs)
    let eleven = catalog.get("eleven-multilingual-v2").unwrap();
    // 500 chars at 2.0 per 1000 chars -> ceil(1.0) = 1 credit
    let speech_cost = CostEstimator::estimate_speech_cost(eleven, 500, None);
    assert_eq!(speech_cost, Some(1));

    // 4. Music cost (Suno)
    let suno = catalog.get("suno-v3-5").unwrap();
    // 30 seconds at 0.2/s -> 6 credits
    let music_cost = CostEstimator::estimate_music_cost(suno, 30);
    assert_eq!(music_cost, Some(6));

    // 5. Upscale cost (Real-ESRGAN)
    let esrgan = catalog.get("real-esrgan-4x").unwrap();
    let upscale_cost = CostEstimator::estimate_upscale_cost(esrgan, 10, 2.0);
    assert_eq!(upscale_cost, Some(15));
}

#[test]
fn test_submission_payloads() {
    let gen_input = GenerationInput {
        prompt: "A cinematic cybernetic tiger walking in neon rain".to_string(),
        model: "seedance-v1-fast".to_string(),
        duration: 5,
        aspect_ratio: "16:9".to_string(),
        resolution: Some("1080p".to_string()),
        upscale_settings: None,
        upscale_source_width: None,
        upscale_source_height: None,
        upscale_source_fps: None,
        quality: None,
        image_urls: None,
        num_images: None,
        voice: None,
        lyrics: None,
        style_instructions: None,
        instrumental: None,
        target_language: None,
        multilingual: None,
        audio_input: None,
        generate_audio: Some(true),
        draft: Some(false),
        uses_source_video: None,
        reference_image_urls: None,
        reference_video_urls: None,
        reference_audio_urls: None,
        image_url_asset_ids: None,
        reference_image_asset_ids: None,
        reference_video_asset_ids: None,
        reference_audio_asset_ids: None,
        created_at: None,
        backend_job_id: None,
        output_index: None,
        result_urls: None,
        cost_credits: None,
        refunded_credits: None,
    };

    // Video submission
    let video_sub = VideoGenerationSubmission::new(gen_input.clone(), 5.0)
        .with_name("Cyber Tiger")
        .with_folder_id("vfx-folder");

    let params = video_sub.build_params(None, None, None, vec!["https://mock/ref.png".to_string()], vec![], vec![]);
    let json_val = serde_json::to_value(&params).expect("serializes to json");
    assert_eq!(json_val["prompt"], "A cinematic cybernetic tiger walking in neon rain");
    assert_eq!(json_val["duration"], 5);
    assert_eq!(json_val["aspectRatio"], "16:9");

    // Image submission
    let image_sub = ImageGenerationSubmission::new(gen_input.clone(), 2);
    let img_params = image_sub.build_params(vec!["https://mock/style.png".to_string()]);
    let img_json = serde_json::to_value(&img_params).expect("serializes to json");
    assert_eq!(img_json["numImages"], 2);

    // Audio submission
    let audio_sub = AudioGenerationSubmission::new(gen_input.clone());
    let audio_params = audio_sub.build_params(None, None, None, None);
    let audio_json = serde_json::to_value(&audio_params).expect("serializes to json");
    assert_eq!(audio_json["prompt"], "A cinematic cybernetic tiger walking in neon rain");

    // Music submission
    let music_sub = MusicGenerationSubmission::new(MusicMode::TextToMusic, gen_input, 30, 0);
    let music_params = music_sub.build_params(None);
    let music_json = serde_json::to_value(&music_params).expect("serializes to json");
    assert_eq!(music_json["durationSeconds"], 30);
}

#[tokio::test]
async fn test_mock_backend_lifecycle() {
    let mock = MockGenerationBackend::new();
    let params = clawvinci_gen::submission::types::BackendGenerationParams::Video(
        clawvinci_gen::submission::types::VideoGenerationParams {
            prompt: "Test prompt".to_string(),
            duration: 5,
            aspect_ratio: "16:9".to_string(),
            resolution: Some("720p".to_string()),
            source_video_url: None,
            start_frame_url: None,
            end_frame_url: None,
            reference_image_urls: vec![],
            reference_video_urls: vec![],
            reference_audio_urls: vec![],
            generate_audio: true,
            draft: None,
        },
    );

    let job_id = mock.submit("seedance-v1-fast", &params, None).await.expect("submission succeeds");
    assert!(job_id.starts_with("mock-job-"));

    let job = mock.get_job(&job_id).await.expect("job found");
    assert_eq!(job.status, BackendGenerationStatus::Succeeded);
    assert_eq!(job.progress, 1.0);
    assert!(!job.result_urls.is_empty());

    let dir = tempdir().expect("tempdir");
    let dest = dir.path().join("output.mp4");
    let cancel = CancellationToken::new();

    mock.download_file(&job.result_urls[0], &dest, &cancel)
        .await
        .expect("download succeeds");
    assert!(dest.exists());
    let content = tokio::fs::read(&dest).await.expect("file read");
    assert_eq!(content, b"MOCK_GENERATED_MEDIA_PAYLOAD");
}

#[tokio::test]
async fn test_service_lifecycle_and_timeline() {
    let mock = Arc::new(MockGenerationBackend::new());
    let service = GenerationService::new(mock);

    let gen_input = GenerationInput {
        prompt: "Sunset over cyber ocean".to_string(),
        model: "seedance-v1-fast".to_string(),
        duration: 5,
        aspect_ratio: "16:9".to_string(),
        resolution: Some("720p".to_string()),
        upscale_settings: None,
        upscale_source_width: None,
        upscale_source_height: None,
        upscale_source_fps: None,
        quality: None,
        image_urls: None,
        num_images: None,
        voice: None,
        lyrics: None,
        style_instructions: None,
        instrumental: None,
        target_language: None,
        multilingual: None,
        audio_input: None,
        generate_audio: Some(true),
        draft: None,
        uses_source_video: None,
        reference_image_urls: None,
        reference_video_urls: None,
        reference_audio_urls: None,
        image_url_asset_ids: None,
        reference_image_asset_ids: None,
        reference_video_asset_ids: None,
        reference_audio_asset_ids: None,
        created_at: None,
        backend_job_id: None,
        output_index: None,
        result_urls: None,
        cost_credits: None,
        refunded_credits: None,
    };

    let dir = tempdir().expect("tempdir");
    let (mut placeholder, dest_path) = service.create_placeholder(
        ClipType::Video,
        "Sunset Cyber Ocean",
        5.0,
        gen_input,
        dir.path(),
        "mp4",
    );

    assert_eq!(placeholder.generation_status.as_deref(), Some("preparing"));

    let mut manifest = MediaManifest::default();
    GenerationService::<MockGenerationBackend>::register_in_manifest(&mut manifest, placeholder.clone());
    assert_eq!(manifest.entries.len(), 1);

    let sub = VideoGenerationSubmission::new(placeholder.generation_input.clone().unwrap(), 5.0);
    let params = sub.build_params(None, None, None, vec![], vec![], vec![]);

    let job_id = service
        .submit("seedance-v1-fast", &params, None, &mut placeholder)
        .await
        .expect("submit succeeds");

    assert_eq!(placeholder.generation_status.as_deref(), Some("generating"));
    assert_eq!(
        placeholder.generation_input.as_ref().unwrap().backend_job_id.as_deref(),
        Some(job_id.as_str())
    );

    let cancel = CancellationToken::new();
    let job = service
        .monitor_and_finalize(&job_id, &dest_path, &mut placeholder, &cancel)
        .await
        .expect("finalize succeeds");

    assert_eq!(job.status, BackendGenerationStatus::Succeeded);
    assert_eq!(placeholder.generation_status.as_deref(), Some("ready"));
    assert!(dest_path.exists());

    // Timeline clip placement
    let mut timeline = Timeline::new(30, 1920, 1080);
    timeline.tracks.push(clawvinci_model::timeline::Track::new(ClipType::Video));
    let clip_id = GenerationService::<MockGenerationBackend>::place_clip_on_timeline(
        &mut timeline,
        0,
        &placeholder.id,
        0,
        150,
    )
    .expect("places clip");

    assert_eq!(timeline.tracks[0].clips.len(), 1);
    assert_eq!(timeline.tracks[0].clips[0].id, clip_id);
    assert_eq!(timeline.tracks[0].clips[0].media_ref, placeholder.id);
    assert_eq!(timeline.total_frames(), 150);
}

#[tokio::test]
async fn test_service_cancellation() {
    let mock = Arc::new(MockGenerationBackend::new());
    let job_id = "test-running-job".to_string();

    mock.register_job(BackendGenerationJob {
        id: job_id.clone(),
        status: BackendGenerationStatus::Running,
        progress: 0.25,
        result_urls: vec![],
        error_message: None,
        cost_credits: None,
        refunded_credits: None,
        completed_at: None,
    })
    .await;

    let service = GenerationService::new(mock);
    let dir = tempdir().expect("tempdir");
    let (mut placeholder, dest_path) = service.create_placeholder(
        ClipType::Video,
        "Cancel Test",
        5.0,
        GenerationInput {
            prompt: "Cancel prompt".to_string(),
            model: "seedance-v1-fast".to_string(),
            duration: 5,
            aspect_ratio: "16:9".to_string(),
            resolution: None,
            upscale_settings: None,
            upscale_source_width: None,
            upscale_source_height: None,
            upscale_source_fps: None,
            quality: None,
            image_urls: None,
            num_images: None,
            voice: None,
            lyrics: None,
            style_instructions: None,
            instrumental: None,
            target_language: None,
            multilingual: None,
            audio_input: None,
            generate_audio: None,
            draft: None,
            uses_source_video: None,
            reference_image_urls: None,
            reference_video_urls: None,
            reference_audio_urls: None,
            image_url_asset_ids: None,
            reference_image_asset_ids: None,
            reference_video_asset_ids: None,
            reference_audio_asset_ids: None,
            created_at: None,
            backend_job_id: None,
            output_index: None,
            result_urls: None,
            cost_credits: None,
            refunded_credits: None,
        },
        dir.path(),
        "mp4",
    );

    let cancel = CancellationToken::new();
    cancel.cancel(); // Pre-cancel token

    let result = service
        .monitor_and_finalize(&job_id, &dest_path, &mut placeholder, &cancel)
        .await;

    assert!(matches!(result, Err(GenError::Cancelled)));
    assert_eq!(placeholder.generation_status.as_deref(), Some("cancelled"));
}

#[test]
fn test_edit_actions() {
    let video_actions = EditActionKind::available_for(ClipType::Video, false);
    assert!(video_actions.contains(&EditActionKind::Upscale));
    assert!(video_actions.contains(&EditActionKind::LipSync));
    assert!(video_actions.contains(&EditActionKind::Reframe));

    let generating_actions = EditActionKind::available_for(ClipType::Video, true);
    assert!(generating_actions.is_empty());

    let image_actions = EditActionKind::available_for(ClipType::Image, false);
    assert!(image_actions.contains(&EditActionKind::Upscale));
    assert!(image_actions.contains(&EditActionKind::CreateVideo));
    assert!(!image_actions.contains(&EditActionKind::LipSync));

    assert!(EditActionKind::Upscale.requires_paid_plan());
    assert!(!EditActionKind::GenerateMusic.requires_paid_plan());
}

#[test]
fn test_preprocessing_types() {
    let trim = TrimmedSource::new(Path::new("test.mp4"), 30, 60, 30.0);
    assert_eq!(trim.start_time_seconds(), 1.0);
    assert_eq!(trim.duration_seconds(), 2.0);

    assert!(ImageConverter::requires_conversion(Path::new("photo.heic")));
    assert!(ImageConverter::requires_conversion(Path::new("photo.TIFF")));
    assert!(ImageConverter::requires_conversion(Path::new("graphic.bmp")));
    assert!(!ImageConverter::requires_conversion(Path::new("photo.jpg")));
    assert!(!ImageConverter::requires_conversion(Path::new("photo.png")));
}
