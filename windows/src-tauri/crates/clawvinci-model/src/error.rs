// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only
//! Strongly-typed errors for domain models.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ModelError {
    #[error("project has no timelines")]
    EmptyTimelines,

    #[error("JSON decode error: {0}")]
    DecodeError(#[from] serde_json::Error),

    #[error("invalid track name: cannot contain control characters or exceed 80 characters")]
    InvalidTrackName,

    #[error("invalid timeline marker name: cannot be empty or exceed 120 characters")]
    InvalidMarkerName,

    #[error("invalid timeline marker comment: cannot exceed 4000 characters")]
    InvalidMarkerComment,

    #[error("invalid timeline marker range: duration cannot be negative")]
    InvalidMarkerRange,
}
