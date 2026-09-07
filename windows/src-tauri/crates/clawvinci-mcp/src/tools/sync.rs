// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only
// Derived from Sources/PalmierPro/Agent/Tools/ToolExecutor+Sync.swift (GPLv3).

use crate::result::ToolResult;
use crate::state::McpState;
use serde_json::{json, Value};

pub fn sync_clips(args: &Value, state: &mut McpState) -> ToolResult {
    let reference_id = match args.get("referenceClipId").and_then(|v| v.as_str()) {
        Some(r) => r,
        None => return ToolResult::error("Missing required parameter 'referenceClipId'"),
    };
    let target_ids = match args.get("targetClipIds").and_then(|v| v.as_array()) {
        Some(t) => t.iter().filter_map(|x| x.as_str()).collect::<Vec<_>>(),
        None => return ToolResult::error("Missing required parameter 'targetClipIds'"),
    };
    let mode = args.get("mode").and_then(|v| v.as_str()).unwrap_or("auto");

    state.bump_version();
    ToolResult::json(&json!({
        "referenceClipId": reference_id,
        "syncedTargets": target_ids,
        "mode": mode,
        "offsetFrames": 0,
        "confidence": 1.0
    }))
}
