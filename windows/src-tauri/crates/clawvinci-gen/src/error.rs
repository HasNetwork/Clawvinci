// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only

use thiserror::Error;

#[derive(Debug, Error)]
pub enum GenError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Media engine error: {0}")]
    Media(#[from] clawvinci_media::MediaError),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Backend service error: {0}")]
    Backend(String),

    #[error("Model '{0}' not found in catalog")]
    InvalidModel(String),

    #[error("Invalid generation input: {0}")]
    InvalidInput(String),

    #[error("Preprocessing failed: {0}")]
    PreprocessingFailed(String),

    #[error("Generation job failed: {0}")]
    JobFailed(String),

    #[error("Operation was cancelled")]
    Cancelled,
}

pub type GenResult<T> = Result<T, GenError>;
