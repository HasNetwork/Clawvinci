// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only
// Derived from Sources/PalmierPro/Editor/ViewModel/EditorViewModel+ClipMutations.swift
// and Sources/PalmierPro/Editor/ViewModel/EditorViewModel+Linking.swift (GPLv3).

use crate::error::TimelineError;
use crate::ripple::TrimEdge;
use clawvinci_model::clip_type::ClipType;
use clawvinci_model::keyframe::{
    AnimPair, Interpolation, Keyframe, KeyframeInterpolatable, KeyframeTrack,
};
use clawvinci_model::timeline::{Clip, ClipLocation, Timeline};
use std::collections::HashSet;
use uuid::Uuid;

fn rescale_keyframes(clip: &mut Clip, scale: f64) {
    fn rescale_track<V: KeyframeInterpolatable + PartialEq>(
        track: Option<KeyframeTrack<V>>,
        scale: f64,
    ) -> Option<KeyframeTrack<V>> {
        let mut track = track?;
        for kf in &mut track.keyframes {
            kf.frame = ((kf.frame as f64) * scale).round() as i64;
        }
        Some(track)
    }

    clip.opacity_track = rescale_track(clip.opacity_track.take(), scale);
    clip.position_track = rescale_track(clip.position_track.take(), scale);
    clip.scale_track = rescale_track(clip.scale_track.take(), scale);
    clip.rotation_track = rescale_track(clip.rotation_track.take(), scale);
    clip.crop_track = rescale_track(clip.crop_track.take(), scale);
    clip.volume_track = rescale_track(clip.volume_track.take(), scale);
}

/// Locates a clip across all tracks by ID.
pub fn find_clip(timeline: &Timeline, clip_id: &str) -> Option<ClipLocation> {
    for (t_idx, track) in timeline.tracks.iter().enumerate() {
        for (c_idx, clip) in track.clips.iter().enumerate() {
            if clip.id == clip_id {
                return Some(ClipLocation {
                    track_index: t_idx,
                    clip_index: c_idx,
                });
            }
        }
    }
    None
}

/// Splits a generic keyframe track at `split_offset`, keeping both halves continuous.
pub fn split_keyframe_track<V: KeyframeInterpolatable + PartialEq + Clone>(
    track: &Option<KeyframeTrack<V>>,
    split_offset: i64,
    fallback: V,
) -> (Option<KeyframeTrack<V>>, Option<KeyframeTrack<V>>) {
    let track = match track {
        Some(t) if t.is_active() => t,
        _ => return (track.clone(), track.clone()),
    };
    let boundary = track.sample(split_offset, fallback.clone());
    let mut left_kfs: Vec<Keyframe<V>> = track
        .keyframes
        .iter()
        .filter(|k| k.frame <= split_offset)
        .cloned()
        .collect();

    if left_kfs.last().map(|k| k.frame) != Some(split_offset) {
        let interp = left_kfs
            .last()
            .map(|k| k.interpolation_out)
            .unwrap_or(Interpolation::Smooth);
        left_kfs.push(Keyframe::new(split_offset, boundary, interp));
    }

    let left = if left_kfs.is_empty() {
        None
    } else {
        Some(KeyframeTrack::new(left_kfs))
    };
    let right = track.rebased(split_offset, fallback);
    (left, right)
}

/// Splits a clip into left and right halves at `at_frame`.
pub fn split_values(clip: &Clip, at_frame: i64) -> Result<(Clip, Clip), TimelineError> {
    if at_frame <= clip.start_frame || at_frame >= clip.end_frame() {
        return Err(TimelineError::CannotSplitAtBoundary {
            at_frame,
            start: clip.start_frame,
            end: clip.end_frame(),
        });
    }

    let split_offset = at_frame - clip.start_frame;
    let speed = clip.speed.max(0.001);
    let left_source = ((split_offset as f64) * speed).round() as i64;
    let right_source = (((clip.duration_frames - split_offset) as f64) * speed).round() as i64;

    let mut left = clip.clone();
    left.duration_frames = split_offset;
    left.trim_end_frame = clip.trim_end_frame + right_source;
    left.fade_out_frames = 0;

    let mut right = clip.clone();
    right.id = Uuid::new_v4().to_string();
    right.start_frame = at_frame;
    right.duration_frames = clip.duration_frames - split_offset;
    right.trim_start_frame = clip.trim_start_frame + left_source;
    right.fade_in_frames = 0;

    // Split keyframe tracks
    let (left_op, right_op) = split_keyframe_track(&clip.opacity_track, split_offset, clip.opacity);
    left.opacity_track = left_op;
    right.opacity_track = right_op;

    let (left_vol, right_vol) = split_keyframe_track(&clip.volume_track, split_offset, clip.volume);
    left.volume_track = left_vol;
    right.volume_track = right_vol;

    let (left_rot, right_rot) = split_keyframe_track(&clip.rotation_track, split_offset, clip.transform.rotation);
    left.rotation_track = left_rot;
    right.rotation_track = right_rot;

    let (left_pos, right_pos) = split_keyframe_track(&clip.position_track, split_offset, AnimPair::new(0.0, 0.0));
    left.position_track = left_pos;
    right.position_track = right_pos;

    let (left_scale, right_scale) = split_keyframe_track(&clip.scale_track, split_offset, AnimPair::new(1.0, 1.0));
    left.scale_track = left_scale;
    right.scale_track = right_scale;

    left.clamp_fades_to_duration();
    left.clamp_keyframes_to_duration();
    right.clamp_fades_to_duration();
    right.clamp_keyframes_to_duration();

    Ok((left, right))
}

