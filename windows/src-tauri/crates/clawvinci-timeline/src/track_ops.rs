// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only
// Derived from Sources/PalmierPro/Editor/ViewModel/EditorViewModel+Tracks.swift
// and Sources/PalmierPro/Models/Timeline.swift (GPLv3).

use crate::error::TimelineError;
use clawvinci_model::clip_type::ClipType;
use clawvinci_model::timeline::{Timeline, Track};

/// Validation and normalization helper for track display names.
pub struct TrackName;

impl TrackName {
    pub const MAXIMUM_LENGTH: usize = 80;

    /// Normalizes a raw track name. Rejects control characters and names longer than 80 chars.
    pub fn normalized(raw_value: Option<&str>) -> Result<Option<String>, TimelineError> {
        let raw = match raw_value {
            Some(r) => r,
            None => return Ok(None),
        };
        let value = raw.trim();
        if raw.chars().any(|c| c.is_control() || c == '\n' || c == '\r')
            || value.chars().count() > Self::MAXIMUM_LENGTH
        {
            return Err(TimelineError::InvalidTrackName(raw.to_string()));
        }
        if value.is_empty() {
            Ok(None)
        } else {
            Ok(Some(value.to_string()))
        }
    }
}

/// Clamps `requested` index so visual tracks always sit strictly above audio tracks.
pub fn partitioned_insertion_index(
    timeline: &Timeline,
    track_type: ClipType,
    requested: usize,
) -> usize {
    let first_audio = timeline
        .tracks
        .iter()
        .position(|t| t.track_type == ClipType::Audio)
        .unwrap_or(timeline.tracks.len());
    let track_count = timeline.tracks.len();
    let bounded = requested.min(track_count);

    if track_type == ClipType::Audio {
        bounded.max(first_audio)
    } else {
        bounded.min(first_audio)
    }
}

/// Inserts a new track of `track_type` into `timeline`, clamping to maintain zone partitioning.
/// Returns the actual insertion index.
pub fn insert_track(
    timeline: &mut Timeline,
    requested_index: usize,
    track_type: ClipType,
) -> usize {
    let index = partitioned_insertion_index(timeline, track_type, requested_index);
    let track = Track::new(track_type);
    timeline.tracks.insert(index, track);
    index
}

/// Removes a track by ID from `timeline`, returning the removed track.
pub fn remove_track(timeline: &mut Timeline, track_id: &str) -> Result<Track, TimelineError> {
    let pos = timeline
        .tracks
        .iter()
        .position(|t| t.id == track_id)
        .ok_or_else(|| TimelineError::TrackIdNotFound(track_id.to_string()))?;
    Ok(timeline.tracks.remove(pos))
}

/// Reorders a track with `track_id` to `target_index`, clamped within its zone partition.
/// Returns the new index of the track.
pub fn reorder_track(
    timeline: &mut Timeline,
    track_id: &str,
    target_index: usize,
) -> Result<usize, TimelineError> {
    let from = timeline
        .tracks
        .iter()
        .position(|t| t.id == track_id)
        .ok_or_else(|| TimelineError::TrackIdNotFound(track_id.to_string()))?;

    if timeline.tracks.len() <= 1 {
        return Ok(from);
    }

    let is_audio = timeline.tracks[from].track_type == ClipType::Audio;
    let first_audio = timeline
        .tracks
        .iter()
        .position(|t| t.track_type == ClipType::Audio)
        .unwrap_or(timeline.tracks.len());

    let (lower, upper) = if is_audio {
        (first_audio, timeline.tracks.len() - 1)
    } else {
        (0, first_audio.saturating_sub(1))
    };

    let dest = target_index.clamp(lower, upper);
    if dest != from {
        let track = timeline.tracks.remove(from);
        timeline.tracks.insert(dest, track);
    }
    Ok(dest)
}

/// Updates the name of a track after normalization. Returns true if name changed.
pub fn set_track_name(
    timeline: &mut Timeline,
    track_id: &str,
    raw_name: Option<&str>,
) -> Result<bool, TimelineError> {
    let normalized = TrackName::normalized(raw_name)?;
    let track = timeline
        .tracks
        .iter_mut()
        .find(|t| t.id == track_id)
        .ok_or_else(|| TimelineError::TrackIdNotFound(track_id.to_string()))?;
    if track.name != normalized {
        track.name = normalized;
        Ok(true)
    } else {
        Ok(false)
    }
}

/// Sets track mute state.
pub fn set_track_muted(
    timeline: &mut Timeline,
    track_id: &str,
    muted: bool,
) -> Result<(), TimelineError> {
    let track = timeline
        .tracks
        .iter_mut()
        .find(|t| t.id == track_id)
        .ok_or_else(|| TimelineError::TrackIdNotFound(track_id.to_string()))?;
    track.muted = muted;
    Ok(())
}

/// Sets track hidden / visibility state.
pub fn set_track_hidden(
    timeline: &mut Timeline,
    track_id: &str,
    hidden: bool,
) -> Result<(), TimelineError> {
    let track = timeline
        .tracks
        .iter_mut()
        .find(|t| t.id == track_id)
        .ok_or_else(|| TimelineError::TrackIdNotFound(track_id.to_string()))?;
    track.hidden = hidden;
    Ok(())
}

/// Sets track sync-lock state.
pub fn set_track_sync_locked(
    timeline: &mut Timeline,
    track_id: &str,
    sync_locked: bool,
) -> Result<(), TimelineError> {
    let track = timeline
        .tracks
        .iter_mut()
        .find(|t| t.id == track_id)
        .ok_or_else(|| TimelineError::TrackIdNotFound(track_id.to_string()))?;
    track.sync_locked = sync_locked;
    Ok(())
}

/// Prunes empty unreferenced tracks from the tail of the timeline.
pub fn prune_empty_tracks(timeline: &mut Timeline) {
    let has_visual = timeline.tracks.iter().any(|t| t.track_type.is_visual() && !t.clips.is_empty());
    let has_audio = timeline.tracks.iter().any(|t| t.track_type == ClipType::Audio && !t.clips.is_empty());

    // Keep at least one visual track and one audio track if available
    let mut min_visual = if has_visual { 1 } else { 0 };
    let mut min_audio = if has_audio { 1 } else { 0 };

    timeline.tracks.retain(|t| {
        if !t.clips.is_empty() {
            true
        } else if t.track_type.is_visual() && min_visual > 0 {
            min_visual -= 1;
            true
        } else if t.track_type == ClipType::Audio && min_audio > 0 {
            min_audio -= 1;
            true
        } else {
            false
        }
    });
}
