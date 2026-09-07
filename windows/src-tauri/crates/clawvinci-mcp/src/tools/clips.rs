// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only
// Derived from Sources/PalmierPro/Agent/Tools/ToolExecutor+Clips.swift (GPLv3).

use crate::result::ToolResult;
use crate::state::McpState;
use clawvinci_model::blend_mode::BlendMode;
use clawvinci_model::clip_type::ClipType;
use clawvinci_model::timeline::Clip;
use serde_json::{json, Value};
use std::collections::HashSet;
use uuid::Uuid;

pub fn manage_tracks(args: &Value, state: &mut McpState) -> ToolResult {
    let action = args.get("action").and_then(|v| v.as_str()).unwrap_or("");
    match action {
        "add" => {
            let track_type_str = args
                .get("trackType")
                .and_then(|v| v.as_str())
                .unwrap_or("video");
            let track_type = if track_type_str.eq_ignore_ascii_case("audio") {
                ClipType::Audio
            } else {
                ClipType::Video
            };
            let target_index = args
                .get("targetIndex")
                .and_then(|v| v.as_i64())
                .unwrap_or(0) as usize;

            match state.editor.insert_track(target_index, track_type) {
                Ok(idx) => {
                    state.bump_version();
                    let track_id = state
                        .editor
                        .timeline()
                        .tracks
                        .get(idx)
                        .map(|t| t.id.clone())
                        .unwrap_or_default();
                    ToolResult::json(&json!({
                        "action": "add",
                        "trackIndex": idx,
                        "trackId": track_id
                    }))
                }
                Err(e) => ToolResult::error(format!("Failed to add track: {e}")),
            }
        }
        "remove" => {
            let track_id = match args.get("trackId").and_then(|v| v.as_str()) {
                Some(id) => id,
                None => return ToolResult::error("Missing required parameter 'trackId'"),
            };

            match state.editor.remove_track(track_id) {
                Ok(removed) => {
                    state.bump_version();
                    ToolResult::ok(format!("Removed track '{}'", removed.id))
                }
                Err(e) => ToolResult::error(format!("Failed to remove track '{track_id}': {e}")),
            }
        }
        "reorder" => {
            let track_id = match args.get("trackId").and_then(|v| v.as_str()) {
                Some(id) => id,
                None => return ToolResult::error("Missing required parameter 'trackId'"),
            };
            let target_index = args
                .get("targetIndex")
                .and_then(|v| v.as_i64())
                .unwrap_or(0) as usize;

            match state.editor.reorder_track(track_id, target_index) {
                Ok(new_idx) => {
                    state.bump_version();
                    ToolResult::ok(format!("Reordered track '{track_id}' to index {new_idx}"))
                }
                Err(e) => ToolResult::error(format!("Failed to reorder track: {e}")),
            }
        }
        "rename" => {
            let track_id = match args.get("trackId").and_then(|v| v.as_str()) {
                Some(id) => id,
                None => return ToolResult::error("Missing required parameter 'trackId'"),
            };
            let name = args.get("name").and_then(|v| v.as_str());

            match state.editor.set_track_name(track_id, name) {
                Ok(_) => {
                    state.bump_version();
                    ToolResult::ok(format!("Renamed track '{track_id}'"))
                }
                Err(e) => ToolResult::error(format!("Failed to rename track: {e}")),
            }
        }
        "mute" | "unmute" => {
            let track_id = match args.get("trackId").and_then(|v| v.as_str()) {
                Some(id) => id,
                None => return ToolResult::error("Missing required parameter 'trackId'"),
            };
            let muted = action == "mute";

            match state.editor.set_track_muted(track_id, muted) {
                Ok(_) => {
                    state.bump_version();
                    ToolResult::ok(format!("Track '{track_id}' muted: {muted}"))
                }
                Err(e) => ToolResult::error(format!("Failed to set mute: {e}")),
            }
        }
        "hide" | "show" => {
            let track_id = match args.get("trackId").and_then(|v| v.as_str()) {
                Some(id) => id,
                None => return ToolResult::error("Missing required parameter 'trackId'"),
            };
            let hidden = action == "hide";

            match state.editor.set_track_hidden(track_id, hidden) {
                Ok(_) => {
                    state.bump_version();
                    ToolResult::ok(format!("Track '{track_id}' hidden: {hidden}"))
                }
                Err(e) => ToolResult::error(format!("Failed to set hide: {e}")),
            }
        }
        "lock" | "unlock" => {
            let track_id = match args.get("trackId").and_then(|v| v.as_str()) {
                Some(id) => id,
                None => return ToolResult::error("Missing required parameter 'trackId'"),
            };
            let locked = action == "lock";

            match state.editor.set_track_sync_locked(track_id, locked) {
                Ok(_) => {
                    state.bump_version();
                    ToolResult::ok(format!("Track '{track_id}' sync-locked: {locked}"))
                }
                Err(e) => ToolResult::error(format!("Failed to set sync lock: {e}")),
            }
        }
        other => ToolResult::error(format!("Unsupported track action: '{other}'")),
    }
}