/// Splits `clip_id` and all its linked partners at `at_frame`.
/// Returns the IDs of the right-half clips created by the split.
pub fn split_clip(timeline: &mut Timeline, clip_id: &str, at_frame: i64) -> Result<Vec<String>, TimelineError> {
    let loc = find_clip(timeline, clip_id).ok_or_else(|| TimelineError::ClipNotFound(clip_id.to_string()))?;
    let clip = &timeline.tracks[loc.track_index].clips[loc.clip_index];

    if at_frame <= clip.start_frame || at_frame >= clip.end_frame() {
        return Err(TimelineError::CannotSplitAtBoundary {
            at_frame,
            start: clip.start_frame,
            end: clip.end_frame(),
        });
    }

    // Collect clips to split (the target plus any linked partners)
    let group_ids: Vec<String> = if let Some(gid) = &clip.link_group_id {
        timeline
            .tracks
            .iter()
            .flat_map(|t| t.clips.iter())
            .filter(|c| c.link_group_id.as_deref() == Some(gid.as_str()) && at_frame > c.start_frame && at_frame < c.end_frame())
            .map(|c| c.id.clone())
            .collect()
    } else {
        vec![clip_id.to_string()]
    };

    let mut right_ids = Vec::new();
    let new_link_group = if group_ids.len() > 1 {
        Some(Uuid::new_v4().to_string())
    } else {
        None
    };

    for target_id in &group_ids {
        if let Some(target_loc) = find_clip(timeline, target_id) {
            let target_clip = &timeline.tracks[target_loc.track_index].clips[target_loc.clip_index];
            let (left, mut right) = split_values(target_clip, at_frame)?;

            if let Some(ref nlg) = new_link_group {
                right.link_group_id = Some(nlg.clone());
            }

            let right_id = right.id.clone();
            right_ids.push(right_id);

            timeline.tracks[target_loc.track_index].clips[target_loc.clip_index] = left;
            timeline.tracks[target_loc.track_index].clips.push(right);
            timeline.tracks[target_loc.track_index].clips.sort_by_key(|c| c.start_frame);
        }
    }

    Ok(right_ids)
}

/// Trims an edge of `clip_id` by `delta` frames.
pub fn trim_clip(
    timeline: &mut Timeline,
    clip_id: &str,
    edge: TrimEdge,
    delta: i64,
) -> Result<(), TimelineError> {
    if delta == 0 {
        return Ok(());
    }

    let loc = find_clip(timeline, clip_id).ok_or_else(|| TimelineError::ClipNotFound(clip_id.to_string()))?;
    let clip = &mut timeline.tracks[loc.track_index].clips[loc.clip_index];
    let unbounded = matches!(clip.media_type, ClipType::Image | ClipType::Text);
    let speed = clip.speed.max(0.001);
    let source_delta = ((delta as f64) * speed).round() as i64;

    match edge {
        TrimEdge::Left => {
            let new_start = clip.start_frame + delta;
            let new_duration = clip.duration_frames - delta;
            if new_duration <= 0 {
                return Err(TimelineError::InvalidDuration(new_duration));
            }
            let new_trim_start = if unbounded {
                clip.trim_start_frame + source_delta
            } else {
                (clip.trim_start_frame + source_delta).max(0)
            };

            clip.start_frame = new_start;
            clip.duration_frames = new_duration;
            clip.trim_start_frame = new_trim_start;
            clip.fade_in_frames = 0;

            if let Some(t) = &clip.opacity_track {
                clip.opacity_track = t.rebased(delta, clip.opacity);
            }
            if let Some(t) = &clip.volume_track {
                clip.volume_track = t.rebased(delta, clip.volume);
            }
            if let Some(t) = &clip.rotation_track {
                clip.rotation_track = t.rebased(delta, clip.transform.rotation);
            }
        }
        TrimEdge::Right => {
            let new_duration = clip.duration_frames + delta;
            if new_duration <= 0 {
                return Err(TimelineError::InvalidDuration(new_duration));
            }
            let new_trim_end = if unbounded {
                clip.trim_end_frame - source_delta
            } else {
                (clip.trim_end_frame - source_delta).max(0)
            };

            clip.duration_frames = new_duration;
            clip.trim_end_frame = new_trim_end;
            clip.fade_out_frames = 0;
        }
    }

    clip.clamp_fades_to_duration();
    clip.clamp_keyframes_to_duration();
    timeline.tracks[loc.track_index].clips.sort_by_key(|c| c.start_frame);
    Ok(())
}

