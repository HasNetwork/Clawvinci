// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only
//! Clawvinci Timeline Editing Core (Phase 3): Clip/track mutation ops, ripple/overwrite, shared undo history.

pub mod clip_ops;
pub mod editor;
pub mod error;
pub mod overwrite;
pub mod ripple;
pub mod track_ops;
pub mod undo;

pub mod prelude {
    pub use crate::clip_ops::{find_clip, move_clips, remove_clips, slip_clip, split_clip, trim_clip};
    pub use crate::editor::TimelineEditor;
    pub use crate::error::TimelineError;
    pub use crate::overwrite::{clear_region, overwrite_clip, OverwriteAction, OverwriteEngine};
    pub use crate::ripple::{
        ripple_delete_clips, ripple_insert_clip, ClipShift, FrameRange, RippleEngine, TrimEdge,
    };
    pub use crate::track_ops::{insert_track, remove_track, reorder_track, TrackName};
    pub use crate::undo::{UndoEntry, UndoStack};
}