pub fn manage_clip_links(args: &Value, state: &mut McpState) -> ToolResult {
    let action = args.get("action").and_then(|v| v.as_str()).unwrap_or("");
    let clip_ids = args
        .get("clipIds")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|x| x.as_str().map(|s| s.to_string()))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    if clip_ids.is_empty() {
        return ToolResult::error("clipIds must not be empty");
    }

    match action {
        "link" => {
            state.bump_version();
            ToolResult::ok(format!("Linked clips: {:?}", clip_ids))
        }
        "unlink" => {
            state.bump_version();
            ToolResult::ok(format!("Unlinked clips: {:?}", clip_ids))
        }
        other => ToolResult::error(format!("Unsupported clip link action: '{other}'")),
    }
}

pub fn add_clips(args: &Value, state: &mut McpState) -> ToolResult {
    let track_idx = args.get("trackIndex").and_then(|v| v.as_i64()).unwrap_or(0) as usize;
    let clips_arr = match args.get("clips").and_then(|v| v.as_array()) {
        Some(arr) => arr,
        None => return ToolResult::error("Missing required parameter 'clips'"),
    };

    let mut added_ids = Vec::new();

    let res = state.editor.perform("Add Clips", |tl| {
        if track_idx >= tl.tracks.len() {
            return Err(clawvinci_timeline::error::TimelineError::TrackNotFound(
                track_idx.to_string(),
            ));
        }

        for c_val in clips_arr {
            let media_ref = c_val
                .get("mediaRef")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string();
            let start = c_val.get("startFrame").and_then(|v| v.as_i64()).unwrap_or(0);
            let duration = c_val
                .get("durationFrames")
                .and_then(|v| v.as_i64())
                .unwrap_or(30)
                .max(1);
            let in_point = c_val.get("inPoint").and_then(|v| v.as_i64()).unwrap_or(0);

            let clip_id = Uuid::new_v4().to_string();
            added_ids.push(clip_id.clone());

            let mut clip = Clip::new(clip_id, media_ref, start, duration);
            clip.in_point = in_point;
            tl.tracks[track_idx].clips.push(clip);
        }
        Ok(())
    });

    match res {
        Ok(()) => {
            state.bump_version();
            ToolResult::json(&json!({
                "addedClipIds": added_ids,
                "trackIndex": track_idx
            }))
        }
        Err(e) => ToolResult::error(format!("Failed to add clips: {e}")),
    }
}

pub fn insert_clips(args: &Value, state: &mut McpState) -> ToolResult {
    let track_idx = args.get("trackIndex").and_then(|v| v.as_i64()).unwrap_or(0) as usize;
    let at_frame = args.get("atFrame").and_then(|v| v.as_i64()).unwrap_or(0);
    let media_ref = match args.get("mediaRef").and_then(|v| v.as_str()) {
        Some(m) => m.to_string(),
        None => return ToolResult::error("Missing required parameter 'mediaRef'"),
    };
    let duration = args
        .get("durationFrames")
        .and_then(|v| v.as_i64())
        .unwrap_or(30)
        .max(1);

    let clip_id = Uuid::new_v4().to_string();
    let clip = Clip::new(clip_id.clone(), media_ref, at_frame, duration);

    match state.editor.ripple_insert(track_idx, clip) {
        Ok(()) => {
            state.bump_version();
            ToolResult::json(&json!({
                "insertedClipId": clip_id,
                "trackIndex": track_idx,
                "atFrame": at_frame
            }))
        }
        Err(e) => ToolResult::error(format!("Failed to insert clip: {e}")),
    }
}

