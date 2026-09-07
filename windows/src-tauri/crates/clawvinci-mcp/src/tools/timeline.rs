// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only
// Derived from Sources/PalmierPro/Agent/Tools/ToolExecutor+Timeline.swift and InspectTimeline.swift (GPLv3).

use crate::result::ToolResult;
use crate::state::McpState;
use serde_json::{json, Value};

pub fn get_timeline(args: &Value, state: &mut McpState) -> ToolResult {
    let timeline = state.editor.timeline();
    let total_frames = timeline.total_frames();
    let fps = timeline.fps.max(1) as f64;
    let duration_seconds = total_frames as f64 / fps;

    let start_window = args.get("startFrame").and_then(|v| v.as_i64());
    let end_window = args.get("endFrame").and_then(|v| v.as_i64());

    let mut tracks_json = Vec::new();
    for (idx, track) in timeline.tracks.iter().enumerate() {
        let mut sorted_clips = track.clips.clone();
        sorted_clips.sort_by_key(|c| c.start_frame);

        let mut clips_json = Vec::new();
        let mut gaps = Vec::new();
        let mut prev_end = 0i64;

        for clip in sorted_clips {
            let start = clip.start_frame;
            let end = clip.end_frame();

            if start > prev_end {
                gaps.push(json!({ "start": prev_end, "end": start }));
            }
            prev_end = end.max(prev_end);

            if let Some(sw) = start_window {
                if end <= sw {
                    continue;
                }
            }
            if let Some(ew) = end_window {
                if start >= ew {
                    continue;
                }
            }

            clips_json.push(json!({
                "id": clip.id,
                "mediaRef": clip.media_ref,
                "startFrame": clip.start_frame,
                "durationFrames": clip.duration_frames,
                "endFrame": end,
                "speed": clip.speed,
                "opacity": clip.opacity,
                "volumeDb": clip.volume,
                "blendMode": format!("{:?}", clip.blend_mode).to_lowercase()
            }));
        }

        tracks_json.push(json!({
            "trackId": track.id,
            "trackIndex": idx,
            "type": format!("{:?}", track.track_type).to_lowercase(),
            "name": track.name,
            "muted": track.muted,
            "hidden": track.hidden,
            "syncLocked": track.sync_locked,
            "clips": clips_json,
            "gaps": gaps
        }));
    }

    let mut markers_json = Vec::new();
    for m in &state.markers {
        markers_json.push(json!({
            "markerId": m.id,
            "name": m.name,
            "comment": m.comment,
            "color": m.color,
            "startFrame": m.start_frame,
            "durationFrames": m.duration_frames,
            "status": m.status
        }));
    }

    ToolResult::json(&json!({
        "timelineId": timeline.id,
        "name": timeline.name,
        "fps": timeline.fps,
        "width": timeline.width,
        "height": timeline.height,
        "totalFrames": total_frames,
        "durationSeconds": duration_seconds,
        "canGenerate": false,
        "tracks": tracks_json,
        "markers": markers_json
    }))
}

pub fn inspect_timeline(args: &Value, state: &mut McpState) -> ToolResult {
    let timeline = state.editor.timeline();
    let total_frames = timeline.total_frames();

    let mut gaps_detected = 0;
    let mut total_clips = 0;
    let mut tracks_report = Vec::new();

    for (idx, track) in timeline.tracks.iter().enumerate() {
        let mut sorted = track.clips.clone();
        sorted.sort_by_key(|c| c.start_frame);
        total_clips += sorted.len();

        let mut track_gaps = Vec::new();
        let mut prev_end = 0i64;
        for c in sorted {
            if c.start_frame > prev_end {
                track_gaps.push(json!({ "start": prev_end, "end": c.start_frame }));
                gaps_detected += 1;
            }
            prev_end = c.end_frame().max(prev_end);
        }

        tracks_report.push(json!({
            "trackIndex": idx,
            "trackType": format!("{:?}", track.track_type).to_lowercase(),
            "clipCount": track.clips.len(),
            "gaps": track_gaps
        }));
    }

    let sample_frame = args.get("startFrame").and_then(|v| v.as_i64()).unwrap_or(0);

    ToolResult::json(&json!({
        "totalFrames": total_frames,
        "totalClips": total_clips,
        "gapsDetected": gaps_detected,
        "sampleFrame": sample_frame,
        "tracks": tracks_report,
        "healthy": gaps_detected == 0
    }))
}

pub fn set_project_settings(args: &Value, state: &mut McpState) -> ToolResult {
    let fps_opt = args.get("fps").and_then(|v| v.as_i64()).map(|f| f as i32);
    let width_opt = args.get("width").and_then(|v| v.as_i64()).map(|w| w as i32);
    let height_opt = args.get("height").and_then(|v| v.as_i64()).map(|h| h as i32);

    let (calc_w, calc_h) = if let Some(aspect) = args.get("aspectRatio").and_then(|v| v.as_str()) {
        match aspect {
            "16:9" => (1920, 1080),
            "9:16" => (1080, 1920),
            "1:1" => (1080, 1080),
            "4:3" => (1440, 1080),
            "21:9" => (2560, 1080),
            _ => (1920, 1080),
        }
    } else {
        (
            width_opt.unwrap_or(state.editor.timeline().width),
            height_opt.unwrap_or(state.editor.timeline().height),
        )
    };

    let result = state.editor.perform("Change Project Settings", |tl| {
        if let Some(f) = fps_opt {
            tl.fps = f;
        }
        tl.width = calc_w;
        tl.height = calc_h;
        Ok(())
    });

    match result {
        Ok(()) => {
            state.bump_version();
            ToolResult::ok(format!(
                "Updated project settings: {}x{}, {} fps",
                calc_w,
                calc_h,
                fps_opt.unwrap_or(state.editor.timeline().fps)
            ))
        }
        Err(e) => ToolResult::error(format!("Failed to update project settings: {e}")),
    }
}
