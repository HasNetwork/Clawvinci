// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only
// Derived from Sources/PalmierPro/Agent/Tools/ToolExecutor+Layout.swift (GPLv3).

use crate::result::ToolResult;
use crate::state::McpState;
use serde_json::{json, Value};

pub fn apply_layout(args: &Value, state: &mut McpState) -> ToolResult {
    let layout = match args.get("layout").and_then(|v| v.as_str()) {
        Some(l) => l,
        None => return ToolResult::error("Missing required parameter 'layout'"),
    };

    let slots = match args.get("slots").and_then(|v| v.as_array()) {
        Some(s) => s,
        None => return ToolResult::error("Missing required parameter 'slots'"),
    };

    let mut configured = 0;
    let res = state.editor.perform("Apply Layout", |tl| {
        for slot in slots {
            let clip_id = slot.get("clipId").and_then(|v| v.as_str()).unwrap_or("");
            let slot_idx = slot.get("slotIndex").and_then(|v| v.as_i64()).unwrap_or(0);

            for track in &mut tl.tracks {
                for clip in &mut track.clips {
                    if clip.id == clip_id {
                        match layout {
                            "pip" => {
                                if slot_idx == 1 {
                                    clip.transform.width = 0.35;
                                    clip.transform.height = 0.35;
                                    clip.transform.center_x = 0.75;
                                    clip.transform.center_y = 0.75;
                                }
                            }
                            "split2" => {
                                clip.transform.width = 0.5;
                                clip.transform.center_x = if slot_idx == 0 { 0.25 } else { 0.75 };
                            }
                            "grid4" => {
                                clip.transform.width = 0.5;
                                clip.transform.height = 0.5;
                                clip.transform.center_x = if slot_idx % 2 == 0 { 0.25 } else { 0.75 };
                                clip.transform.center_y = if slot_idx < 2 { 0.25 } else { 0.75 };
                            }
                            _ => {}
                        }
                        configured += 1;
                    }
                }
            }
        }
        Ok(())
    });

    match res {
        Ok(()) => {
            state.bump_version();
            ToolResult::json(&json!({
                "layout": layout,
                "configuredClips": configured
            }))
        }
        Err(e) => ToolResult::error(format!("Failed to apply layout: {e}")),
    }
}