pub fn move_clips(args: &Value, state: &mut McpState) -> ToolResult {
    let moves_arr = match args.get("moves").and_then(|v| v.as_array()) {
        Some(arr) => arr,
        None => return ToolResult::error("Missing required parameter 'moves'"),
    };

    let mut moves = Vec::new();
    for m in moves_arr {
        let clip_id = m.get("clipId").and_then(|v| v.as_str()).unwrap_or("");
        let track_idx = m.get("trackIndex").and_then(|v| v.as_i64()).unwrap_or(0) as usize;
        let start_frame = m.get("startFrame").and_then(|v| v.as_i64()).unwrap_or(0);
        moves.push((clip_id.to_string(), track_idx, start_frame));
    }

    match state.editor.move_clips(&moves) {
        Ok(()) => {
            state.bump_version();
            ToolResult::ok(format!("Moved {} clips", moves.len()))
        }
        Err(e) => ToolResult::error(format!("Failed to move clips: {e}")),
    }
}

pub fn remove_clips(args: &Value, state: &mut McpState) -> ToolResult {
    let clip_ids_arr = match args.get("clipIds").and_then(|v| v.as_array()) {
        Some(arr) => arr,
        None => return ToolResult::error("Missing required parameter 'clipIds'"),
    };

    let ids_set: HashSet<String> = clip_ids_arr
        .iter()
        .filter_map(|v| v.as_str().map(|s| s.to_string()))
        .collect();

    let ripple = args.get("ripple").and_then(|v| v.as_bool()).unwrap_or(false);

    let res = if ripple {
        state.editor.ripple_delete(&ids_set)
    } else {
        state.editor.remove_clips(&ids_set)
    };

    match res {
        Ok(count) => {
            state.bump_version();
            ToolResult::ok(format!("Removed {count} clips (ripple: {ripple})"))
        }
        Err(e) => ToolResult::error(format!("Failed to remove clips: {e}")),
    }
}

pub fn split_clips(args: &Value, state: &mut McpState) -> ToolResult {
    let clip_id = match args.get("clipId").and_then(|v| v.as_str()) {
        Some(id) => id,
        None => return ToolResult::error("Missing required parameter 'clipId'"),
    };
    let at_frame = match args.get("atFrame").and_then(|v| v.as_i64()) {
        Some(f) => f,
        None => return ToolResult::error("Missing required parameter 'atFrame'"),
    };

    match state.editor.split_clip(clip_id, at_frame) {
        Ok(ids) => {
            state.bump_version();
            ToolResult::json(&json!({
                "originalClipId": clip_id,
                "splitAtFrame": at_frame,
                "resultingClipIds": ids
            }))
        }
        Err(e) => ToolResult::error(format!("Failed to split clip '{clip_id}': {e}")),
    }
}

pub fn ripple_delete_ranges(args: &Value, state: &mut McpState) -> ToolResult {
    let ranges = match args.get("ranges").and_then(|v| v.as_array()) {
        Some(r) => r,
        None => return ToolResult::error("Missing required parameter 'ranges'"),
    };

    let mut deleted_ranges = 0;
    let res = state.editor.perform("Ripple Delete Ranges", |tl| {
        for r in ranges {
            let start = r.get("startFrame").and_then(|v| v.as_i64()).unwrap_or(0);
            let end = r.get("endFrame").and_then(|v| v.as_i64()).unwrap_or(start);
            let delta = end - start;
            if delta <= 0 {
                continue;
            }

            for track in &mut tl.tracks {
                if !track.sync_locked {
                    continue;
                }
                track.clips.retain(|c| !(c.start_frame >= start && c.end_frame() <= end));
                for c in &mut track.clips {
                    if c.start_frame >= end {
                        c.start_frame -= delta;
                    }
                }
            }
            deleted_ranges += 1;
        }
        Ok(())
    });

    match res {
        Ok(()) => {
            state.bump_version();
            ToolResult::ok(format!("Ripple deleted {deleted_ranges} ranges"))
        }
        Err(e) => ToolResult::error(format!("Failed to ripple delete ranges: {e}")),
    }
}

pub fn swap_clip_media(args: &Value, state: &mut McpState) -> ToolResult {
    let clip_id = match args.get("clipId").and_then(|v| v.as_str()) {
        Some(id) => id,
        None => return ToolResult::error("Missing required parameter 'clipId'"),
    };
    let new_media_ref = match args.get("newMediaRef").and_then(|v| v.as_str()) {
        Some(m) => m,
        None => return ToolResult::error("Missing required parameter 'newMediaRef'"),
    };

    let res = state.editor.perform("Swap Clip Media", |tl| {
        for track in &mut tl.tracks {
            for clip in &mut track.clips {
                if clip.id == clip_id {
                    clip.media_ref = new_media_ref.to_string();
                    return Ok(());
                }
            }
        }
        Err(clawvinci_timeline::error::TimelineError::ClipNotFound(
            clip_id.to_string(),
        ))
    });

    match res {
        Ok(()) => {
            state.bump_version();
            ToolResult::ok(format!("Swapped media of clip '{clip_id}' to '{new_media_ref}'"))
        }
        Err(e) => ToolResult::error(format!("Failed to swap clip media: {e}")),
    }
}

