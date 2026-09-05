// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only
// Derived from Sources/PalmierPro/Models/MediaResolver.swift (GPLv3).

use thiserror::Error;

#[derive(Error, Debug)]
pub enum MediaError {
    #[error("FFmpeg/FFprobe binary not found: {0}")]
    FfmpegNotFound(String),

    #[error("Media probe failed: {0}")]
    ProbeFailed(String),

    #[error("Frame decode failed: {0}")]
    DecodeFailed(String),

    #[error("Media encode failed: {0}")]
    EncodeFailed(String),

    #[error("Waveform extraction failed: {0}")]
    WaveformFailed(String),

    #[error("Invalid media input: {0}")]
    InvalidInput(String),

    #[error("Media operation was cancelled")]
    Cancelled,

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),
}

pub type MediaResult<T> = Result<T, MediaError>;
