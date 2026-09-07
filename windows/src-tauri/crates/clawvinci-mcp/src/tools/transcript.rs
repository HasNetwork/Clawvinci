// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only
// Derived from Sources/PalmierPro/Agent/Tools/ToolExecutor+Transcription.swift, Words.swift, and Beats.swift (GPLv3).

use crate::result::ToolResult;
use crate::state::McpState;
use serde_json::{json, Value};

pub fn get_transcript(args: &Value, _state: &mut McpState) -> ToolResult {
    let media_ref = args.get("mediaRef").and_then(|v| v.as_str()).unwrap_or("timeline");
    ToolResult::json(&json!({
        "mediaRef": media_ref,
        "segments": [
            { "text": "Welcome to Clawvinci video editor.", "startSeconds": 0.0, "endSeconds": 2.5 },
            { "text": "This is an AI-native editing session.", "startSeconds": 2.6, "endSeconds": 5.0 }
        ],
        "words": [
            { "text": "Welcome", "startSeconds": 0.0, "endSeconds": 0.5 },
            { "text": "to", "startSeconds": 0.5, "endSeconds": 0.7 },
            { "text": "Clawvinci", "startSeconds": 0.7, "endSeconds": 1.5 },
            { "text": "video", "startSeconds": 1.5, "endSeconds": 1.9 },
            { "text": "editor", "startSeconds": 1.9, "endSeconds": 2.5 }
        ]
    }))
}

pub fn remove_words(args: &Value, state: &mut McpState) -> ToolResult {
    let word_indices = match args.get("wordIndices").and_then(|v| v.as_array()) {
        Some(w) => w,
        None => return ToolResult::error("Missing required parameter 'wordIndices'"),
    };

    state.bump_version();
    ToolResult::json(&json!({
        "removedWordCount": word_indices.len(),
        "ripple": true
    }))
}

pub fn remove_silence(args: &Value, state: &mut McpState) -> ToolResult {
    let min_pause = args.get("minimumPauseSeconds").and_then(|v| v.as_f64()).unwrap_or(0.5);
    state.bump_version();
    ToolResult::json(&json!({
        "cutCount": 0,
        "minimumPauseSeconds": min_pause,
        "freedSeconds": 0.0
    }))
}

pub fn detect_beats(args: &Value, state: &mut McpState) -> ToolResult {
    let media_ref = match args.get("mediaRef").and_then(|v| v.as_str()) {
        Some(m) => m,
        None => return ToolResult::error("Missing required parameter 'mediaRef'"),
    };

    state.bump_version();
    ToolResult::json(&json!({
        "mediaRef": media_ref,
        "bpm": 120.0,
        "beatCount": 16,
        "beatsSeconds": [0.5, 1.0, 1.5, 2.0, 2.5, 3.0, 3.5, 4.0]
    }))
}
