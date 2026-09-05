// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only
// Derived from Sources/PalmierPro/Editor/RippleEngine.swift and
// Sources/PalmierPro/Editor/ViewModel/EditorViewModel+Ripple.swift (GPLv3).

use crate::error::TimelineError;
use clawvinci_model::timeline::{Clip, Timeline};
use clawvinci_model::timeline_marker::TimelineMarker;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Proposed new start frame for a single clip produced by ripple calculation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClipShift {
    pub clip_id: String,
    pub new_start_frame: i64,
}

/// A half-open `[start, end)` frame interval on a track describing removed regions or gaps.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct FrameRange {
    pub start: i64,
    pub end: i64,
}

impl FrameRange {
    pub fn new(start: i64, end: i64) -> Self {
        Self { start, end }
    }

    pub fn length(&self) -> i64 {
        self.end - self.start
    }
}

/// Edge of a clip being trimmed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrimEdge {
    Left,
    Right,
}

/// Pure functions for ripple editing: computing shifts after deletion, insertion, or trimming.
pub struct RippleEngine;

impl RippleEngine {
    /// After removing clips from a track, computes new start frames for remaining
    /// clips that should shift backward to close the gap.
    pub fn compute_ripple_shifts(clips: &[Clip], removed_ids: &HashSet<String>) -> Vec<ClipShift> {
        let removed_ranges: Vec<FrameRange> = clips
            .iter()
            .filter(|c| removed_ids.contains(&c.id))
            .map(|c| FrameRange::new(c.start_frame, c.end_frame()))
            .collect();
        let remaining_clips: Vec<&Clip> = clips
            .iter()
            .filter(|c| !removed_ids.contains(&c.id))
            .collect();
        Self::compute_ripple_shifts_for_ranges(&remaining_clips, &removed_ranges)
    }

    /// Shifts clips leftward to close the gaps defined by `removed_ranges`.
    pub fn compute_ripple_shifts_for_ranges(
        clips: &[&Clip],
        removed_ranges: &[FrameRange],
    ) -> Vec<ClipShift> {
        let merged = Self::merge_ranges(removed_ranges);
        if merged.is_empty() {
            return Vec::new();
        }

        let mut sorted_clips = clips.to_vec();
        sorted_clips.sort_by_key(|c| c.start_frame);

        let mut shifts = Vec::new();
        for clip in sorted_clips {
            let shift: i64 = merged
                .iter()
                .filter(|r| r.end <= clip.start_frame)
                .map(|r| r.length())
                .sum();
            if shift > 0 {
                shifts.push(ClipShift {
                    clip_id: clip.id.clone(),
                    new_start_frame: clip.start_frame - shift,
                });
            }
        }
        shifts
    }

    /// Pushes all clips at or after `insert_frame` forward by `push_amount` frames.
    pub fn compute_ripple_push(
        clips: &[Clip],
        insert_frame: i64,
        push_amount: i64,
        exclude_ids: &HashSet<String>,
    ) -> Vec<ClipShift> {
        clips
            .iter()
            .filter(|c| !exclude_ids.contains(&c.id) && c.start_frame >= insert_frame)
            .map(|c| ClipShift {
                clip_id: c.id.clone(),
                new_start_frame: c.start_frame + push_amount,
            })
            .collect()
    }

    /// Merges overlapping and abutting frame ranges.
    pub fn merge_ranges(ranges: &[FrameRange]) -> Vec<FrameRange> {
        if ranges.is_empty() {
            return Vec::new();
        }
        let mut sorted = ranges.to_vec();
        sorted.sort_by_key(|r| r.start);

        let mut merged: Vec<FrameRange> = Vec::new();
        for range in sorted {
            if let Some(last) = merged.last_mut() {
                if range.start <= last.end {
                    last.end = last.end.max(range.end);
                } else {
                    merged.push(range);
                }
            } else {
                merged.push(range);
            }
        }
        merged
    }

    /// Shifts timeline markers to close removed spans across tracks.
    pub fn ripple_markers(
        markers: &[TimelineMarker],
        closing_track_ranges: &[Vec<FrameRange>],
    ) -> Vec<TimelineMarker> {
        let merged_tracks: Vec<Vec<FrameRange>> = closing_track_ranges
            .iter()
            .map(|ranges| {
                let valid: Vec<FrameRange> = ranges.iter().filter(|r| r.length() > 0).copied().collect();
                Self::merge_ranges(&valid)
            })
            .filter(|m| !m.is_empty())
            .collect();

        if merged_tracks.is_empty() {
            return markers.to_vec();
        }

        let mut result = Vec::new();
        for marker in markers {
            let is_range = marker.duration_frames > 0;
            let mut surviving = Vec::new();

            for ranges in &merged_tracks {
                let start = Self::map_frame(marker.start_frame, ranges);
                if !is_range {
                    let removed = ranges.iter().any(|r| r.start <= marker.start_frame && marker.start_frame < r.end);
                    if !removed {
                        surviving.push((start, start));
                    }
                } else {
                    let end = Self::map_frame(marker.end_frame(), ranges);
                    if end > start {
                        surviving.push((start, end));
                    }
                }
            }

            if let Some(&(first_start, first_end)) = surviving.first() {
                let mut next = marker.clone();
                let min_start = surviving.iter().map(|s| s.0).min().unwrap_or(first_start);
                next.start_frame = min_start;
                if is_range {
                    let min_end = surviving.iter().map(|s| s.1).min().unwrap_or(first_end);
                    if min_end > next.start_frame {
                        next.duration_frames = min_end - next.start_frame;
                        result.push(next);
                    }
                } else {
                    result.push(next);
                }
            }
        }
        result
    }

