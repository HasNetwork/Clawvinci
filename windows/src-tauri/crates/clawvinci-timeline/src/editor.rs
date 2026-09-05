// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only
// Derived from Sources/PalmierPro/Editor/ViewModel/EditorViewModel.swift (GPLv3).

use crate::clip_ops;
use crate::error::TimelineError;
use crate::overwrite;
use crate::ripple::{self, TrimEdge};
use crate::track_ops;
use crate::undo::UndoStack;
use clawvinci_model::clip_type::ClipType;
use clawvinci_model::timeline::{Clip, Timeline, Track};
use std::collections::HashSet;

/// The central coordinator managing timeline mutation operations and shared undo history.
/// UI handlers (Phase 6) and MCP tools (Phase 8) execute all modifications through this struct.
#[derive(Debug, Clone)]
pub struct TimelineEditor {
    timeline: Timeline,
    undo_stack: UndoStack,
}

impl TimelineEditor {
    pub fn new(timeline: Timeline) -> Self {
        Self {
            timeline,
            undo_stack: UndoStack::default(),
        }
    }

    pub fn with_undo_capacity(timeline: Timeline, capacity: usize) -> Self {
        Self {
            timeline,
            undo_stack: UndoStack::new(capacity),
        }
    }

    pub fn timeline(&self) -> &Timeline {
        &self.timeline
    }

    pub fn timeline_mut(&mut self) -> &mut Timeline {
        &mut self.timeline
    }

    pub fn undo_stack(&self) -> &UndoStack {
        &self.undo_stack
    }

    pub fn undo_stack_mut(&mut self) -> &mut UndoStack {
        &mut self.undo_stack
    }

    pub fn can_undo(&self) -> bool {
        self.undo_stack.can_undo()
    }

    pub fn can_redo(&self) -> bool {
        self.undo_stack.can_redo()
    }

    /// Executes closure `work` with undo recording temporarily disabled.
    pub fn without_undo<F, R>(&mut self, work: F) -> R
    where
        F: FnOnce(&mut Self) -> R,
    {
        let prev = self.undo_stack.is_registration_enabled();
        self.undo_stack.set_registration_enabled(false);
        let res = work(self);
        self.undo_stack.set_registration_enabled(prev);
        res
    }

    pub fn undo_action_name(&self) -> Option<&str> {
        self.undo_stack.undo_action_name()
    }

    pub fn redo_action_name(&self) -> Option<&str> {
        self.undo_stack.redo_action_name()
    }

    /// Reverts the latest mutation on the timeline.
    pub fn undo(&mut self) -> Result<String, TimelineError> {
        self.undo_stack.undo(&mut self.timeline)
    }

    /// Re-applies the next undone mutation on the timeline.
    pub fn redo(&mut self) -> Result<String, TimelineError> {
        self.undo_stack.redo(&mut self.timeline)
    }

    /// Executes an atomic mutation enclosed within an undo transaction.
    /// Nested mutations coalesce into the outermost transaction.
    /// If an error occurs, the transaction is cancelled and timeline is rolled back.
    pub fn perform<F, R>(&mut self, action_name: &str, mutate: F) -> Result<R, TimelineError>
    where
        F: FnOnce(&mut Timeline) -> Result<R, TimelineError>,
    {
        let before_snapshot = self.timeline.clone();
        self.undo_stack.begin_transaction(action_name, &self.timeline);
        match mutate(&mut self.timeline) {
            Ok(val) => {
                self.undo_stack.commit_transaction(&self.timeline);
                Ok(val)
            }
            Err(err) => {
                self.timeline = before_snapshot;
                self.undo_stack.cancel_transaction();
                Err(err)
            }
        }
    }

    // MARK: - Clip operations

    pub fn split_clip(&mut self, clip_id: &str, at_frame: i64) -> Result<Vec<String>, TimelineError> {
        self.perform("Split Clip", |tl| clip_ops::split_clip(tl, clip_id, at_frame))
    }

