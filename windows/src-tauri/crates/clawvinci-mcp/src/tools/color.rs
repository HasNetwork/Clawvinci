// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only
// Derived from Sources/PalmierPro/Agent/Tools/ToolExecutor+Color.swift (GPLv3).

use crate::result::ToolResult;
use crate::state::McpState;
use clawvinci_model::effect::{Effect, EffectParam};
use serde_json::{json, Value};

pub fn apply_color(args: &Value, state: &mut McpState) -> ToolResult {
    let clip_id = match args.get("clipId").and_then(|v| v.as_str()) {
        Some(id) => id,
        None => return ToolResult::error("Missing required parameter 'clipId'"),
    };

    let exposure = args.get("exposure").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let contrast = args.get("contrast").and_then(|v| v.as_f64()).unwrap_or(1.0);
    let saturation = args.get("saturation").and_then(|v| v.as_f64()).unwrap_or(1.0);
    let temperature = args.get("temperature").and_then(|v| v.as_f64()).unwrap_or(0.0);

    let res = state.editor.perform("Apply Color Grade", |tl| {
        for track in &mut tl.tracks {
            for clip in &mut track.clips {
                if clip.id == clip_id {
                    let mut color_fx = Effect::new("color.grade");
                    color_fx.params.insert("exposure".to_string(), EffectParam::Float(exposure));
                    color_fx.params.insert("contrast".to_string(), EffectParam::Float(contrast));
                    color_fx.params.insert("saturation".to_string(), EffectParam::Float(saturation));
                    color_fx.params.insert("temperature".to_string(), EffectParam::Float(temperature));

                    clip.effects.retain(|e| e.effect_type != "color.grade");
                    clip.effects.push(color_fx);
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
            ToolResult::ok(format!("Applied color grade to clip '{clip_id}'"))
        }
        Err(e) => ToolResult::error(format!("Failed to apply color grade: {e}")),
    }
}

pub fn inspect_color(args: &Value, state: &mut McpState) -> ToolResult {
    let clip_id = match args.get("clipId").and_then(|v| v.as_str()) {
        Some(id) => id,
        None => return ToolResult::error("Missing required parameter 'clipId'"),
    };

    let timeline = state.editor.timeline();
    for track in &timeline.tracks {
        for clip in &track.clips {
            if clip.id == clip_id {
                let color_fx = clip.effects.iter().find(|e| e.effect_type == "color.grade");
                return ToolResult::json(&json!({
                    "clipId": clip_id,
                    "hasGrade": color_fx.is_some(),
                    "grade": color_fx
                }));
            }
        }
    }

    ToolResult::error(format!("Clip '{clip_id}' not found"))
}
