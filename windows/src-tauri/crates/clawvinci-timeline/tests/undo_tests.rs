// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only

use clawvinci_model::clip_type::ClipType;
use clawvinci_model::timeline::{Clip, Timeline, Track};
use clawvinci_timeline::editor::TimelineEditor;
use clawvinci_timeline::error::TimelineError;
use clawvinci_timeline::ripple::TrimEdge;
use std::collections::HashSet;

fn create_sample_timeline() -> Timeline {
    let mut timeline = Timeline::new(30, 1920, 1080);
    let mut v1 = Track::new(ClipType::Video);
    v1.name = Some("V1".to_string());
    v1.clips.push(Clip::new("asset-1", 0, 100));
    v1.clips.push(Clip::new("asset-2", 150, 90));

    let mut a1 = Track::new(ClipType::Audio);
    a1.name = Some("A1".to_string());
    a1.clips.push(Clip::new("asset-audio-1", 0, 100));

    timeline.tracks.push(v1);
    timeline.tracks.push(a1);
    timeline
}

#[test]
fn test_undo_redo_exact_roundtrip() {
    let initial = create_sample_timeline();
    let mut editor = TimelineEditor::new(initial.clone());

    assert!(!editor.can_undo());
    assert!(!editor.can_redo());

    // Perform an edit: split clip
    let clip_id = initial.tracks[0].clips[0].id.clone();
    let split_res = editor.split_clip(&clip_id, 40);
    assert!(split_res.is_ok());

    assert!(editor.can_undo());
    assert_eq!(editor.undo_action_name(), Some("Split Clip"));
    assert_eq!(editor.timeline().tracks[0].clips.len(), 3);

    // Undo the edit
    let action = editor.undo().expect("Undo should succeed");
    assert_eq!(action, "Split Clip");
    assert_eq!(*editor.timeline(), initial, "Undone timeline must match initial state exactly");
    assert!(editor.can_redo());
    assert_eq!(editor.redo_action_name(), Some("Split Clip"));

    // Redo the edit
    let redo_action = editor.redo().expect("Redo should succeed");
    assert_eq!(redo_action, "Split Clip");
    assert_eq!(editor.timeline().tracks[0].clips.len(), 3);

    // Undo again to verify multiple passes
    editor.undo().expect("Undo should succeed again");
    assert_eq!(*editor.timeline(), initial);
}

#[test]
fn test_nested_transaction_coalescing() {
    let initial = create_sample_timeline();
    let mut editor = TimelineEditor::new(initial.clone());

    // Perform a compound operation wrapped in perform
    let result = editor.perform("Batch Trimming", |tl| {
        let cid = tl.tracks[0].clips[0].id.clone();
        clawvinci_timeline::clip_ops::trim_clip(tl, &cid, TrimEdge::Right, 20)?;
        let cid2 = tl.tracks[0].clips[1].id.clone();
        clawvinci_timeline::clip_ops::trim_clip(tl, &cid2, TrimEdge::Left, 10)?;
        Ok(())
    });
    assert!(result.is_ok());

    // Even though two clips were trimmed, there must be exactly 1 undo entry
    assert_eq!(editor.undo_stack().undo_count(), 1);
    assert_eq!(editor.undo_action_name(), Some("Batch Trimming"));

    editor.undo().expect("Undo should succeed");
    assert_eq!(*editor.timeline(), initial);
    assert_eq!(editor.undo_stack().undo_count(), 0);
}

#[test]
fn test_noop_and_failure_do_not_record_undo() {
    let initial = create_sample_timeline();
    let mut editor = TimelineEditor::new(initial.clone());

    // 1. Failed operation: split at out-of-bounds frame
    let res = editor.split_clip("nonexistent-clip", 50);
    assert!(res.is_err());
    assert_eq!(editor.undo_stack().undo_count(), 0);
    assert_eq!(*editor.timeline(), initial);

    // 2. No-op mutation: trimming by delta 0
    let cid = initial.tracks[0].clips[0].id.clone();
    let res2 = editor.trim_clip(&cid, TrimEdge::Right, 0);
    assert!(res2.is_ok());
    // Since state did not change, no undo entry was committed
    assert_eq!(editor.undo_stack().undo_count(), 0);

    // 3. Removing empty set of clips
    let empty_set = HashSet::new();
    let res3 = editor.remove_clips(&empty_set);
    assert!(res3.is_err()); // Returns NoOp
    assert_eq!(editor.undo_stack().undo_count(), 0);
}

#[test]
fn test_new_mutation_clears_redo_stack() {
    let initial = create_sample_timeline();
    let mut editor = TimelineEditor::new(initial);

    let cid = editor.timeline().tracks[0].clips[0].id.clone();
    editor.trim_clip(&cid, TrimEdge::Right, 10).unwrap();
    assert_eq!(editor.undo_stack().undo_count(), 1);

    editor.undo().unwrap();
    assert_eq!(editor.undo_stack().redo_count(), 1);

    // Perform a brand new mutation
    let cid2 = editor.timeline().tracks[0].clips[1].id.clone();
    editor.trim_clip(&cid2, TrimEdge::Left, 5).unwrap();

    // Redo stack must now be empty
    assert_eq!(editor.undo_stack().redo_count(), 0);
    assert!(!editor.can_redo());
}

#[test]
fn test_without_undo_disables_recording() {
    let initial = create_sample_timeline();
    let mut editor = TimelineEditor::new(initial);

    let cid = editor.timeline().tracks[0].clips[0].id.clone();
    editor.undo_stack_mut().without_undo(|_| {
        // Internal edit without recording
    });

    let res = editor.undo_stack_mut().without_undo(|stack| {
        stack.begin_transaction("Hidden", editor.timeline());
        let _ = clawvinci_timeline::clip_ops::trim_clip(editor.timeline_mut(), &cid, TrimEdge::Right, 15);
        stack.commit_transaction(editor.timeline())
    });

    assert!(!res); // Did not record
    assert_eq!(editor.undo_stack().undo_count(), 0);
    assert!(!editor.can_undo());
}
