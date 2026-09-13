// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only
// Derived from Sources/PalmierPro/Search/ and Sources/PalmierPro/Transcription/ (GPLv3).

use thiserror::Error;

#[derive(Debug, Error)]
pub enum SearchError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Corrupt embedding store: {0}")]
    CorruptStore(String),

    #[error("Search model not ready: {0}")]
    ModelNotReady(String),

    #[error("Model download failed: {0}")]
    DownloadFailed(String),

    #[error("Visual analysis failed: {0}")]
    AnalysisFailed(String),

    #[error("Cloud transcription error: {0}")]
    CloudTranscriptionFailed(String),

    #[error("Local transcription failed: {0}")]
    LocalTranscriptionFailed(String),

    #[error("Invalid range: {0}")]
    InvalidRange(String),

    #[error("Unsupported media: {0}")]
    UnsupportedMedia(String),
}

pub type SearchResult<T> = Result<T, SearchError>;
