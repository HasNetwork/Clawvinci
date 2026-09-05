// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only

use clawvinci_model::clip_type::ClipType;
use clawvinci_model::timeline::{Clip, Timeline, Track};
use clawvinci_model::timeline_marker::TimelineMarker;
use clawvinci_timeline::editor::TimelineEditor;
use clawvinci_timeline::ripple::{FrameRange, RippleEngine};
use std::collections::HashSet;

#[test]
fn test_merge_ranges() {
    let input = vec![
        FrameRange::new(10, 20),
        FrameRange::new(15, 30),
        FrameRange::new(40, 50),
        FrameRange::new(50, 60),
    ];
    let merged = RippleEngine::merge_ranges(&input);
    assert_eq!(
        merged,
        vec![FrameRange::new(10, 30), FrameRange::new(40, 60)]
    );
}

#[test]
fn test_compute_ripple_shifts() {
    let mut clips = vec![
        Clip::new("c1", 0, 50),
        Clip::new("c2", 50, 50),
        Clip::new("c3", 100, 50),
        Clip::new("c4", 200, 50),
    ];
    let mut removed = HashSet::new();
    removed.insert(clips[1].id.clone()); // remove c2 [50, 100)

    let shifts = RippleEngine::compute_ripple_shifts(&clips, &removed);
    assert_eq!(shifts.len(), 2);
    // c3 was at 100 -> shifts left by 50 to 50
    assert_eq!(shifts[0].clip_id, clips[2].id);
    assert_eq!(shifts[0].new_start_frame, 50);
    // c4 was at 200 -> shifts left by 50 to 150
    assert_eq!(shifts[1].clip_id, clips[3].id);
    assert_eq!(shifts[1].new_start_frame, 150);
}

#[test]
fn test_compute_ripple_push() {
    let clips = vec![
        Clip::new("c1", 0, 50),
        Clip::new("c2", 50, 50),
        Clip::new("c3", 100, 50),
    ];
    let exclude = HashSet::new();
    let shifts = RippleEngine::compute_ripple_push(&clips, 50, 30, &exclude);
    assert_eq!(shifts.len(), 2);
    assert_eq!(shifts[0].clip_id, clips[1].id);
    assert_eq!(shifts[0].new_start_frame, 80);
    assert_eq!(shifts[1].clip_id, clips[2].id);
    assert_eq!(shifts[1].new_start_frame, 130);
}

#[test]
fn test_ripple_delete_sync_locked_tracks() {
    let mut timeline = Timeline::new(30, 1920, 1080);
    let mut v1 = Track::new(ClipType::Video);
    v1.sync_locked = true;
    let c1 = Clip::new("v-asset-1", 0, 50);
    let c2 = Clip::new("v-asset-2", 50, 50);
    let c3 = Clip::new("v-asset-3", 100, 50);
    let c2_id = c2.id.clone();
    v1.clips = vec![c1, c2, c3];

    let mut a1 = Track::new(ClipType::Audio);
    a1.sync_locked = true;
    let a_clip1 = Clip::new("a-asset-1", 0, 50);
    let a_clip2 = Clip::new("a-asset-2", 60, 80);
    let a_clip2_id = a_clip2.id.clone();
    a1.clips = vec![a_clip1, a_clip2];

    timeline.tracks = vec![v1, a1];
    let mut editor = TimelineEditor::new(timeline);

    let mut to_remove = HashSet::new();
    to_remove.insert(c2_id);

    let removed = editor.ripple_delete(&to_remove).expect("Ripple delete should succeed");
    assert_eq!(removed, 1);

    let tl = editor.timeline();
    // In V1, c2 was removed, c3 shifted from 100 to 50
    assert_eq!(tl.tracks[0].clips.len(), 2);
    assert_eq!(tl.tracks[0].clips[1].start_frame, 50);

    // In A1, because sync_locked is true, clips after frame 50 shifted left by 50 frames
    assert_eq!(tl.tracks[1].clips.len(), 2);
    let shifted_a_clip = tl.tracks[1].clips.iter().find(|c| c.id == a_clip2_id).unwrap();
    assert_eq!(shifted_a_clip.start_frame, 10); // 60 - 50 = 10
}

#[test]
fn test_ripple_insert() {
    let mut timeline = Timeline::new(30, 1920, 1080);
    let mut v1 = Track::new(ClipType::Video);
    v1.clips = vec![Clip::new("v1", 0, 100), Clip::new("v2", 100, 100)];
    timeline.tracks.push(v1);

    let mut editor = TimelineEditor::new(timeline);
    let inserted = Clip::new("inserted", 50, 60);

    editor.ripple_insert(0, inserted).expect("Ripple insert should succeed");
    let tl = editor.timeline();
    assert_eq!(tl.tracks[0].clips.len(), 3);
    // The clip at 100 should have shifted forward by 60 to 160
    assert_eq!(tl.tracks[0].clips[2].start_frame, 160);
}

#[test]
fn test_ripple_markers() {
    let markers = vec![
        TimelineMarker::new("Before", 20),
        TimelineMarker::new("Inside", 60),
        TimelineMarker::new("After", 120),
    ];
    let closed = vec![vec![FrameRange::new(50, 100)]];
    let rippled = RippleEngine::ripple_markers(&markers, &closed);

    // Marker inside [50, 100) is removed
    assert_eq!(rippled.len(), 2);
    assert_eq!(rippled[0].name, "Before");
    assert_eq!(rippled[0].start_frame, 20);
    assert_eq!(rippled[1].name, "After");
    assert_eq!(rippled[1].start_frame, 70); // 120 - 50 = 70
}