/// Slips the source content of `clip_id` by `delta` frames without altering
/// its timeline start position or duration.
pub fn slip_clip(timeline: &mut Timeline, clip_id: &str, delta: i64) -> Result<(), TimelineError> {
    if delta == 0 {
        return Ok(());
    }

    let loc = find_clip(timeline, clip_id).ok_or_else(|| TimelineError::ClipNotFound(clip_id.to_string()))?;
    let clip = &mut timeline.tracks[loc.track_index].clips[loc.clip_index];
    let unbounded = matches!(clip.media_type, ClipType::Image | ClipType::Text);
    let speed = clip.speed.max(0.001);
    let source_delta = ((delta as f64) * speed).round() as i64;

    let new_trim_start = clip.trim_start_frame - source_delta;
    let new_trim_end = clip.trim_end_frame + source_delta;

    if !unbounded && (new_trim_start < 0 || new_trim_end < 0) {
        return Err(TimelineError::InvalidFrame(new_trim_start.min(new_trim_end)));
    }

    clip.trim_start_frame = new_trim_start;
    clip.trim_end_frame = new_trim_end;
    Ok(())
}

/// Changes the playback speed of `clip_id`, retiming duration and keyframes.
pub fn set_clip_speed(
    timeline: &mut Timeline,
    clip_id: &str,
    new_speed: f64,
    ripple: bool,
) -> Result<(), TimelineError> {
    if new_speed <= 0.0 {
        return Err(TimelineError::InvalidSpeed(new_speed));
    }

    let loc = find_clip(timeline, clip_id).ok_or_else(|| TimelineError::ClipNotFound(clip_id.to_string()))?;
    let clip = &timeline.tracks[loc.track_index].clips[loc.clip_index];

    if clip.multicam_group_id.is_some() {
        return Err(TimelineError::MulticamRetimeRefused(clip_id.to_string()));
    }

    let old_duration = clip.duration_frames;
    let old_end = clip.end_frame();
    let new_duration = ((((old_duration as f64) * clip.speed) / new_speed).round() as i64).max(1);

    let ti = loc.track_index;
    let ci = loc.clip_index;
    let c = &mut timeline.tracks[ti].clips[ci];
    c.speed = new_speed;
    c.duration_frames = new_duration;

    // Rescale keyframes
    let scale_factor = (new_duration as f64) / (old_duration.max(1) as f64);
    rescale_keyframes(c, scale_factor);
    c.clamp_keyframes_to_duration();
    c.clamp_fades_to_duration();

    let ripple_delta = (c.start_frame + new_duration) - old_end;
    if ripple && ripple_delta != 0 {
        let chain_ids = timeline.tracks[ti].contiguous_clip_ids(old_end, clip_id);
        for other in &mut timeline.tracks[ti].clips {
            if chain_ids.contains(&other.id) {
                other.start_frame += ripple_delta;
            }
        }
    }

    timeline.tracks[ti].clips.sort_by_key(|c| c.start_frame);
    Ok(())
}

/// Removes clips matching `clip_ids` across all tracks.
pub fn remove_clips(timeline: &mut Timeline, clip_ids: &HashSet<String>) -> usize {
    let mut total_removed = 0;
    for track in &mut timeline.tracks {
        let before = track.clips.len();
        track.clips.retain(|c| !clip_ids.contains(&c.id));
        total_removed += before - track.clips.len();
    }
    total_removed
}

/// Moves clips to target tracks and start frames.
pub fn move_clips(
    timeline: &mut Timeline,
    moves: &[(String, usize, i64)],
) -> Result<(), TimelineError> {
    if moves.is_empty() {
        return Ok(());
    }

    // Validate track indices and compatibility
    let mut clip_moves = Vec::new();
    for (clip_id, to_track, to_frame) in moves {
        let loc = find_clip(timeline, clip_id).ok_or_else(|| TimelineError::ClipNotFound(clip_id.clone()))?;
        if *to_track >= timeline.tracks.len() {
            return Err(TimelineError::TrackNotFound(*to_track));
        }
        let clip = &timeline.tracks[loc.track_index].clips[loc.clip_index];
        let dest_track = &timeline.tracks[*to_track];
        if !dest_track.track_type.is_compatible(clip.media_type) {
            return Err(TimelineError::TrackIncompatible {
                track_type: dest_track.track_type,
                clip_type: clip.media_type,
            });
        }
        clip_moves.push((clip.clone(), loc.track_index, *to_track, (*to_frame).max(0)));
    }

    let moved_ids: HashSet<String> = moves.iter().map(|m| m.0.clone()).collect();

    // Pull clips off their source tracks first
    for track in &mut timeline.tracks {
        track.clips.retain(|c| !moved_ids.contains(&c.id));
    }

    // Clear destination regions and insert clips
    for (mut clip, _, to_track, to_frame) in clip_moves {
        clip.start_frame = to_frame;
        crate::overwrite::clear_region(&mut timeline.tracks[to_track], to_frame, to_frame + clip.duration_frames);
        timeline.tracks[to_track].clips.push(clip);
        timeline.tracks[to_track].clips.sort_by_key(|c| c.start_frame);
    }

    Ok(())
}
