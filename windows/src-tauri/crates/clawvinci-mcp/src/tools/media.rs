// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only
// Derived from Sources/PalmierPro/Agent/Tools/ToolExecutor+Media.swift and Import.swift (GPLv3).

use crate::result::ToolResult;
use crate::state::{McpMediaItem, McpState};
use serde_json::{json, Value};
use std::path::Path;
use uuid::Uuid;

pub fn get_media(args: &Value, state: &mut McpState) -> ToolResult {
    let ids_filter = args.get("ids").and_then(|v| v.as_array()).map(|arr| {
        arr.iter()
            .filter_map(|x| x.as_str())
            .collect::<Vec<_>>()
    });
    let folder_filter = args.get("folder").and_then(|v| v.as_str());
    let pending_only = args.get("pending").and_then(|v| v.as_bool()).unwrap_or(false);

    let filtered = state
        .media_items
        .iter()
        .filter(|item| {
            if let Some(ref ids) = ids_filter {
                if !ids.contains(&item.id.as_str()) {
                    return false;
                }
            }
            if let Some(folder) = folder_filter {
                if item.folder.as_deref() != Some(folder) {
                    return false;
                }
            }
            if pending_only && item.generation_status.is_none() {
                return false;
            }
            true
        })
        .collect::<Vec<_>>();

    let timelines_json = state
        .child_timelines
        .iter()
        .map(|(id, tl)| {
            json!({
                "timelineId": id,
                "name": tl.name,
                "active": false
            })
        })
        .chain(std::iter::once(json!({
            "timelineId": state.active_timeline_id,
            "name": state.editor.timeline().name,
            "active": true
        })))
        .collect::<Vec<_>>();

    ToolResult::json(&json!({
        "assets": filtered,
        "timelines": timelines_json
    }))
}

pub fn inspect_media(args: &Value, state: &mut McpState) -> ToolResult {
    let media_ref = match args.get("mediaRef").and_then(|v| v.as_str()) {
        Some(m) => m,
        None => return ToolResult::error("Missing required parameter 'mediaRef'"),
    };

    let item = match state.media_items.iter().find(|i| i.id == media_ref) {
        Some(i) => i,
        None => return ToolResult::error(format!("Media reference '{media_ref}' not found")),
    };

    ToolResult::json(&json!({
        "mediaRef": item.id,
        "name": item.name,
        "mediaType": item.media_type,
        "durationSeconds": item.duration_seconds,
        "width": item.width,
        "height": item.height,
        "fps": item.fps,
        "hasAudio": item.has_audio,
        "folder": item.folder,
        "segments": [
            { "text": "Sample dialogue segment", "startSeconds": 0.0, "endSeconds": item.duration_seconds }
        ]
    }))
}

pub fn search_media(args: &Value, state: &mut McpState) -> ToolResult {
    let query = match args.get("query").and_then(|v| v.as_str()) {
        Some(q) => q.trim(),
        None => return ToolResult::error("Missing required parameter 'query'"),
    };

    if query.is_empty() {
        return ToolResult::error("search_media: query is empty");
    }

    let scope = args.get("scope").and_then(|v| v.as_str()).unwrap_or("both");
    if scope != "visual" && scope != "spoken" && scope != "both" {
        return ToolResult::error(format!(
            "search_media: scope must be visual, spoken, or both (got '{scope}')"
        ));
    }

    let limit = args
        .get("limit")
        .and_then(|v| v.as_i64())
        .unwrap_or(10)
        .clamp(1, 50) as usize;

    let media_ref_filter = args.get("mediaRef").and_then(|v| v.as_str());

    let filtered_items: Vec<&McpMediaItem> = state
        .media_items
        .iter()
        .filter(|item| {
            if let Some(target_ref) = media_ref_filter {
                item.id == target_ref
            } else {
                true
            }
        })
        .collect();

    let query_lower = query.to_lowercase();
    let query_terms = clawvinci_search::transcription::TranscriptSearch::terms(&query_lower);

    let mut moments = Vec::new();
    if scope != "spoken" {
        // Visual search scoring
        for item in &filtered_items {
            let matches_name = item.name.to_lowercase().contains(&query_lower);
            let matches_prompt = item
                .generation_prompt
                .as_ref()
                .map(|p| p.to_lowercase().contains(&query_lower))
                .unwrap_or(false);

            let score = if matches_prompt {
                0.96
            } else if matches_name {
                0.88
            } else {
                0.72
            };

            if item.media_type == "image" {
                moments.push(json!({
                    "mediaRef": item.id,
                    "name": item.name,
                    "score": score,
                    "type": "image"
                }));
            } else {
                moments.push(json!({
                    "mediaRef": item.id,
                    "name": item.name,
                    "score": score,
                    "startSeconds": 0.0,
                    "endSeconds": item.duration_seconds
                }));
            }
        }
        moments.truncate(limit);
    }

    let mut spoken = Vec::new();
    if scope != "visual" {
        // Spoken transcript search
        for item in &filtered_items {
            let text = item
                .generation_prompt
                .clone()
                .unwrap_or_else(|| item.name.clone());
            if clawvinci_search::transcription::TranscriptSearch::matches(&text, &query_terms) {
                spoken.push(json!({
                    "mediaRef": item.id,
                    "name": item.name,
                    "startSeconds": 0.0,
                    "endSeconds": item.duration_seconds,
                    "text": text
                }));
            }
        }
        spoken.truncate(limit);
    }

    let fps = state.editor.timeline().fps;
    let mut payload = serde_json::Map::new();
    payload.insert("timelineFps".to_string(), json!(fps));

    if scope != "spoken" {
        payload.insert("moments".to_string(), json!(moments));
        payload.insert(
            "index".to_string(),
            json!({
                "status": "ready",
                "indexableAssets": filtered_items.len()
            }),
        );
    }
    if scope != "visual" {
        payload.insert("spoken".to_string(), json!(spoken));
    }

    ToolResult::json(&Value::Object(payload))
}

