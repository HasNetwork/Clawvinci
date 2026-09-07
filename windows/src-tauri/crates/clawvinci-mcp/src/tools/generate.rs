// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only
// Derived from Sources/PalmierPro/Agent/Tools/ToolExecutor+Generate.swift (GPLv3).

use crate::result::ToolResult;
use crate::state::McpState;
use serde_json::{json, Value};
use uuid::Uuid;

pub fn list_models(_args: &Value, _state: &mut McpState) -> ToolResult {
    ToolResult::json(&json!({
        "video": [
            { "id": "gen-video-fast", "name": "Clawvinci Video Fast (Preview)", "maxDuration": 10 }
        ],
        "image": [
            { "id": "gen-image-sdxl", "name": "Stable Diffusion XL", "resolutions": ["1024x1024", "1920x1080"] }
        ],
        "audio": [
            { "id": "gen-tts-natural", "name": "Neural Speech Synthesizer" },
            { "id": "gen-music-ambient", "name": "Cinematic Soundtrack Engine" }
        ],
        "upscale": [
            { "id": "upscale-real-esrgan", "factors": [2, 4] }
        ]
    }))
}

pub fn generate_video(args: &Value, state: &mut McpState) -> ToolResult {
    let prompt = match args.get("prompt").and_then(|v| v.as_str()) {
        Some(p) => p,
        None => return ToolResult::error("Missing required parameter 'prompt'"),
    };
    let duration = args.get("durationSeconds").and_then(|v| v.as_f64()).unwrap_or(5.0);

    let media_ref = Uuid::new_v4().to_string();
    state.bump_version();

    ToolResult::json(&json!({
        "mediaRef": media_ref,
        "prompt": prompt,
        "durationSeconds": duration,
        "status": "generating"
    }))
}

pub fn generate_image(args: &Value, state: &mut McpState) -> ToolResult {
    let prompt = match args.get("prompt").and_then(|v| v.as_str()) {
        Some(p) => p,
        None => return ToolResult::error("Missing required parameter 'prompt'"),
    };

    let media_ref = Uuid::new_v4().to_string();
    state.bump_version();

    ToolResult::json(&json!({
        "mediaRef": media_ref,
        "prompt": prompt,
        "status": "ready"
    }))
}

pub fn generate_audio(args: &Value, state: &mut McpState) -> ToolResult {
    let prompt = match args.get("prompt").and_then(|v| v.as_str()) {
        Some(p) => p,
        None => return ToolResult::error("Missing required parameter 'prompt'"),
    };
    let audio_type = args.get("audioType").and_then(|v| v.as_str()).unwrap_or("speech");

    let media_ref = Uuid::new_v4().to_string();
    state.bump_version();

    ToolResult::json(&json!({
        "mediaRef": media_ref,
        "prompt": prompt,
        "audioType": audio_type,
        "status": "ready"
    }))
}

pub fn upscale_media(args: &Value, state: &mut McpState) -> ToolResult {
    let media_ref = match args.get("mediaRef").and_then(|v| v.as_str()) {
        Some(m) => m,
        None => return ToolResult::error("Missing required parameter 'mediaRef'"),
    };
    let factor = args.get("scaleFactor").and_then(|v| v.as_i64()).unwrap_or(2);

    state.bump_version();
    ToolResult::json(&json!({
        "originalMediaRef": media_ref,
        "upscaledMediaRef": format!("{media_ref}-upscaled-{factor}x"),
        "scaleFactor": factor,
        "status": "ready"
    }))
}
