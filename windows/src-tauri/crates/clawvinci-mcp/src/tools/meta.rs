// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only
// Derived from Sources/PalmierPro/Agent/Tools/ToolExecutor+Skills.swift and Feedback.swift (GPLv3).

use crate::result::ToolResult;
use crate::state::{McpSkill, McpState};
use serde_json::{json, Value};
use uuid::Uuid;

pub fn send_feedback(args: &Value, _state: &mut McpState) -> ToolResult {
    let message = match args.get("message").and_then(|v| v.as_str()) {
        Some(m) => m,
        None => return ToolResult::error("Missing required parameter 'message'"),
    };
    let category = args.get("category").and_then(|v| v.as_str()).unwrap_or("general");

    ToolResult::json(&json!({
        "received": true,
        "category": category,
        "length": message.len()
    }))
}

pub fn read_skill(args: &Value, state: &mut McpState) -> ToolResult {
    let skill_id = match args.get("id").and_then(|v| v.as_str()) {
        Some(id) => id,
        None => return ToolResult::error("Missing required parameter 'id'"),
    };

    if let Some(skill) = state.skills.iter().find(|s| s.id == skill_id) {
        ToolResult::json(&json!({
            "id": skill.id,
            "name": skill.name,
            "description": skill.description,
            "instructions": skill.instructions
        }))
    } else {
        ToolResult::error(format!("Skill '{skill_id}' not found"))
    }
}

pub fn manage_skills(args: &Value, state: &mut McpState) -> ToolResult {
    let action = args.get("action").and_then(|v| v.as_str()).unwrap_or("");
    match action {
        "create" => {
            let name = match args.get("name").and_then(|v| v.as_str()) {
                Some(n) => n,
                None => return ToolResult::error("Missing required parameter 'name'"),
            };
            let instructions = match args.get("instructions").and_then(|v| v.as_str()) {
                Some(i) => i,
                None => return ToolResult::error("Missing required parameter 'instructions'"),
            };
            let description = args.get("description").and_then(|v| v.as_str()).unwrap_or("");

            let id = Uuid::new_v4().to_string();
            state.skills.push(McpSkill {
                id: id.clone(),
                name: name.to_string(),
                description: description.to_string(),
                instructions: instructions.to_string(),
            });
            state.bump_version();

            ToolResult::json(&json!({
                "id": id,
                "name": name,
                "created": true
            }))
        }
        "update" => {
            let skill_id = match args.get("id").and_then(|v| v.as_str()) {
                Some(id) => id,
                None => return ToolResult::error("Missing required parameter 'id'"),
            };

            if let Some(skill) = state.skills.iter_mut().find(|s| s.id == skill_id) {
                if let Some(n) = args.get("name").and_then(|v| v.as_str()) {
                    skill.name = n.to_string();
                }
                if let Some(d) = args.get("description").and_then(|v| v.as_str()) {
                    skill.description = d.to_string();
                }
                if let Some(i) = args.get("instructions").and_then(|v| v.as_str()) {
                    skill.instructions = i.to_string();
                }
                state.bump_version();
                ToolResult::ok(format!("Updated skill '{skill_id}'"))
            } else {
                ToolResult::error(format!("Skill '{skill_id}' not found"))
            }
        }
        "remove" => {
            let skill_id = match args.get("id").and_then(|v| v.as_str()) {
                Some(id) => id,
                None => return ToolResult::error("Missing required parameter 'id'"),
            };

            let before = state.skills.len();
            state.skills.retain(|s| s.id != skill_id);
            if state.skills.len() < before {
                state.bump_version();
                ToolResult::ok(format!("Removed skill '{skill_id}'"))
            } else {
                ToolResult::error(format!("Skill '{skill_id}' not found"))
            }
        }
        other => ToolResult::error(format!("Unsupported skill action: '{other}'")),
    }
}
