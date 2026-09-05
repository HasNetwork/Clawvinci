// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only

use clawvinci_model::clip_type::ClipType;
use clawvinci_model::timeline::{Clip, Timeline, Track};
use clawvinci_timeline::editor::TimelineEditor;
use clawvinci_timeline::track_ops::TrackName;

#[test]
fn test_track_name_normalization() {
    // 1. None -> None
    assert_eq!(TrackName::normalized(None).unwrap(), None);

    // 2. Whitespace trimming -> None if empty
    assert_eq!(TrackName::normalized(Some("   ")).unwrap(), None);

    // 3. Valid name
    assert_eq!(
        TrackName::normalized(Some("  Dialogue Main  ")).unwrap(),
        Some("Dialogue Main".to_string())
    );

    // 4. Over 80 characters -> InvalidTrackName error
    let long_name = "a".repeat(81);
    assert!(TrackName::normalized(Some(&long_name)).is_err());

    // 5. Control characters or newlines -> InvalidTrackName error
    assert!(TrackName::normalized(Some("Line 1\nLine 2")).is_err());
    assert!(TrackName::normalized(Some("Line 1\rLine 2")).is_err());
    assert!(TrackName::normalized(Some("Track\u{0007}")).is_err());
}

#[test]
fn test_zone_partitioned_track_insertion() {
    let mut timeline = Timeline::new(30, 1920, 1080);
    // Initial: V1, V2, A1
    timeline.tracks.push(Track::new(ClipType::Video));
    timeline.tracks.push(Track::new(ClipType::Video));
    timeline.tracks.push(Track::new(ClipType::Audio));

    let mut editor = TimelineEditor::new(timeline);

    // Try inserting an Audio track at requested index 0 (top of timeline)
    // It must be clamped to the audio zone (at index 2)
    let audio_idx = editor.insert_track(0, ClipType::Audio).unwrap();
    assert_eq!(audio_idx, 2);
    assert_eq!(editor.timeline().tracks[audio_idx].track_type, ClipType::Audio);

    // Try inserting a Video track at requested index 10 (bottom of timeline)
    // It must be clamped to the visual zone (at index 2, right before audio)
    let video_idx = editor.insert_track(10, ClipType::Video).unwrap();
    assert_eq!(video_idx, 2);
    assert_eq!(editor.timeline().tracks[video_idx].track_type, ClipType::Video);

    // All visual tracks must strictly precede all audio tracks
    let first_audio = editor
        .timeline()
        .tracks
        .iter()
        .position(|t| t.track_type == ClipType::Audio)
        .unwrap();

    for (i, t) in editor.timeline().tracks.iter().enumerate() {
        if i < first_audio {
            assert!(t.track_type.is_visual(), "Track at index {} should be visual", i);
        } else {
            assert_eq!(t.track_type, ClipType::Audio, "Track at index {} should be audio", i);
        }
    }
}

#[test]
fn test_track_reordering_constrained_to_zones() {
    let mut timeline = Timeline::new(30, 1920, 1080);
    let mut v1 = Track::new(ClipType::Video);
    v1.name = Some("V1".to_string());
    let mut v2 = Track::new(ClipType::Video);
    v2.name = Some("V2".to_string());
    let mut a1 = Track::new(ClipType::Audio);
    a1.name = Some("A1".to_string());
    let mut a2 = Track::new(ClipType::Audio);
    a2.name = Some("A2".to_string());

    let v1_id = v1.id.clone();
    let a2_id = a2.id.clone();

    timeline.tracks = vec![v1, v2, a1, a2];
    let mut editor = TimelineEditor::new(timeline);

    // Reorder V1 (index 0) down to index 1 (within visual zone)
    let new_v1_idx = editor.reorder_track(&v1_id, 1).unwrap();
    assert_eq!(new_v1_idx, 1);
    assert_eq!(editor.timeline().tracks[1].name.as_deref(), Some("V1"));

    // Try dragging V1 into the audio zone (index 3) -> must clamp to index 1 (last visual slot)
    let clamped_v1_idx = editor.reorder_track(&v1_id, 3).unwrap();
    assert_eq!(clamped_v1_idx, 1);

    // Try dragging A2 (index 3) into the visual zone (index 0) -> must clamp to index 2 (first audio slot)
    let clamped_a2_idx = editor.reorder_track(&a2_id, 0).unwrap();
    assert_eq!(clamped_a2_idx, 2);
    assert_eq!(editor.timeline().tracks[2].name.as_deref(), Some("A2"));
}

#[test]
fn test_track_removal_and_undo() {
    let mut timeline = Timeline::new(30, 1920, 1080);
    let mut track = Track::new(ClipType::Video);
    track.clips.push(Clip::new("video-asset", 0, 120));
    let track_id = track.id.clone();
    timeline.tracks.push(track);

    let initial = timeline.clone();
    let mut editor = TimelineEditor::new(timeline);

    let removed = editor.remove_track(&track_id).expect("Remove track should succeed");
    assert_eq!(removed.id, track_id);
    assert_eq!(editor.timeline().tracks.len(), 0);

    // Undo track removal
    editor.undo().expect("Undo should succeed");
    assert_eq!(*editor.timeline(), initial);
    assert_eq!(editor.timeline().tracks.len(), 1);
    assert_eq!(editor.timeline().tracks[0].clips.len(), 1);
}
