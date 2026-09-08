// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only
// Derived from Sources/PalmierPro/Agent/Tools/ToolExecutor+Generate.swift (GPLv3).

use crate::result::ToolResult;
use crate::state::{McpMediaItem, McpState};
use clawvinci_gen::catalog::{CostEstimator, ModelCatalog, ModelModality};
use serde_json::{json, Value};
use uuid::Uuid;

pub fn list_models(_args: &Value, _state: &mut McpState) -> ToolResult {
    let catalog = ModelCatalog::default_catalog();

    let video_models: Vec<Value> = catalog
        .models_for_modality(ModelModality::Video)
        .iter()
        .map(|m| {
            let caps = m.video_caps();
            json!({
                "id": m.id,
                "name": m.display_name,
                "provider": m.provider,
                "durations": caps.map(|c| &c.durations),
                "resolutions": caps.map(|c| &c.resolutions),
                "aspectRatios": caps.map(|c| &c.aspect_ratios),
                "description": m.description,
            })
        })
        .collect();

    let image_models: Vec<Value> = catalog
        .models_for_modality(ModelModality::Image)
        .iter()
        .map(|m| {
            let caps = m.image_caps();
            json!({
                "id": m.id,
                "name": m.display_name,
                "provider": m.provider,
                "resolutions": caps.map(|c| &c.resolutions),
                "aspectRatios": caps.map(|c| &c.aspect_ratios),
                "qualities": caps.map(|c| &c.qualities),
                "description": m.description,
            })
        })
        .collect();

    let audio_models: Vec<Value> = catalog
        .models_for_modality(ModelModality::Audio)
        .iter()
        .map(|m| {
            let caps = m.audio_caps();
            json!({
                "id": m.id,
                "name": m.display_name,
                "provider": m.provider,
                "category": caps.map(|c| &c.category),
                "voices": caps.map(|c| &c.voices),
                "description": m.description,
            })
        })
        .collect();

    let music_models: Vec<Value> = catalog
        .models_for_modality(ModelModality::Music)
        .iter()
        .map(|m| {
            let caps = m.audio_caps();
            json!({
                "id": m.id,
                "name": m.display_name,
                "provider": m.provider,
                "durations": caps.map(|c| &c.durations),
                "description": m.description,
            })
        })
        .collect();

    let upscale_models: Vec<Value> = catalog
        .models_for_modality(ModelModality::Upscale)
        .iter()
        .map(|m| {
            let caps = m.upscale_caps();
            json!({
                "id": m.id,
                "name": m.display_name,
                "provider": m.provider,
                "speed": caps.map(|c| &c.speed),
                "maxFactor": caps.and_then(|c| c.maximum_upscale_factor),
                "supportedTypes": caps.map(|c| &c.supported_types),
                "description": m.description,
            })
        })
        .collect();

    ToolResult::json(&json!({
        "video": video_models,
        "image": image_models,
        "speech": audio_models,
        "music": music_models,
        "upscale": upscale_models,
    }))
}

pub fn generate_video(args: &Value, state: &mut McpState) -> ToolResult {
    let prompt = match args.get("prompt").and_then(|v| v.as_str()) {
        Some(p) => p,
        None => return ToolResult::error("Missing required parameter 'prompt'"),
    };
    let duration = args.get("durationSeconds").and_then(|v| v.as_f64()).unwrap_or(5.0);
    let model_id = args.get("model").and_then(|v| v.as_str()).unwrap_or("seedance-v1-fast");
    let aspect_ratio = args.get("aspectRatio").and_then(|v| v.as_str()).unwrap_or("16:9");
    let resolution = args.get("resolution").and_then(|v| v.as_str()).unwrap_or("720p");

    let catalog = ModelCatalog::default_catalog();
    let estimated_cost = catalog.get(model_id).and_then(|m| {
        CostEstimator::estimate_video_cost(m, duration as i32, Some(resolution), true, false, false)
    });

    let media_id = Uuid::new_v4().to_string();
    let short_id = if media_id.len() >= 8 { &media_id[..8] } else { &media_id };
    let filename = format!("gen-{short_id}.mp4");

    let item = McpMediaItem {
        id: media_id.clone(),
        name: format!("Generated: {}", &prompt.chars().take(24).collect::<String>()),
        path: format!("media/{filename}"),
        media_type: "video".to_string(),
        duration_seconds: duration,
        width: Some(1280),
        height: Some(720),
        fps: Some(30.0),
        has_audio: true,
        folder: args.get("folder").and_then(|v| v.as_str()).map(|s| s.to_string()),
        generation_prompt: Some(prompt.to_string()),
        generation_status: Some("generating".to_string()),
    };
    state.media_items.push(item);
    state.bump_version();

    ToolResult::json(&json!({
        "mediaRef": media_id,
        "prompt": prompt,
        "model": model_id,
        "aspectRatio": aspect_ratio,
        "resolution": resolution,
        "durationSeconds": duration,
        "estimatedCostCredits": estimated_cost,
        "status": "generating"
    }))
}

