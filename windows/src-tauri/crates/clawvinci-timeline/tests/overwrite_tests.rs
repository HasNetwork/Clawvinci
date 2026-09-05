// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only

use clawvinci_model::clip_type::ClipType;
use clawvinci_model::timeline::{Clip, Timeline, Track};
use clawvinci_timeline::editor::TimelineEditor;
use clawvinci_timeline::overwrite::{OverwriteAction, OverwriteEngine};

#[test]
fn test_compute_overwrite_actions() {
    let clip = Clip::new("asset", 100, 100); // [100, 200)

    // 1. Fully covers: [90, 210)
    let actions = OverwriteEngine::compute_overwrite(&[clip.clone()], 90, 210);
    assert_eq!(actions, vec![OverwriteAction::Remove { clip_id: clip.id.clone() }]);

    // 2. Overlaps left: [80, 140) -> Trims end of left portion?
    // region is [80, 140). Clip is [100, 200).
    // cs (100) >= regionStart (80), and ce (200) > regionEnd (140).
    // That overlaps the clip's left side, so clip's start is trimmed: TrimStart!
    let actions2 = OverwriteEngine::compute_overwrite(&[clip.clone()], 80, 140);
    assert_eq!(actions2.len(), 1);
    match actions2[0].clone() {
        OverwriteAction::TrimStart { clip_id, new_start_frame, new_duration, .. } => {
            assert_eq!(clip_id, clip.id);
            assert_eq!(new_start_frame, 140);
            assert_eq!(new_duration, 60);
        }
        other => panic!("Expected TrimStart, got {:?}", other),
    }

    // 3. Overlaps right side of clip: region [160, 220)
    // cs = 100, ce = 200. regionStart = 160, regionEnd = 220.
    // cs < regionStart, ce <= regionEnd -> TrimEnd
    let actions3 = OverwriteEngine::compute_overwrite(&[clip.clone()], 160, 220);
    assert_eq!(actions3.len(), 1);
    match actions3[0].clone() {
        OverwriteAction::TrimEnd { clip_id, new_duration } => {
            assert_eq!(clip_id, clip.id);
            assert_eq!(new_duration, 60); // 160 - 100 = 60
        }
        other => panic!("Expected TrimEnd, got {:?}", other),
    }

    // 4. Middle punch: region [130, 170) -> Split
    let actions4 = OverwriteEngine::compute_overwrite(&[clip.clone()], 130, 170);
    assert_eq!(actions4.len(), 1);
    match actions4[0].clone() {
        OverwriteAction::Split { clip_id, left_duration, right_start_frame, right_duration, .. } => {
            assert_eq!(clip_id, clip.id);
            assert_eq!(left_duration, 30); // 130 - 100 = 30
            assert_eq!(right_start_frame, 170);
            assert_eq!(right_duration, 30); // 200 - 170 = 30
        }
        other => panic!("Expected Split, got {:?}", other),
    }
}

#[test]
fn test_overwrite_middle_split_execution_and_undo() {
    let mut timeline = Timeline::new(30, 1920, 1080);
    let mut track = Track::new(ClipType::Video);
    let base_clip = Clip::new("base-asset", 0, 200); // [0, 200)
    track.clips.push(base_clip);
    timeline.tracks.push(track);

    let initial = timeline.clone();
    let mut editor = TimelineEditor::new(timeline);

    // Overwrite middle [50, 150) with a new clip
    let mut new_clip = Clip::new("new-asset", 50, 100);
    new_clip.speed = 1.0;

    editor.overwrite_clip(0, new_clip.clone()).expect("Overwrite should succeed");

    let tl = editor.timeline();
    // Should now have 3 clips on track 0:
    // Left fragment: [0, 50)
    // New clip: [50, 150)
    // Right fragment: [150, 200)
    assert_eq!(tl.tracks[0].clips.len(), 3);
    assert_eq!(tl.tracks[0].clips[0].start_frame, 0);
    assert_eq!(tl.tracks[0].clips[0].duration_frames, 50);

    assert_eq!(tl.tracks[0].clips[1].id, new_clip.id);
    assert_eq!(tl.tracks[0].clips[1].start_frame, 50);
    assert_eq!(tl.tracks[0].clips[1].duration_frames, 100);

    assert_eq!(tl.tracks[0].clips[2].start_frame, 150);
    assert_eq!(tl.tracks[0].clips[2].duration_frames, 50);

    // Exact undo restoration: must restore the single original [0, 200) clip
    editor.undo().expect("Undo should succeed");
    assert_eq!(*editor.timeline(), initial);
    assert_eq!(editor.timeline().tracks[0].clips.len(), 1);
    assert_eq!(editor.timeline().tracks[0].clips[0].duration_frames, 200);
}
