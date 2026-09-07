// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only
// Derived from Sources/PalmierPro/Agent/Tools/ToolExecutor+Projects.swift (GPLv3).

use crate::result::ToolResult;
use crate::state::McpState;
use clawvinci_model::timeline::Timeline;
use serde_json::{json, Value};
use uuid::Uuid;

pub fn manage_project(args: &Value, state: &mut McpState) -> ToolResult {
    let action = args.get("action").and_then(|v| v.as_str()).unwrap_or("");
    match action {
        "list" => {
            let active_id = state.active_timeline_id.clone();
            let mut timelines = vec![json!({
                "timelineId": active_id,
                "name": state.editor.timeline().name.clone(),
                "active": true
            })];
            for (id, tl) in &state.child_timelines {
                timelines.push(json!({
                    "timelineId": id,
                    "name": tl.name.clone(),
                    "active": false
                }));
            }
            ToolResult::json(&json!({
                "projectName": state.active_project_name,
                "projectPath": state.active_project_path,
                "timelines": timelines
            }))
        }
        "open" => {
            let name = args.get("name").and_then(|v| v.as_str());
            let path = args.get("path").and_then(|v| v.as_str());
            if let Some(n) = name {
                state.active_project_name = n.to_string();
            }
            if let Some(p) = path {
                state.active_project_path = Some(std::path::PathBuf::from(p));
            }
            ToolResult::ok(format!("Opened project: {}", state.active_project_name))
        }
        "create" => {
            let name = args
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("Untitled Project");
            state.active_project_name = name.to_string();
            if let Some(fps) = args.get("fps").and_then(|v| v.as_i64()) {
                let _ = state.editor.perform("Change Project FPS", |tl| {
                    tl.fps = fps as i32;
                    Ok(())
                });
            }
            ToolResult::ok(format!("Created project '{name}'"))
        }
        "close" => {
            ToolResult::ok(format!("Saved and closed project '{}'", state.active_project_name))
        }
        other => ToolResult::error(format!("Unsupported project action: '{other}'")),
    }
}

pub fn create_timeline(args: &Value, state: &mut McpState) -> ToolResult {
    let name = args
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("New Timeline")
        .to_string();
    let from_id = args.get("from").and_then(|v| v.as_str());

    let new_timeline = if from_id.is_some() {
        let mut cloned = state.editor.timeline().clone();
        cloned.id = Uuid::new_v4().to_string();
        cloned.name = name.clone();
        cloned
    } else {
        let mut empty = Timeline::default();
        empty.id = Uuid::new_v4().to_string();
        empty.name = name.clone();
        empty
    };

    let new_id = new_timeline.id.clone();
    state.child_timelines.insert(new_id.clone(), new_timeline);
    state.bump_version();

    ToolResult::json(&json!({
        "timelineId": new_id,
        "name": name,
        "created": true
    }))
}

pub fn set_active_timeline(args: &Value, state: &mut McpState) -> ToolResult {
    let target_id = match args.get("timelineId").and_then(|v| v.as_str()) {
        Some(id) => id,
        None => return ToolResult::error("Missing required parameter 'timelineId'"),
    };

    if state.active_timeline_id == target_id {
        return ToolResult::ok(format!("Timeline '{target_id}' is already active"));
    }

    if let Some(new_tl) = state.child_timelines.remove(target_id) {
        let old_active = state.editor.timeline().clone();
        let old_id = state.active_timeline_id.clone();
        state.child_timelines.insert(old_id, old_active);
        state.active_timeline_id = target_id.to_string();
        state.editor = clawvinci_timeline::editor::TimelineEditor::new(new_tl);
        state.bump_version();
        ToolResult::ok(format!("Switched active timeline to '{target_id}'"))
    } else {
        ToolResult::error(format!("Timeline with id '{target_id}' not found"))
    }
}
