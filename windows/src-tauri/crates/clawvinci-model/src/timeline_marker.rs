// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only
// Derived from Sources/PalmierPro/Models/TimelineMarker.swift (GPLv3).

use crate::text_style::Rgba;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum MarkerStatus {
    #[default]
    Open,
    Review,
    Resolved,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimelineMarker {
    #[serde(default = "default_uuid")]
    pub id: String,
    pub name: String,
    pub start_frame: i64,
    #[serde(default)]
    pub duration_frames: i64,
    #[serde(default = "default_marker_color")]
    pub color: Rgba,
    #[serde(default)]
    pub comment: String,
    #[serde(default)]
    pub status: MarkerStatus,
}

fn default_uuid() -> String {
    Uuid::new_v4().to_string()
}

fn default_marker_color() -> Rgba {
    Rgba::new(0.0, 0.478, 1.0, 1.0)
}

impl TimelineMarker {
    pub const MAXIMUM_NAME_LENGTH: usize = 120;
    pub const MAXIMUM_COMMENT_LENGTH: usize = 4_000;

    pub fn new(name: impl Into<String>, start_frame: i64) -> Self {
        Self {
            id: default_uuid(),
            name: name.into(),
            start_frame,
            duration_frames: 0,
            color: default_marker_color(),
            comment: String::new(),
            status: MarkerStatus::Open,
        }
    }

    pub fn end_frame(&self) -> i64 {
        self.start_frame + self.duration_frames
    }

    pub fn is_range(&self) -> bool {
        self.duration_frames > 0
    }

    pub fn intersects(&self, start: i64, end: i64) -> bool {
        if self.is_range() {
            self.start_frame < end && self.end_frame() > start
        } else {
            self.start_frame >= start && self.start_frame < end
        }
    }

    pub fn rescale_frames(&mut self, scale: f64) {
        let scaled_end = (self.end_frame() as f64 * scale).round() as i64;
        self.start_frame = 0.max((self.start_frame as f64 * scale).round() as i64);
        self.duration_frames = if self.is_range() {
            1.max(scaled_end - self.start_frame)
        } else {
            0
        };
    }
}