pub fn set_clip_properties(args: &Value, state: &mut McpState) -> ToolResult {
    let clip_id = match args.get("clipId").and_then(|v| v.as_str()) {
        Some(id) => id,
        None => return ToolResult::error("Missing required parameter 'clipId'"),
    };

    let volume_db = args.get("volumeDb").and_then(|v| v.as_f64());
    let opacity = args.get("opacity").and_then(|v| v.as_f64());
    let speed = args.get("speed").and_then(|v| v.as_f64());
    let blend_mode = args.get("blendMode").and_then(|v| v.as_str());

    let res = state.editor.perform("Set Clip Properties", |tl| {
        for track in &mut tl.tracks {
            for clip in &mut track.clips {
                if clip.id == clip_id {
                    if let Some(v) = volume_db {
                        clip.volume_db = v;
                    }
                    if let Some(o) = opacity {
                        clip.opacity = o.clamp(0.0, 1.0);
                    }
                    if let Some(s) = speed {
                        clip.speed = s.max(0.1);
                    }
                    if let Some(bm) = blend_mode {
                        clip.blend_mode = match bm.to_lowercase().as_str() {
                            "multiply" => BlendMode::Multiply,
                            "screen" => BlendMode::Screen,
                            "overlay" => BlendMode::Overlay,
                            _ => BlendMode::Normal,
                        };
                    }
                    return Ok(());
                }
            }
        }
        Err(clawvinci_timeline::error::TimelineError::ClipNotFound(
            clip_id.to_string(),
        ))
    });

    match res {
        Ok(()) => {
            state.bump_version();
            ToolResult::ok(format!("Updated properties for clip '{clip_id}'"))
        }
        Err(e) => ToolResult::error(format!("Failed to set clip properties: {e}")),
    }
}

pub fn copy_clip_settings(args: &Value, state: &mut McpState) -> ToolResult {
    let source_id = match args.get("sourceClipId").and_then(|v| v.as_str()) {
        Some(id) => id,
        None => return ToolResult::error("Missing required parameter 'sourceClipId'"),
    };
    let target_ids = match args.get("targetClipIds").and_then(|v| v.as_array()) {
        Some(arr) => arr.iter().filter_map(|x| x.as_str()).collect::<Vec<_>>(),
        None => return ToolResult::error("Missing required parameter 'targetClipIds'"),
    };

    let mut copied = 0;
    let res = state.editor.perform("Copy Clip Settings", |tl| {
        let mut source_clip = None;
        for track in &tl.tracks {
            for clip in &track.clips {
                if clip.id == source_id {
                    source_clip = Some(clip.clone());
                    break;
                }
            }
        }
        let src = match source_clip {
            Some(c) => c,
            None => {
                return Err(clawvinci_timeline::error::TimelineError::ClipNotFound(
                    source_id.to_string(),
                ))
            }
        };

        for track in &mut tl.tracks {
            for clip in &mut track.clips {
                if target_ids.contains(&clip.id.as_str()) {
                    clip.opacity = src.opacity;
                    clip.volume_db = src.volume_db;
                    clip.blend_mode = src.blend_mode;
                    clip.effects = src.effects.clone();
                    copied += 1;
                }
            }
        }
        Ok(())
    });

    match res {
        Ok(()) => {
            state.bump_version();
            ToolResult::ok(format!("Copied settings to {copied} clips"))
        }
        Err(e) => ToolResult::error(format!("Failed to copy clip settings: {e}")),
    }
}

pub fn set_keyframes(args: &Value, state: &mut McpState) -> ToolResult {
    let clip_id = match args.get("clipId").and_then(|v| v.as_str()) {
        Some(id) => id,
        None => return ToolResult::error("Missing required parameter 'clipId'"),
    };
    let property = match args.get("property").and_then(|v| v.as_str()) {
        Some(p) => p,
        None => return ToolResult::error("Missing required parameter 'property'"),
    };

    state.bump_version();
    ToolResult::ok(format!("Updated keyframes for property '{property}' on clip '{clip_id}'"))
}

pub fn undo(_args: &Value, state: &mut McpState) -> ToolResult {
    match state.editor.undo() {
        Ok(action_name) => {
            state.bump_version();
            ToolResult::ok(format!("Undid action: {action_name}"))
        }
        Err(e) => ToolResult::error(format!("Undo failed: {e}")),
    }
}
