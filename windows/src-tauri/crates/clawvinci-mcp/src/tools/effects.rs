// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only
// Derived from Sources/PalmierPro/Agent/Tools/ToolExecutor+Effect.swift and Denoise.swift (GPLv3).

use crate::result::ToolResult;
use crate::state::McpState;
use clawvinci_model::effect::{Effect, EffectParam};
use serde_json::{json, Value};

pub fn apply_effect(args: &Value, state: &mut McpState) -> ToolResult {
    let clip_id = match args.get("clipId").and_then(|v| v.as_str()) {
        Some(id) => id,
        None => return ToolResult::error("Missing required parameter 'clipId'"),
    };
    let effect_type = match args.get("effectType").and_then(|v| v.as_str()) {
        Some(t) => t,
        None => return ToolResult::error("Missing required parameter 'effectType'"),
    };
    let action = args.get("action").and_then(|v| v.as_str()).unwrap_or("add");

    let res = state.editor.perform("Apply Effect", |tl| {
        for track in &mut tl.tracks {
            for clip in &mut track.clips {
                if clip.id == clip_id {
                    if action == "remove" {
                        clip.effects.retain(|e| e.effect_type != effect_type);
                    } else {
                        let mut fx = Effect::new(effect_type);
                        if let Some(params_obj) = args.get("params").and_then(|v| v.as_object()) {
                            for (k, v) in params_obj {
                                if let Some(f) = v.as_f64() {
                                    fx.params.insert(k.clone(), EffectParam::Float(f));
                                } else if let Some(b) = v.as_bool() {
                                    fx.params.insert(k.clone(), EffectParam::Bool(b));
                                }
                            }
                        }
                        clip.effects.retain(|e| e.effect_type != effect_type);
                        clip.effects.push(fx);
                    }
                    return Ok(());
                }
            }
        }
        Err(clawvinci_timeline::error::TimelineError::ClipNotFound(
            clip_id.to_string(),
        ))
    });

    match res {
        Ok(()) => {
            state.bump_version();
            ToolResult::ok(format!("Applied effect '{effect_type}' on clip '{clip_id}' (action: {action})"))
        }
        Err(e) => ToolResult::error(format!("Failed to apply effect: {e}")),
    }
}

pub fn denoise_audio(args: &Value, state: &mut McpState) -> ToolResult {
    let clip_id = match args.get("clipId").and_then(|v| v.as_str()) {
        Some(id) => id,
        None => return ToolResult::error("Missing required parameter 'clipId'"),
    };
    let intensity = args.get("intensity").and_then(|v| v.as_f64()).unwrap_or(0.7);

    let res = state.editor.perform("Denoise Audio", |tl| {
        for track in &mut tl.tracks {
            for clip in &mut track.clips {
                if clip.id == clip_id {
                    let mut fx = Effect::new("audio.denoise");
                    fx.params.insert("intensity".to_string(), EffectParam::Float(intensity));
                    clip.effects.retain(|e| e.effect_type != "audio.denoise");
                    clip.effects.push(fx);
                    return Ok(());
                }
            }
        }
        Err(clawvinci_timeline::error::TimelineError::ClipNotFound(
            clip_id.to_string(),
        ))
    });

    match res {
        Ok(()) => {
            state.bump_version();
            ToolResult::json(&json!({
                "clipId": clip_id,
                "denoised": true,
                "intensity": intensity
            }))
        }
        Err(e) => ToolResult::error(format!("Failed to denoise audio: {e}")),
    }
}
