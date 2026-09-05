// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only

use clawvinci_model::clip_type::ClipType;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TimelineError {
    #[error("Track at index {0} not found")]
    TrackNotFound(usize),

    #[error("Track with ID '{0}' not found")]
    TrackIdNotFound(String),

    #[error("Clip with ID '{0}' not found")]
    ClipNotFound(String),

    #[error("Invalid frame: {0}")]
    InvalidFrame(i64),

    #[error("Invalid duration: {0}")]
    InvalidDuration(i64),

    #[error("Invalid playback speed: {0}")]
    InvalidSpeed(f64),

    #[error("Track type {track_type:?} is incompatible with clip type {clip_type:?}")]
    TrackIncompatible {
        track_type: ClipType,
        clip_type: ClipType,
    },

    #[error("Invalid track name: '{0}'")]
    InvalidTrackName(String),

    #[error("Cannot split clip at frame {at_frame}; must be strictly between {start} and {end}")]
    CannotSplitAtBoundary {
        at_frame: i64,
        start: i64,
        end: i64,
    },

    #[error("Cannot retime multicam clip '{0}'")]
    MulticamRetimeRefused(String),

    #[error("Operation produced no change to timeline")]
    NoOp,

    #[error("Nothing to undo")]
    NothingToUndo,

    #[error("Nothing to redo")]
    NothingToRedo,

    #[error("An undo transaction is already active")]
    TransactionActive,

    #[error("{0}")]
    Custom(String),
}