    /// Adjusts markers when a gap is opened or closed at `insert_frame` by `push_amount`.
    pub fn ripple_markers_opening(
        markers: &[TimelineMarker],
        insert_frame: i64,
        push_amount: i64,
    ) -> Vec<TimelineMarker> {
        if push_amount == 0 {
            return markers.to_vec();
        }
        if push_amount < 0 {
            return Self::ripple_markers(
                markers,
                &[vec![FrameRange::new(insert_frame + push_amount, insert_frame)]],
            );
        }
        markers
            .iter()
            .map(|marker| {
                let mut next = marker.clone();
                let is_range = next.duration_frames > 0;
                if next.start_frame >= insert_frame {
                    next.start_frame += push_amount;
                } else if is_range && next.end_frame() > insert_frame {
                    next.duration_frames += push_amount;
                }
                next
            })
            .collect()
    }

    fn map_frame(frame: i64, ranges: &[FrameRange]) -> i64 {
        let mut mapped = frame;
        for range in ranges {
            if range.end <= frame {
                mapped -= range.length();
            } else if range.start < frame {
                mapped -= frame - range.start;
            }
        }
        mapped.max(0)
    }
}

/// High-level ripple deletion: removes clips and shifts downstream clips on
/// the owning tracks and all sync-locked tracks.
pub fn ripple_delete_clips(
    timeline: &mut Timeline,
    clip_ids: &HashSet<String>,
) -> Result<usize, TimelineError> {
    if clip_ids.is_empty() {
        return Ok(0);
    }

    // Collect tracks and removed ranges
    let mut track_ranges: Vec<(usize, Vec<FrameRange>)> = Vec::new();
    let mut total_removed = 0;

    for (t_idx, track) in timeline.tracks.iter().enumerate() {
        let ranges: Vec<FrameRange> = track
            .clips
            .iter()
            .filter(|c| clip_ids.contains(&c.id))
            .map(|c| FrameRange::new(c.start_frame, c.end_frame()))
            .collect();
        if !ranges.is_empty() {
            total_removed += ranges.len();
            track_ranges.push((t_idx, ranges));
        }
    }

    if total_removed == 0 {
        return Ok(0);
    }

    // Remove clips from tracks
    for track in &mut timeline.tracks {
        track.clips.retain(|c| !clip_ids.contains(&c.id));
    }

    // Apply shifts: for each track with removals, shift remaining clips.
    // Also shift sync-locked tracks by the combined removed ranges.
    let all_removed_ranges: Vec<FrameRange> = track_ranges
        .iter()
        .flat_map(|(_, ranges)| ranges.iter().copied())
        .collect();
    let merged_global_ranges = RippleEngine::merge_ranges(&all_removed_ranges);

    for (t_idx, track) in timeline.tracks.iter_mut().enumerate() {
        let own_ranges = track_ranges.iter().find(|(idx, _)| *idx == t_idx);
        let ranges_to_apply = if let Some((_, r)) = own_ranges {
            r.as_slice()
        } else if track.sync_locked {
            merged_global_ranges.as_slice()
        } else {
            &[]
        };

        if !ranges_to_apply.is_empty() {
            let clip_refs: Vec<&Clip> = track.clips.iter().collect();
            let shifts = RippleEngine::compute_ripple_shifts_for_ranges(&clip_refs, ranges_to_apply);
            for shift in shifts {
                if let Some(c) = track.clips.iter_mut().find(|c| c.id == shift.clip_id) {
                    c.start_frame = shift.new_start_frame;
                }
            }
            track.clips.sort_by_key(|c| c.start_frame);
        }
    }

    // Ripple timeline markers
    let closing_ranges: Vec<Vec<FrameRange>> = track_ranges
        .into_iter()
        .map(|(_, ranges)| ranges)
        .collect();
    timeline.markers = RippleEngine::ripple_markers(&timeline.markers, &closing_ranges);

    Ok(total_removed)
}

/// High-level ripple insertion: pushes downstream clips forward and places new clip.
pub fn ripple_insert_clip(
    timeline: &mut Timeline,
    track_index: usize,
    clip: Clip,
) -> Result<(), TimelineError> {
    if track_index >= timeline.tracks.len() {
        return Err(TimelineError::TrackNotFound(track_index));
    }
    if !timeline.tracks[track_index].track_type.is_compatible(clip.media_type) {
        return Err(TimelineError::TrackIncompatible {
            track_type: timeline.tracks[track_index].track_type,
            clip_type: clip.media_type,
        });
    }

    let insert_frame = clip.start_frame;
    let push_amount = clip.duration_frames;
    if push_amount <= 0 {
        return Err(TimelineError::InvalidDuration(push_amount));
    }

    let empty_exclude = HashSet::new();

    // Push downstream clips on target track and on all sync-locked tracks
    for (t_idx, track) in timeline.tracks.iter_mut().enumerate() {
        if t_idx == track_index || track.sync_locked {
            let shifts = RippleEngine::compute_ripple_push(
                &track.clips,
                insert_frame,
                push_amount,
                &empty_exclude,
            );
            for shift in shifts {
                if let Some(c) = track.clips.iter_mut().find(|c| c.id == shift.clip_id) {
                    c.start_frame = shift.new_start_frame;
                }
            }
        }
    }

    // Insert clip on target track and sort
    timeline.tracks[track_index].clips.push(clip);
    timeline.tracks[track_index].clips.sort_by_key(|c| c.start_frame);

    // Ripple markers opening
    timeline.markers = RippleEngine::ripple_markers_opening(&timeline.markers, insert_frame, push_amount);

    Ok(())
}
