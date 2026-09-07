// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only
// Derived from Sources/PalmierPro/Export/ExportService.swift (GPLv3).

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ExportError {
    #[error("An export to {0} is already waiting or in progress.")]
    DestinationInUse(String),

    #[error("Export preset not supported on this system: {0}")]
    UnsupportedPreset(String),

    #[error("Invalid export format: {0}")]
    InvalidFormat(String),

    #[error("Couldn't encode the timeline as XML: {0}")]
    XmlEncodingFailed(String),

    #[error("Media error during export: {0}")]
    MediaError(#[from] clawvinci_media::error::MediaError),

    #[error("Render error during export: {0}")]
    RenderError(#[from] clawvinci_render::error::RenderError),

    #[error("I/O error during export: {0}")]
    IoError(#[from] std::io::Error),

    #[error("JSON error during export: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Export was cancelled")]
    Cancelled,

    #[error("Export failed: {0}")]
    Failed(String),
}

pub type ExportResult<T> = Result<T, ExportError>;