pub fn generate_image(args: &Value, state: &mut McpState) -> ToolResult {
    let prompt = match args.get("prompt").and_then(|v| v.as_str()) {
        Some(p) => p,
        None => return ToolResult::error("Missing required parameter 'prompt'"),
    };
    let model_id = args.get("model").and_then(|v| v.as_str()).unwrap_or("nano-banana-pro");
    let aspect_ratio = args.get("aspectRatio").and_then(|v| v.as_str()).unwrap_or("1:1");

    let catalog = ModelCatalog::default_catalog();
    let estimated_cost = catalog.get(model_id).and_then(|m| {
        CostEstimator::estimate_image_cost(m, Some("1024x1024"), None, 1)
    });

    let media_id = Uuid::new_v4().to_string();
    let short_id = if media_id.len() >= 8 { &media_id[..8] } else { &media_id };
    let filename = format!("gen-{short_id}.jpg");

    let item = McpMediaItem {
        id: media_id.clone(),
        name: format!("Generated: {}", &prompt.chars().take(24).collect::<String>()),
        path: format!("media/{filename}"),
        media_type: "image".to_string(),
        duration_seconds: 4.0,
        width: Some(1024),
        height: Some(1024),
        fps: None,
        has_audio: false,
        folder: args.get("folder").and_then(|v| v.as_str()).map(|s| s.to_string()),
        generation_prompt: Some(prompt.to_string()),
        generation_status: Some("ready".to_string()),
    };
    state.media_items.push(item);
    state.bump_version();

    ToolResult::json(&json!({
        "mediaRef": media_id,
        "prompt": prompt,
        "model": model_id,
        "aspectRatio": aspect_ratio,
        "estimatedCostCredits": estimated_cost,
        "status": "ready"
    }))
}

pub fn generate_audio(args: &Value, state: &mut McpState) -> ToolResult {
    let prompt = match args.get("prompt").and_then(|v| v.as_str()) {
        Some(p) => p,
        None => return ToolResult::error("Missing required parameter 'prompt'"),
    };
    let audio_type = args.get("audioType").and_then(|v| v.as_str()).unwrap_or("speech");
    let duration = args.get("durationSeconds").and_then(|v| v.as_f64()).unwrap_or(15.0);

    let catalog = ModelCatalog::default_catalog();
    let (model_id, estimated_cost) = if audio_type == "music" {
        let id = "suno-v3-5";
        let cost = catalog.get(id).and_then(|m| CostEstimator::estimate_music_cost(m, duration as i32));
        (id, cost)
    } else {
        let id = "eleven-multilingual-v2";
        let cost = catalog.get(id).and_then(|m| CostEstimator::estimate_speech_cost(m, prompt.len(), None));
        (id, cost)
    };

    let media_id = Uuid::new_v4().to_string();
    let short_id = if media_id.len() >= 8 { &media_id[..8] } else { &media_id };
    let filename = format!("gen-{short_id}.mp3");

    let item = McpMediaItem {
        id: media_id.clone(),
        name: format!("Audio: {}", &prompt.chars().take(24).collect::<String>()),
        path: format!("media/{filename}"),
        media_type: "audio".to_string(),
        duration_seconds: duration,
        width: None,
        height: None,
        fps: None,
        has_audio: true,
        folder: None,
        generation_prompt: Some(prompt.to_string()),
        generation_status: Some("ready".to_string()),
    };
    state.media_items.push(item);
    state.bump_version();

    ToolResult::json(&json!({
        "mediaRef": media_id,
        "prompt": prompt,
        "audioType": audio_type,
        "model": model_id,
        "durationSeconds": duration,
        "estimatedCostCredits": estimated_cost,
        "status": "ready"
    }))
}

pub fn upscale_media(args: &Value, state: &mut McpState) -> ToolResult {
    let media_ref = match args.get("mediaRef").and_then(|v| v.as_str()) {
        Some(m) => m,
        None => return ToolResult::error("Missing required parameter 'mediaRef'"),
    };
    let factor = args.get("scaleFactor").and_then(|v| v.as_f64()).unwrap_or(2.0);

    let catalog = ModelCatalog::default_catalog();
    let estimated_cost = catalog.get("real-esrgan-4x").and_then(|m| {
        CostEstimator::estimate_upscale_cost(m, 10, factor)
    });

    state.bump_version();
    ToolResult::json(&json!({
        "originalMediaRef": media_ref,
        "upscaledMediaRef": format!("{media_ref}-upscaled-{factor:.0}x"),
        "scaleFactor": factor,
        "estimatedCostCredits": estimated_cost,
        "status": "ready"
    }))
}
