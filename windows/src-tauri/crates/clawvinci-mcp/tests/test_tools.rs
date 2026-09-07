// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only

use clawvinci_export::queue::ExportQueue;
use clawvinci_mcp::executor::ToolExecutor;
use clawvinci_mcp::state::{McpMediaItem, McpState};
use clawvinci_model::clip_type::ClipType;
use clawvinci_model::timeline::{Timeline, Track};
use serde_json::json;

fn setup_test_state() -> McpState {
    let mut timeline = Timeline::new(30, 1920, 1080);
    let track = Track::new(ClipType::Video);
    timeline.tracks.push(track);

    let media_items = vec![McpMediaItem {
        id: "sample-media-1".to_string(),
        name: "test_clip.mp4".to_string(),
        path: "d:/test_clip.mp4".to_string(),
        media_type: "video".to_string(),
        duration_seconds: 10.0,
        width: Some(1920),
        height: Some(1080),
        fps: Some(30.0),
        has_audio: true,
        folder: None,
        generation_prompt: None,
        generation_status: None,
    }];

    McpState::new(timeline, media_items, ExportQueue::new())
}

#[test]
fn test_tool_execution_clips_and_undo_cycle() {
    let mut state = setup_test_state();
    let executor = ToolExecutor::new();

    // 1. get_timeline initial read
    let res = executor.execute("get_timeline", &json!({}), &mut state);
    assert!(!res.is_error);
    let text = res.to_text();
    assert!(text.contains("\"totalFrames\": 0"));

    // 2. add_clips
    let add_res = executor.execute(
        "add_clips",
        &json!({
            "trackIndex": 0,
            "clips": [
                { "mediaRef": "sample-media-1", "startFrame": 0, "durationFrames": 90 }
            ]
        }),
        &mut state,
    );
    assert!(!add_res.is_error);
    assert_eq!(state.editor.timeline().tracks[0].clips.len(), 1);
    let clip_id = state.editor.timeline().tracks[0].clips[0].id.clone();
    assert_eq!(state.editor.timeline().total_frames(), 90);

    // 3. split_clips at frame 45
    let split_res = executor.execute(
        "split_clips",
        &json!({
            "clipId": clip_id,
            "atFrame": 45
        }),
        &mut state,
    );
    assert!(!split_res.is_error);
    assert_eq!(
        state.editor.timeline().tracks[0].clips.len(),
        2,
        "Expected 2 clips after split"
    );

    // 4. undo the split
    let undo_res = executor.execute("undo", &json!({}), &mut state);
    assert!(!undo_res.is_error);
    assert_eq!(
        state.editor.timeline().tracks[0].clips.len(),
        1,
        "Expected 1 clip after undo"
    );
    assert_eq!(state.editor.timeline().tracks[0].clips[0].id, clip_id);

    // 5. set_clip_properties
    let prop_res = executor.execute(
        "set_clip_properties",
        &json!({
            "clipId": clip_id,
            "opacity": 0.8,
            "volumeDb": -6.0
        }),
        &mut state,
    );
    assert!(!prop_res.is_error);
    assert_eq!(state.editor.timeline().tracks[0].clips[0].opacity, 0.8);
    assert_eq!(state.editor.timeline().tracks[0].clips[0].volume, -6.0);
}

#[test]
fn test_tool_execution_markers_and_texts() {
    let mut state = setup_test_state();
    let executor = ToolExecutor::new();

    // 1. Create review marker
    let marker_res = executor.execute(
        "manage_markers",
        &json!({
            "action": "create",
            "name": "Review Note",
            "startFrame": 30,
            "durationFrames": 15,
            "color": "#FF0055",
            "comment": "Check audio sync",
            "status": "open"
        }),
        &mut state,
    );
    assert!(!marker_res.is_error);
    assert_eq!(state.markers.len(), 1);
    let marker_id = state.markers[0].id.clone();

    // 2. Update marker
    let update_res = executor.execute(
        "manage_markers",
        &json!({
            "action": "update",
            "markerId": marker_id,
            "status": "resolved"
        }),
        &mut state,
    );
    assert!(!update_res.is_error);
    assert_eq!(state.markers[0].status, "resolved");

    // 3. Add text overlay
    let text_res = executor.execute(
        "add_texts",
        &json!({
            "trackIndex": 0,
            "texts": [
                {
                    "text": "Opening Title",
                    "startFrame": 0,
                    "durationFrames": 60,
                    "fontSize": 48.0,
                    "color": "#FFCC00"
                }
            ]
        }),
        &mut state,
    );
    assert!(!text_res.is_error);
    assert_eq!(state.editor.timeline().tracks[0].clips.len(), 1);
    assert_eq!(
        state.editor.timeline().tracks[0].clips[0].text_content.as_deref(),
        Some("Opening Title")
    );
}

#[test]
fn test_tool_execution_color_and_effects() {
    let mut state = setup_test_state();
    let executor = ToolExecutor::new();

    let _ = executor.execute(
        "add_clips",
        &json!({
            "trackIndex": 0,
            "clips": [
                { "mediaRef": "sample-media-1", "startFrame": 0, "durationFrames": 100 }
            ]
        }),
        &mut state,
    );
    let clip_id = state.editor.timeline().tracks[0].clips[0].id.clone();

    // Apply color
    let color_res = executor.execute(
        "apply_color",
        &json!({
            "clipId": clip_id,
            "exposure": 0.5,
            "contrast": 1.2
        }),
        &mut state,
    );
    assert!(!color_res.is_error);
    assert!(state.editor.timeline().tracks[0].clips[0]
        .effects
        .as_ref()
        .map(|fxs| fxs.iter().any(|e| e.effect_type == "color.grade"))
        .unwrap_or(false));

    // Apply effect
    let fx_res = executor.execute(
        "apply_effect",
        &json!({
            "clipId": clip_id,
            "effectType": "gaussian_blur",
            "params": { "radius": 12.0 }
        }),
        &mut state,
    );
    assert!(!fx_res.is_error);
    assert!(state.editor.timeline().tracks[0].clips[0]
        .effects
        .as_ref()
        .map(|fxs| fxs.iter().any(|e| e.effect_type == "gaussian_blur"))
        .unwrap_or(false));
}
