// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only
// Derived from Sources/PalmierPro/Agent/Tools/ToolExecutor.swift (GPLv3).

use clawvinci_export::queue::ExportQueue;
use clawvinci_model::timeline::Timeline;
use clawvinci_timeline::editor::TimelineEditor;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpMediaItem {
    pub id: String,
    pub name: String,
    pub path: String,
    pub media_type: String,
    pub duration_seconds: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fps: Option<f64>,
    #[serde(default)]
    pub has_audio: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub folder: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generation_prompt: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generation_status: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpMarker {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    pub color: String,
    pub start_frame: i64,
    pub duration_frames: i64,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpSkill {
    pub id: String,
    pub name: String,
    pub description: String,
    pub instructions: String,
}

/// Authoritative mutable domain state shared across the MCP HTTP server,
/// in-app agent chat, and the Tauri desktop shell.
pub struct McpState {
    pub editor: TimelineEditor,
    pub media_items: Vec<McpMediaItem>,
    pub export_queue: ExportQueue,
    pub markers: Vec<McpMarker>,
    pub active_project_path: Option<PathBuf>,
    pub active_project_name: String,
    pub active_timeline_id: String,
    pub child_timelines: HashMap<String, Timeline>,
    pub skills: Vec<McpSkill>,
    pub version: u64,
}

impl std::fmt::Debug for McpState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("McpState")
            .field("media_items", &self.media_items)
            .field("markers", &self.markers)
            .field("active_project_path", &self.active_project_path)
            .field("active_project_name", &self.active_project_name)
            .field("active_timeline_id", &self.active_timeline_id)
            .field("child_timelines_count", &self.child_timelines.len())
            .field("skills_count", &self.skills.len())
            .field("version", &self.version)
            .finish()
    }
}

impl McpState {
    pub fn new(
        timeline: Timeline,
        media_items: Vec<McpMediaItem>,
        export_queue: ExportQueue,
    ) -> Self {
        let active_timeline_id = timeline.id.clone();
        let editor = TimelineEditor::new(timeline);
        Self {
            editor,
            media_items,
            export_queue,
            markers: Vec::new(),
            active_project_path: None,
            active_project_name: "Untitled Project".to_string(),
            active_timeline_id,
            child_timelines: HashMap::new(),
            skills: Vec::new(),
            version: 0,
        }
    }

    pub fn bump_version(&mut self) {
        self.version = self.version.wrapping_add(1);
    }
}

pub type SharedMcpState = Arc<Mutex<McpState>>;
