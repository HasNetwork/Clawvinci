// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only
// Derived from Sources/PalmierPro/Editor/OverwriteEngine.swift (GPLv3).

use crate::error::TimelineError;
use clawvinci_model::timeline::{Clip, Timeline, Track};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Actions computed by the overwrite engine to clear a track interval.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OverwriteAction {
    Remove {
        clip_id: String,
    },
    TrimEnd {
        clip_id: String,
        new_duration: i64,
    },
    TrimStart {
        clip_id: String,
        new_start_frame: i64,
        new_trim_start: i64,
        new_duration: i64,
    },
    Split {
        clip_id: String,
        left_duration: i64,
        right_id: String,
        right_start_frame: i64,
        right_trim_start: i64,
        right_duration: i64,
    },
}

/// Pure functions for non-destructive overwrite region clearing.
pub struct OverwriteEngine;

impl OverwriteEngine {
    /// Computes the exact actions needed to clear `[region_start, region_end)` on a track.
    pub fn compute_overwrite(
        clips: &[Clip],
        region_start: i64,
        region_end: i64,
    ) -> Vec<OverwriteAction> {
        if region_end <= region_start {
            return Vec::new();
        }

        let mut actions = Vec::new();
        for clip in clips {
            let cs = clip.start_frame;
            let ce = clip.end_frame();

            if ce <= region_start || cs >= region_end {
                continue;
            }

            if cs >= region_start && ce <= region_end {
                actions.push(OverwriteAction::Remove {
                    clip_id: clip.id.clone(),
                });
            } else if cs < region_start && ce > region_end {
                let left_duration = region_start - cs;
                let right_start_frame = region_end;
                let speed = clip.speed.max(0.001);
                let right_source_offset = (((region_end - cs) as f64) * speed).round() as i64;
                let right_trim_start = clip.trim_start_frame + right_source_offset;
                let right_duration = ce - region_end;

                actions.push(OverwriteAction::Split {
                    clip_id: clip.id.clone(),
                    left_duration,
                    right_id: Uuid::new_v4().to_string(),
                    right_start_frame,
                    right_trim_start,
                    right_duration,
                });
            } else if cs < region_start {
                // Overlaps left side — trim right edge
                let new_duration = region_start - cs;
                actions.push(OverwriteAction::TrimEnd {
                    clip_id: clip.id.clone(),
                    new_duration,
                });
            } else {
                // Overlaps right side — trim left edge
                let trim_amount = region_end - cs;
                let speed = clip.speed.max(0.001);
                let new_start_frame = region_end;
                let new_trim_start = clip.trim_start_frame + ((trim_amount as f64) * speed).round() as i64;
                let new_duration = ce - region_end;
                actions.push(OverwriteAction::TrimStart {
                    clip_id: clip.id.clone(),
                    new_start_frame,
                    new_trim_start,
                    new_duration,
                });
            }
        }

        actions
    }
}

/// Clears region `[region_start, region_end)` on a track by executing computed overwrite actions.
pub fn clear_region(track: &mut Track, region_start: i64, region_end: i64) {
    let actions = OverwriteEngine::compute_overwrite(&track.clips, region_start, region_end);
    if actions.is_empty() {
        return;
    }

    let mut new_splits: Vec<Clip> = Vec::new();

    for action in actions {
        match action {
            OverwriteAction::Remove { clip_id } => {
                track.clips.retain(|c| c.id != clip_id);
            }
            OverwriteAction::TrimEnd {
                clip_id,
                new_duration,
            } => {
                if let Some(c) = track.clips.iter_mut().find(|c| c.id == clip_id) {
                    let old_duration = c.duration_frames;
                    let speed = c.speed.max(0.001);
                    let source_delta = (((old_duration - new_duration) as f64) * speed).round() as i64;
                    c.duration_frames = new_duration;
                    c.trim_end_frame += source_delta;
                    c.fade_out_frames = 0;
                    c.clamp_fades_to_duration();
                    c.clamp_keyframes_to_duration();
                }
            }
            OverwriteAction::TrimStart {
                clip_id,
                new_start_frame,
                new_trim_start,
                new_duration,
            } => {
                if let Some(c) = track.clips.iter_mut().find(|c| c.id == clip_id) {
                    let offset = new_start_frame - c.start_frame;
                    c.start_frame = new_start_frame;
                    c.trim_start_frame = new_trim_start;
                    c.duration_frames = new_duration;
                    c.fade_in_frames = 0;
                    c.clamp_fades_to_duration();

                    // Rebase keyframe tracks
                    if let Some(track) = &c.opacity_track {
                        c.opacity_track = Some(track.rebased(offset, c.opacity));
                    }
                    if let Some(track) = &c.volume_track {
                        c.volume_track = Some(track.rebased(offset, c.volume));
                    }
                    if let Some(track) = &c.rotation_track {
                        c.rotation_track = Some(track.rebased(offset, c.rotation));
                    }
                    c.clamp_keyframes_to_duration();
                }
            }
            OverwriteAction::Split {
                clip_id,
                left_duration,
                right_id,
                right_start_frame,
                right_trim_start,
                right_duration,
            } => {
                if let Some(c) = track.clips.iter_mut().find(|c| c.id == clip_id) {
                    let mut right = c.clone();
                    right.id = right_id;
                    right.start_frame = right_start_frame;
                    right.trim_start_frame = right_trim_start;
                    right.duration_frames = right_duration;
                    right.fade_in_frames = 0;

                    let split_offset = right_start_frame - c.start_frame;
                    if let Some(t) = &right.opacity_track {
                        right.opacity_track = Some(t.rebased(split_offset, right.opacity));
                    }
                    if let Some(t) = &right.volume_track {
                        right.volume_track = Some(t.rebased(split_offset, right.volume));
                    }
                    if let Some(t) = &right.rotation_track {
                        right.rotation_track = Some(t.rebased(split_offset, right.rotation));
                    }
                    right.clamp_fades_to_duration();
                    right.clamp_keyframes_to_duration();

                    // Adjust left
                    let old_duration = c.duration_frames;
                    let speed = c.speed.max(0.001);
                    let source_delta = (((old_duration - left_duration) as f64) * speed).round() as i64;
                    c.duration_frames = left_duration;
                    c.trim_end_frame += source_delta;
                    c.fade_out_frames = 0;
                    c.clamp_fades_to_duration();
                    c.clamp_keyframes_to_duration();

                    new_splits.push(right);
                }
            }
        }
    }

    track.clips.extend(new_splits);
    track.clips.sort_by_key(|c| c.start_frame);
}

/// Overwrites a region on `track_index` with `clip`, non-destructively clearing
/// any overlapping portion and leaving surrounding clips in place.
pub fn overwrite_clip(
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

    if clip.duration_frames <= 0 {
        return Err(TimelineError::InvalidDuration(clip.duration_frames));
    }

    let start = clip.start_frame;
    let end = clip.end_frame();

    clear_region(&mut timeline.tracks[track_index], start, end);
    timeline.tracks[track_index].clips.push(clip);
    timeline.tracks[track_index].clips.sort_by_key(|c| c.start_frame);

    Ok(())
}
