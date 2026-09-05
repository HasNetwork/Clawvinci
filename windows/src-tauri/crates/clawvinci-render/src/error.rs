// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only
// Derived from Sources/PalmierPro/Preview/CompositionBuilder.swift (GPLv3).

use thiserror::Error;

#[derive(Error, Debug)]
pub enum RenderError {
    #[error("Invalid timeline: {0}")]
    InvalidTimeline(String),

    #[error("Frame index {frame} is out of bounds (total frames: {total})")]
    FrameOutOfBounds { frame: usize, total: usize },

    #[error("Source frame missing for layer: {0}")]
    SourceMissing(String),

    #[error("Frame composition failed: {0}")]
    CompositeFailed(String),

    #[error("Playback engine error: {0}")]
    PlaybackError(String),

    #[error("Media error: {0}")]
    Media(#[from] clawvinci_media::error::MediaError),

    #[error("Serialization error: {0}")]
    Json(#[from] serde_json::Error),
}

pub type RenderResult<T> = Result<T, RenderError>;