    pub fn trim_clip(
        &mut self,
        clip_id: &str,
        edge: TrimEdge,
        delta: i64,
    ) -> Result<(), TimelineError> {
        self.perform("Trim Clip", |tl| clip_ops::trim_clip(tl, clip_id, edge, delta))
    }

    pub fn slip_clip(&mut self, clip_id: &str, delta: i64) -> Result<(), TimelineError> {
        self.perform("Slip Clip", |tl| clip_ops::slip_clip(tl, clip_id, delta))
    }

    pub fn move_clips(&mut self, moves: &[(String, usize, i64)]) -> Result<(), TimelineError> {
        let action = if moves.len() == 1 { "Move Clip" } else { "Move Clips" };
        self.perform(action, |tl| clip_ops::move_clips(tl, moves))
    }

    pub fn set_clip_speed(
        &mut self,
        clip_id: &str,
        new_speed: f64,
        ripple: bool,
    ) -> Result<(), TimelineError> {
        self.perform("Change Speed", |tl| clip_ops::set_clip_speed(tl, clip_id, new_speed, ripple))
    }

    pub fn remove_clips(&mut self, clip_ids: &HashSet<String>) -> Result<usize, TimelineError> {
        let action = if clip_ids.len() == 1 { "Remove Clip" } else { "Remove Clips" };
        self.perform(action, |tl| {
            let removed = clip_ops::remove_clips(tl, clip_ids);
            if removed == 0 {
                Err(TimelineError::NoOp)
            } else {
                Ok(removed)
            }
        })
    }

    // MARK: - Ripple operations

    pub fn ripple_delete(&mut self, clip_ids: &HashSet<String>) -> Result<usize, TimelineError> {
        self.perform("Ripple Delete", |tl| ripple::ripple_delete_clips(tl, clip_ids))
    }

    pub fn ripple_insert(&mut self, track_index: usize, clip: Clip) -> Result<(), TimelineError> {
        self.perform("Ripple Insert", |tl| ripple::ripple_insert_clip(tl, track_index, clip))
    }

    // MARK: - Overwrite operations

    pub fn overwrite_clip(&mut self, track_index: usize, clip: Clip) -> Result<(), TimelineError> {
        self.perform("Overwrite Clip", |tl| overwrite::overwrite_clip(tl, track_index, clip))
    }

    // MARK: - Track operations

    pub fn insert_track(&mut self, requested_index: usize, track_type: ClipType) -> Result<usize, TimelineError> {
        self.perform("Add Track", |tl| Ok(track_ops::insert_track(tl, requested_index, track_type)))
    }

    pub fn remove_track(&mut self, track_id: &str) -> Result<Track, TimelineError> {
        self.perform("Remove Track", |tl| track_ops::remove_track(tl, track_id))
    }

    pub fn reorder_track(&mut self, track_id: &str, target_index: usize) -> Result<usize, TimelineError> {
        self.perform("Reorder Track", |tl| track_ops::reorder_track(tl, track_id, target_index))
    }

    pub fn set_track_name(&mut self, track_id: &str, raw_name: Option<&str>) -> Result<bool, TimelineError> {
        self.perform("Rename Track", |tl| track_ops::set_track_name(tl, track_id, raw_name))
    }

    pub fn set_track_muted(&mut self, track_id: &str, muted: bool) -> Result<(), TimelineError> {
        let action = if muted { "Mute Track" } else { "Unmute Track" };
        self.perform(action, |tl| track_ops::set_track_muted(tl, track_id, muted))
    }

    pub fn set_track_hidden(&mut self, track_id: &str, hidden: bool) -> Result<(), TimelineError> {
        let action = if hidden { "Hide Track" } else { "Show Track" };
        self.perform(action, |tl| track_ops::set_track_hidden(tl, track_id, hidden))
    }

    pub fn set_track_sync_locked(&mut self, track_id: &str, sync_locked: bool) -> Result<(), TimelineError> {
        self.perform("Toggle Sync Lock", |tl| track_ops::set_track_sync_locked(tl, track_id, sync_locked))
    }
}