pub fn import_media(args: &Value, state: &mut McpState) -> ToolResult {
    let source = match args.get("source").and_then(|v| v.as_object()) {
        Some(s) => s,
        None => return ToolResult::error("Missing required parameter 'source'"),
    };

    let folder = args.get("folder").and_then(|v| v.as_str()).map(|f| f.to_string());

    let (path_str, name) = if let Some(p) = source.get("path").and_then(|v| v.as_str()) {
        let fname = Path::new(p)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("imported_media")
            .to_string();
        (p.to_string(), fname)
    } else if let Some(u) = source.get("url").and_then(|v| v.as_str()) {
        let fname = u.split('/').next_back().unwrap_or("downloaded_media").to_string();
        (u.to_string(), fname)
    } else if source.contains_key("bytes") {
        ("inline_bytes".to_string(), "inline_asset".to_string())
    } else {
        return ToolResult::error("Source must contain 'path', 'url', or 'bytes'");
    };

    let id = Uuid::new_v4().to_string();
    let media_type = if name.ends_with(".wav") || name.ends_with(".mp3") || name.ends_with(".aac") {
        "audio".to_string()
    } else if name.ends_with(".png") || name.ends_with(".jpg") || name.ends_with(".jpeg") {
        "image".to_string()
    } else {
        "video".to_string()
    };

    let item = McpMediaItem {
        id: id.clone(),
        name: name.clone(),
        path: path_str,
        media_type,
        duration_seconds: 10.0,
        width: Some(1920),
        height: Some(1080),
        fps: Some(30.0),
        has_audio: true,
        folder,
        generation_prompt: None,
        generation_status: None,
    };

    state.media_items.push(item);
    state.bump_version();

    ToolResult::json(&json!({
        "mediaRef": id,
        "name": name,
        "status": "ready"
    }))
}

pub fn organize_media(args: &Value, state: &mut McpState) -> ToolResult {
    let media_ref = match args.get("mediaRef").and_then(|v| v.as_str()) {
        Some(m) => m,
        None => return ToolResult::error("Missing required parameter 'mediaRef'"),
    };

    let action = args.get("action").and_then(|v| v.as_str()).unwrap_or("");
    match action {
        "move" => {
            let folder = args.get("folder").and_then(|v| v.as_str());
            if let Some(item) = state.media_items.iter_mut().find(|i| i.id == media_ref) {
                item.folder = folder.map(|f| f.to_string());
                state.bump_version();
                ToolResult::ok(format!("Moved media '{media_ref}' to folder '{folder:?}'"))
            } else {
                ToolResult::error(format!("Media '{media_ref}' not found"))
            }
        }
        "delete" => {
            let before = state.media_items.len();
            state.media_items.retain(|i| i.id != media_ref);
            if state.media_items.len() < before {
                state.bump_version();
                ToolResult::ok(format!("Deleted media '{media_ref}'"))
            } else {
                ToolResult::error(format!("Media '{media_ref}' not found"))
            }
        }
        other => ToolResult::error(format!("Unsupported organize action: '{other}'")),
    }
}
