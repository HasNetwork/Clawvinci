// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only
// Derived from Sources/PalmierPro/Agent/Tools/ToolExecutor+Texts.swift and Captions.swift (GPLv3).

use crate::result::ToolResult;
use crate::state::McpState;
use clawvinci_model::text_style::{Rgba, TextStyle};
use clawvinci_model::timeline::Clip;
use serde_json::{json, Value};
use uuid::Uuid;

pub fn add_texts(args: &Value, state: &mut McpState) -> ToolResult {
    let track_idx = args.get("trackIndex").and_then(|v| v.as_i64()).unwrap_or(0) as usize;
    let texts_arr = match args.get("texts").and_then(|v| v.as_array()) {
        Some(arr) => arr,
        None => return ToolResult::error("Missing required parameter 'texts'"),
    };

    let mut added_ids = Vec::new();

    let res = state.editor.perform("Add Texts", |tl| {
        if track_idx >= tl.tracks.len() {
            return Err(clawvinci_timeline::error::TimelineError::TrackNotFound(
                track_idx.to_string(),
            ));
        }

        for t_val in texts_arr {
            let text_content = t_val
                .get("text")
                .and_then(|v| v.as_str())
                .unwrap_or("Title")
                .to_string();
            let start = t_val.get("startFrame").and_then(|v| v.as_i64()).unwrap_or(0);
            let duration = t_val
                .get("durationFrames")
                .and_then(|v| v.as_i64())
                .unwrap_or(90)
                .max(1);

            let font_name = t_val
                .get("fontName")
                .and_then(|v| v.as_str())
                .unwrap_or("Segoe UI")
                .to_string();
            let font_size = t_val.get("fontSize").and_then(|v| v.as_f64()).unwrap_or(64.0);
            let color_hex = t_val.get("color").and_then(|v| v.as_str()).unwrap_or("#FFFFFF");

            let mut style = TextStyle {
                font_name,
                font_size,
                ..Default::default()
            };
            if let Some(c) = Rgba::from_hex(color_hex) {
                style.color = c;
            }

            let clip_id = Uuid::new_v4().to_string();
            added_ids.push(clip_id.clone());

            let mut clip = Clip::new(clip_id, "text://title".to_string(), start, duration);
            clip.text = Some(text_content);
            clip.text_style = Some(style);
            tl.tracks[track_idx].clips.push(clip);
        }
        Ok(())
    });

    match res {
        Ok(()) => {
            state.bump_version();
            ToolResult::json(&json!({
                "addedTextClipIds": added_ids,
                "trackIndex": track_idx
            }))
        }
        Err(e) => ToolResult::error(format!("Failed to add texts: {e}")),
    }
}

pub fn update_text(args: &Value, state: &mut McpState) -> ToolResult {
    let clip_id = match args.get("clipId").and_then(|v| v.as_str()) {
        Some(id) => id,
        None => return ToolResult::error("Missing required parameter 'clipId'"),
    };

    let text_content = args.get("text").and_then(|v| v.as_str());
    let font_name = args.get("fontName").and_then(|v| v.as_str());
    let font_size = args.get("fontSize").and_then(|v| v.as_f64());
    let color_hex = args.get("color").and_then(|v| v.as_str());

    let res = state.editor.perform("Update Text", |tl| {
        for track in &mut tl.tracks {
            for clip in &mut track.clips {
                if clip.id == clip_id {
                    if let Some(t) = text_content {
                        clip.text = Some(t.to_string());
                    }
                    if font_name.is_some() || font_size.is_some() || color_hex.is_some() {
                        let mut style = clip.text_style.clone().unwrap_or_default();
                        if let Some(fn_str) = font_name {
                            style.font_name = fn_str.to_string();
                        }
                        if let Some(fs) = font_size {
                            style.font_size = fs;
                        }
                        if let Some(hex) = color_hex {
                            if let Some(c) = Rgba::from_hex(hex) {
                                style.color = c;
                            }
                        }
                        clip.text_style = Some(style);
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
            ToolResult::ok(format!("Updated text clip '{clip_id}'"))
        }
        Err(e) => ToolResult::error(format!("Failed to update text clip: {e}")),
    }
}

pub fn add_captions(args: &Value, state: &mut McpState) -> ToolResult {
    let track_idx = args.get("trackIndex").and_then(|v| v.as_i64()).unwrap_or(0) as usize;
    let caption_group_id = Uuid::new_v4().to_string();

    state.bump_version();
    ToolResult::json(&json!({
        "captionGroupId": caption_group_id,
        "trackIndex": track_idx,
        "clipCount": 0,
        "frameRange": [0, 0]
    }))
}
