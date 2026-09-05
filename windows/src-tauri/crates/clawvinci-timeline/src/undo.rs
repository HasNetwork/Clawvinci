// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only
// Derived from Sources/PalmierPro/Editor/EditorUndo.swift (GPLv3).

use crate::error::TimelineError;
use clawvinci_model::timeline::Timeline;
use serde::{Deserialize, Serialize};

/// A single atomic undoable entry in the undo/redo history.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UndoEntry {
    pub action_name: String,
    pub undo_state: Timeline,
    pub redo_state: Timeline,
}

/// An in-memory, transaction-grouped undo/redo history stack for timeline mutations.
#[derive(Debug, Clone)]
pub struct UndoStack {
    undo_entries: Vec<UndoEntry>,
    redo_entries: Vec<UndoEntry>,
    max_depth: usize,
    registration_enabled: bool,
    nesting_level: usize,
    pending_transaction: Option<(String, Timeline)>,
}

impl Default for UndoStack {
    fn default() -> Self {
        Self::new(100)
    }
}

impl UndoStack {
    pub fn new(max_depth: usize) -> Self {
        Self {
            undo_entries: Vec::new(),
            redo_entries: Vec::new(),
            max_depth: max_depth.max(1),
            registration_enabled: true,
            nesting_level: 0,
            pending_transaction: None,
        }
    }

    pub fn is_registration_enabled(&self) -> bool {
        self.registration_enabled
    }

    pub fn set_registration_enabled(&mut self, enabled: bool) {
        self.registration_enabled = enabled;
    }

    pub fn can_undo(&self) -> bool {
        !self.undo_entries.is_empty() && self.nesting_level == 0
    }

    pub fn can_redo(&self) -> bool {
        !self.redo_entries.is_empty() && self.nesting_level == 0
    }

    pub fn undo_action_name(&self) -> Option<&str> {
        self.undo_entries.last().map(|e| e.action_name.as_str())
    }

    pub fn redo_action_name(&self) -> Option<&str> {
        self.redo_entries.last().map(|e| e.action_name.as_str())
    }

    pub fn undo_count(&self) -> usize {
        self.undo_entries.len()
    }

    pub fn redo_count(&self) -> usize {
        self.redo_entries.len()
    }

    pub fn clear(&mut self) {
        self.undo_entries.clear();
        self.redo_entries.clear();
        self.nesting_level = 0;
        self.pending_transaction = None;
    }

    /// Begins an undo transaction. Nested calls within an existing transaction
    /// increment nesting and coalesce into the outer action.
    pub fn begin_transaction(&mut self, action_name: &str, current_state: &Timeline) {
        if self.nesting_level == 0 {
            if self.registration_enabled {
                self.pending_transaction = Some((action_name.to_string(), current_state.clone()));
            } else {
                self.pending_transaction = None;
            }
        }
        self.nesting_level += 1;
    }

    /// Commits an undo transaction. When the outermost transaction completes,
    /// checks whether state changed. If unchanged, no entry is recorded.
    pub fn commit_transaction(&mut self, final_state: &Timeline) -> bool {
        if self.nesting_level == 0 {
            return false;
        }
        self.nesting_level -= 1;
        if self.nesting_level == 0 {
            if let Some((action_name, undo_state)) = self.pending_transaction.take() {
                if undo_state != *final_state && self.registration_enabled {
                    self.undo_entries.push(UndoEntry {
                        action_name,
                        undo_state,
                        redo_state: final_state.clone(),
                    });
                    if self.undo_entries.len() > self.max_depth {
                        self.undo_entries.remove(0);
                    }
                    self.redo_entries.clear();
                    return true;
                }
            }
        }
        false
    }

    /// Cancels the active transaction, discarding any recorded undo snapshot.
    pub fn cancel_transaction(&mut self) {
        if self.nesting_level > 0 {
            self.nesting_level -= 1;
            if self.nesting_level == 0 {
                self.pending_transaction = None;
            }
        }
    }

    /// Executes `work` with undo registration temporarily disabled.
    pub fn without_undo<F, R>(&mut self, work: F) -> R
    where
        F: FnOnce(&mut Self) -> R,
    {
        let prev = self.registration_enabled;
        self.registration_enabled = false;
        let res = work(self);
        self.registration_enabled = prev;
        res
    }

    /// Reverts the latest mutation on `timeline`. Returns the action name reverted.
    pub fn undo(&mut self, timeline: &mut Timeline) -> Result<String, TimelineError> {
        if self.nesting_level > 0 {
            return Err(TimelineError::TransactionActive);
        }
        let entry = self.undo_entries.pop().ok_or(TimelineError::NothingToUndo)?;
        *timeline = entry.undo_state.clone();
        let name = entry.action_name.clone();
        self.redo_entries.push(entry);
        Ok(name)
    }

    /// Re-applies the next undone mutation on `timeline`. Returns the action name re-applied.
    pub fn redo(&mut self, timeline: &mut Timeline) -> Result<String, TimelineError> {
        if self.nesting_level > 0 {
            return Err(TimelineError::TransactionActive);
        }
        let entry = self.redo_entries.pop().ok_or(TimelineError::NothingToRedo)?;
        *timeline = entry.redo_state.clone();
        let name = entry.action_name.clone();
        self.undo_entries.push(entry);
        Ok(name)
    }
}
