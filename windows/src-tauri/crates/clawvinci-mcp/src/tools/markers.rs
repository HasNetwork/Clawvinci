// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only
// Derived from Sources/PalmierPro/Agent/Tools/ToolExecutor+Markers.swift (GPLv3).

use crate::result::ToolResult;
use crate::state::{McpMarker, McpState};
use serde_json::{json, Value};
use uuid::Uuid;

pub fn manage_markers(args: &Value, state: &mut McpState) -> ToolResult {
    let action = args.get("action").and_then(|v| v.as_str()).unwrap_or("");
    match action {
        "create" => {
            let name = args.get("name").and_then(|v| v.as_str()).unwrap_or("Marker");
            let start_frame = args.get("startFrame").and_then(|v| v.as_i64()).unwrap_or(0);
            let duration_frames = args.get("durationFrames").and_then(|v| v.as_i64()).unwrap_or(0);
            let color = args
                .get("color")
                .and_then(|v| v.as_str())
                .unwrap_or("#00E5FF")
                .to_string();
            let comment = args.get("comment").and_then(|v| v.as_str()).map(|s| s.to_string());
            let status = args
                .get("status")
                .and_then(|v| v.as_str())
                .unwrap_or("open")
                .to_string();

            let id = Uuid::new_v4().to_string();
            let marker = McpMarker {
                id: id.clone(),
                name: name.to_string(),
                comment,
                color,
                start_frame,
                duration_frames,
                status,
            };
            state.markers.push(marker);
            state.bump_version();

            ToolResult::json(&json!({
                "markerId": id,
                "created": true,
                "startFrame": start_frame
            }))
        }
        "update" => {
            let marker_id = match args.get("markerId").and_then(|v| v.as_str()) {
                Some(id) => id,
                None => return ToolResult::error("Missing required parameter 'markerId'"),
            };

            let marker = match state.markers.iter_mut().find(|m| m.id == marker_id) {
                Some(m) => m,
                None => return ToolResult::error(format!("Marker '{marker_id}' not found")),
            };

            if let Some(name) = args.get("name").and_then(|v| v.as_str()) {
                marker.name = name.to_string();
            }
            if let Some(start) = args.get("startFrame").and_then(|v| v.as_i64()) {
                marker.start_frame = start;
            }
            if let Some(dur) = args.get("durationFrames").and_then(|v| v.as_i64()) {
                marker.duration_frames = dur;
            }
            if let Some(color) = args.get("color").and_then(|v| v.as_str()) {
                marker.color = color.to_string();
            }
            if let Some(comment) = args.get("comment").and_then(|v| v.as_str()) {
                marker.comment = Some(comment.to_string());
            }
            if let Some(status) = args.get("status").and_then(|v| v.as_str()) {
                marker.status = status.to_string();
            }

            state.bump_version();
            ToolResult::ok(format!("Updated marker '{marker_id}'"))
        }
        "delete" => {
            let marker_id = match args.get("markerId").and_then(|v| v.as_str()) {
                Some(id) => id,
                None => return ToolResult::error("Missing required parameter 'markerId'"),
            };

            let before = state.markers.len();
            state.markers.retain(|m| m.id != marker_id);
            if state.markers.len() < before {
                state.bump_version();
                ToolResult::ok(format!("Deleted marker '{marker_id}'"))
            } else {
                ToolResult::error(format!("Marker '{marker_id}' not found"))
            }
        }
        "list" => ToolResult::json(&json!({ "markers": state.markers })),
        other => ToolResult::error(format!("Unsupported marker action: '{other}'")),
    }
}
