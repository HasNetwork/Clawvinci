// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only

use thiserror::Error;

/// Error types for audio analysis and processing.
#[derive(Debug, Error)]
pub enum AudioError {
    #[error("No audio track found: {0}")]
    NoAudioTrack(String),

    #[error("Audio read or decode failed: {0}")]
    ReadFailed(String),

    #[error("Audio sync correlation failed: {0}")]
    SyncFailed(String),

    #[error("Invalid audio range: {0}")]
    InvalidRange(String),

    #[error("Analysis failed: {0}")]
    AnalysisFailed(String),

    #[error("Media error: {0}")]
    Media(#[from] clawvinci_media::error::MediaError),
}

pub type AudioResult<T> = Result<T, AudioError>;
