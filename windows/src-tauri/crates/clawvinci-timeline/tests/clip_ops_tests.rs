// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only

use clawvinci_model::clip_type::ClipType;
use clawvinci_model::keyframe::{Interpolation, Keyframe, KeyframeTrack};
use clawvinci_model::timeline::{Clip, Timeline, Track};
use clawvinci_timeline::clip_ops::split_values;
use clawvinci_timeline::editor::TimelineEditor;
use clawvinci_timeline::error::TimelineError;
use clawvinci_timeline::ripple::TrimEdge;
use uuid::Uuid;

#[test]
fn test_split_clip_with_keyframes() {
    let mut clip = Clip::new("video-asset", 0, 100);
    let mut op_track = KeyframeTrack::default();
    op_track.upsert(Keyframe::new(0, 0.0, Interpolation::Linear));
    op_track.upsert(Keyframe::new(40, 0.5, Interpolation::Smooth));
    op_track.upsert(Keyframe::new(80, 1.0, Interpolation::Smooth));
    clip.opacity_track = Some(op_track);

    let (left, right) = split_values(&clip, 50).expect("Split values should succeed");

    // Left clip [0, 50)
    assert_eq!(left.start_frame, 0);
    assert_eq!(left.duration_frames, 50);
    let left_op = left.opacity_track.unwrap();
    // Keyframes at 0, 40, and split boundary at 50
    assert_eq!(left_op.keyframes.len(), 3);
    assert_eq!(left_op.keyframes[0].frame, 0);
    assert_eq!(left_op.keyframes[1].frame, 40);
    assert_eq!(left_op.keyframes[2].frame, 50);

    // Right clip [50, 100) -> rebased to 0..50
    assert_eq!(right.start_frame, 50);
    assert_eq!(right.duration_frames, 50);
    let right_op = right.opacity_track.unwrap();
    // Rebased keyframes: boundary at 0, keyframe at 80 rebased to 80 - 50 = 30
    assert_eq!(right_op.keyframes.len(), 2);
    assert_eq!(right_op.keyframes[0].frame, 0);
    assert_eq!(right_op.keyframes[1].frame, 30);
}

#[test]
fn test_split_linked_clips() {
    let mut timeline = Timeline::new(30, 1920, 1080);
    let link_id = Uuid::new_v4().to_string();

    let mut v_track = Track::new(ClipType::Video);
    let mut v_clip = Clip::new("v-asset", 0, 100);
    v_clip.link_group_id = Some(link_id.clone());
    let v_id = v_clip.id.clone();
    v_track.clips.push(v_clip);

    let mut a_track = Track::new(ClipType::Audio);
    let mut a_clip = Clip::new("a-asset", 0, 100);
    a_clip.link_group_id = Some(link_id);
    a_track.clips.push(a_clip);

    timeline.tracks = vec![v_track, a_track];
    let mut editor = TimelineEditor::new(timeline);

    // Split video clip at frame 60; partner audio clip must also split
    let rights = editor.split_clip(&v_id, 60).expect("Split linked should succeed");
    assert_eq!(rights.len(), 2);

    let tl = editor.timeline();
    // Both tracks should have 2 clips each
    assert_eq!(tl.tracks[0].clips.len(), 2);
    assert_eq!(tl.tracks[1].clips.len(), 2);

    // Left halves still share original link group or have it
    let right_v = tl.tracks[0].clips.iter().find(|c| c.start_frame == 60).unwrap();
    let right_a = tl.tracks[1].clips.iter().find(|c| c.start_frame == 60).unwrap();

    // Right halves must share a new common link group!
    assert!(right_v.link_group_id.is_some());
    assert_eq!(right_v.link_group_id, right_a.link_group_id);
    assert_ne!(right_v.link_group_id, tl.tracks[0].clips[0].link_group_id);
}

#[test]
fn test_trim_and_slip_clip() {
    let mut timeline = Timeline::new(30, 1920, 1080);
    let mut track = Track::new(ClipType::Video);
    let mut clip = Clip::new("asset", 10, 80); // [10, 90)
    clip.trim_start_frame = 50;
    clip.trim_end_frame = 50;
    let cid = clip.id.clone();
    track.clips.push(clip);
    timeline.tracks.push(track);

    let mut editor = TimelineEditor::new(timeline);

    // Slip by 10 frames
    editor.slip_clip(&cid, 10).expect("Slip should succeed");
    let c = &editor.timeline().tracks[0].clips[0];
    assert_eq!(c.start_frame, 10);
    assert_eq!(c.duration_frames, 80);
    assert_eq!(c.trim_start_frame, 40); // 50 - 10
    assert_eq!(c.trim_end_frame, 60);   // 50 + 10

    // Trim right edge by 20 frames
    editor.trim_clip(&cid, TrimEdge::Right, 20).expect("Trim right should succeed");
    let c2 = &editor.timeline().tracks[0].clips[0];
    assert_eq!(c2.duration_frames, 100);
}

#[test]
fn test_move_clips_compatibility() {
    let mut timeline = Timeline::new(30, 1920, 1080);
    let mut v_track = Track::new(ClipType::Video);
    let v_clip = Clip::new("v-asset", 0, 50);
    let v_id = v_clip.id.clone();
    v_track.clips.push(v_clip);

    let a_track = Track::new(ClipType::Audio);
    timeline.tracks = vec![v_track, a_track];

    let mut editor = TimelineEditor::new(timeline);

    // Attempting to move video clip to audio track (track 1) must be rejected
    let err = editor.move_clips(&[(v_id.clone(), 1, 10)]);
    assert!(err.is_err());
    match err.unwrap_err() {
        TimelineError::TrackIncompatible { .. } => {}
        other => panic!("Expected TrackIncompatible, got {:?}", other),
    }

    // Moving on same video track or compatible visual track succeeds
    let ok = editor.move_clips(&[(v_id, 0, 20)]);
    assert!(ok.is_ok());
    assert_eq!(editor.timeline().tracks[0].clips[0].start_frame, 20);
}
