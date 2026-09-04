// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only

use clawvinci_model::*;

#[test]
fn test_legacy_bare_timeline_decode_fallback() {
    // Legacy projects stored a bare Timeline directly in project.json
    let legacy_timeline_json = r#"{
        "id": "legacy-timeline-42",
        "name": "Old Project",
        "fps": 24,
        "width": 3840,
        "height": 2160,
        "tracks": [
            {
                "id": "track-1",
                "type": "video",
                "muted": false,
                "hidden": false,
                "syncLocked": true,
                "clips": [
                    {
                        "id": "clip-1",
                        "mediaRef": "legacy-video-1",
                        "startFrame": 0,
                        "durationFrames": 240
                    }
                ]
            }
        ]
    }"#;

    let project = ProjectFile::decode(legacy_timeline_json.as_bytes())
        .expect("failed to decode legacy bare timeline");

    assert_eq!(project.timelines.len(), 1);
    assert_eq!(project.timelines[0].id, "legacy-timeline-42");
    assert_eq!(project.timelines[0].name, "Old Project");
    assert_eq!(project.timelines[0].fps, 24);
    assert_eq!(project.timelines[0].width, 3840);
    assert_eq!(project.timelines[0].height, 2160);
    assert_eq!(project.active_timeline_id, Some("legacy-timeline-42".to_string()));
    assert_eq!(project.open_timeline_ids, Some(vec!["legacy-timeline-42".to_string()]));

    let track = &project.timelines[0].tracks[0];
    assert_eq!(track.track_type, ClipType::Video);
    assert_eq!(track.clips.len(), 1);
    assert_eq!(track.clips[0].media_ref, "legacy-video-1");
    assert_eq!(track.clips[0].duration_frames, 240);
}

#[test]
fn test_legacy_transform_xy_keys() {
    // Legacy Transform used x and y instead of centerX and centerY
    // centerX = x + width - 0.5; centerY = y + height - 0.5
    let transform_json = r#"{
        "x": 0.2,
        "y": 0.3,
        "width": 0.8,
        "height": 0.6,
        "rotation": 45.0
    }"#;

    let t: Transform = serde_json::from_str(transform_json).expect("failed to parse legacy transform");
    assert_eq!(t.width, 0.8);
    assert_eq!(t.height, 0.6);
    assert_eq!(t.rotation, 45.0);
    // 0.2 + 0.8 - 0.5 = 0.5
    assert!((t.center_x - 0.5).abs() < 1e-6);
    // 0.3 + 0.6 - 0.5 = 0.4
    assert!((t.center_y - 0.4).abs() < 1e-6);
}

#[test]
fn test_corrupt_json_fails_loudly() {
    let corrupt_json = b"{ this is corrupt json";
    let res = ProjectFile::decode(corrupt_json);
    assert!(res.is_err());
    match res {
        Err(ModelError::DecodeError(_)) => {}
        _ => panic!("expected DecodeError for corrupt JSON"),
    }
}

#[test]
fn test_empty_timelines_rejected() {
    let empty_project_json = r#"{"timelines": []}"#;
    let res = ProjectFile::decode(empty_project_json.as_bytes());
    assert!(res.is_err());
    match res {
        Err(ModelError::EmptyTimelines) => {}
        _ => panic!("expected EmptyTimelines error"),
    }
}
