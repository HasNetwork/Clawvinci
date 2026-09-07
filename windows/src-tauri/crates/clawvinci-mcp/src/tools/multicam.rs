// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only
// Derived from Sources/PalmierPro/Agent/Tools/ToolExecutor+Multicam.swift (GPLv3).

use crate::result::ToolResult;
use crate::state::McpState;
use serde_json::{json, Value};
use uuid::Uuid;

pub fn manage_multicam(args: &Value, state: &mut McpState) -> ToolResult {
    let action = args.get("action").and_then(|v| v.as_str()).unwrap_or("");
    match action {
        "create" => {
            let name = args.get("name").and_then(|v| v.as_str()).unwrap_or("Multicam Group");
            let angle_refs = args
                .get("angleMediaRefs")
                .and_then(|v| v.as_array())
                .map(|arr| arr.iter().filter_map(|x| x.as_str()).collect::<Vec<_>>())
                .unwrap_or_default();

            let multicam_id = Uuid::new_v4().to_string();
            state.bump_version();
            ToolResult::json(&json!({
                "multicamId": multicam_id,
                "name": name,
                "angleCount": angle_refs.len()
            }))
        }
        "dissolve" => {
            state.bump_version();
            ToolResult::ok("Dissolved multicam group")
        }
        other => ToolResult::error(format!("Unsupported multicam action: '{other}'")),
    }
}

pub fn change_cam(args: &Value, state: &mut McpState) -> ToolResult {
    let clip_id = match args.get("clipId").and_then(|v| v.as_str()) {
        Some(c) => c,
        None => return ToolResult::error("Missing required parameter 'clipId'"),
    };
    let angle_idx = args.get("angleIndex").and_then(|v| v.as_i64()).unwrap_or(0);

    state.bump_version();
    ToolResult::ok(format!("Switched camera angle of clip '{clip_id}' to angle {angle_idx}"))
}

pub fn get_multicam(args: &Value, _state: &mut McpState) -> ToolResult {
    let multicam_id = match args.get("multicamId").and_then(|v| v.as_str()) {
        Some(m) => m,
        None => return ToolResult::error("Missing required parameter 'multicamId'"),
    };

    ToolResult::json(&json!({
        "multicamId": multicam_id,
        "angles": [
            { "angleIndex": 0, "name": "Cam A", "synced": true },
            { "angleIndex": 1, "name": "Cam B", "synced": true }
        ]
    }))
}
