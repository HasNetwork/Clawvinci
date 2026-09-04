// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only

use clawvinci_model::*;
use std::collections::HashMap;

#[test]
fn test_project_file_roundtrip() {
    let mut timeline = Timeline::new(30, 1920, 1080);
    timeline.name = "Action Scene".to_string();

    let mut video_track = Track::new(ClipType::Video);
    video_track.name = Some("V1".to_string());

    let mut clip = Clip::new("asset-001", 0, 150);
    clip.speed = 1.25;
    clip.volume = 0.8;
    clip.opacity = 0.95;
    clip.transform.rotation = 15.0;
    clip.transform.flip_horizontal = true;
    clip.crop = Crop::new(0.05, 0.05, 0.05, 0.05);
    clip.blend_mode = Some(BlendMode::Overlay);

    // Keyframe tracks
    let mut opacity_track = KeyframeTrack::default();
    opacity_track.upsert(Keyframe::new(0, 0.0, Interpolation::Linear));
    opacity_track.upsert(Keyframe::new(30, 1.0, Interpolation::Smooth));
    clip.opacity_track = Some(opacity_track);

    let mut position_track = KeyframeTrack::default();
    position_track.upsert(Keyframe::new(0, AnimPair::new(0.1, 0.2), Interpolation::Smooth));
    position_track.upsert(Keyframe::new(60, AnimPair::new(0.5, 0.5), Interpolation::Hold));
    clip.position_track = Some(position_track);

    // Effect
    let mut effect = Effect::make("blur.gaussian", &[("radius", 12.5)]);
    effect.enabled = true;
    clip.effects = Some(vec![effect]);

    video_track.clips.push(clip);

    let mut marker = TimelineMarker::new("Important cut", 45);
    marker.duration_frames = 10;
    marker.comment = "Check color grading here".to_string();
    marker.status = MarkerStatus::Review;
    timeline.markers.push(marker);

    timeline.tracks.push(video_track);

    let mut project = ProjectFile::new(vec![timeline]);
    let mut view_states = HashMap::new();
    view_states.insert(
        project.timelines[0].id.clone(),
        TimelineViewState {
            playhead_frame: 45,
            zoom_scale: 1.5,
            scroll_offset_x: 120.0,
        },
    );
    project.view_states = Some(view_states);

    // Multicam
    let mut mc = MulticamSource::new("Interview A/B");
    mc.members.push(MulticamMember::new(
        "asset-cam-a",
        MemberKind::Both,
        "Camera A",
    ));
    let mut member_b = MulticamMember::new("asset-cam-b", MemberKind::Angle, "Camera B");
    member_b.sync.offset_seconds = 0.45;
    member_b.sync.confidence = 0.98;
    mc.members.push(member_b);
    project.multicam_groups = Some(vec![mc]);

    // Speaker
    project.speakers = Some(vec![SpeakerRegistryEntry::new(
        1,
        "Host",
        vec![0.2, 0.5, 0.9, 1.0],
        vec![0.1, 0.2, 0.3],
    )]);

    // Encode to JSON
    let json_bytes = project.encode().expect("failed to encode project file");
    let json_str = String::from_utf8(json_bytes.clone()).expect("invalid utf8");

    // Deserialize back
    let decoded = ProjectFile::decode(&json_bytes).expect("failed to decode project file");

    assert_eq!(decoded.timelines.len(), 1);
    assert_eq!(decoded.timelines[0].name, "Action Scene");
    assert_eq!(decoded.timelines[0].tracks.len(), 1);
    assert_eq!(decoded.timelines[0].tracks[0].clips.len(), 1);

    let decoded_clip = &decoded.timelines[0].tracks[0].clips[0];
    assert_eq!(decoded_clip.media_ref, "asset-001");
    assert_eq!(decoded_clip.blend_mode, Some(BlendMode::Overlay));
    assert_eq!(decoded_clip.transform.rotation, 15.0);
    assert!(decoded_clip.transform.flip_horizontal);
    assert_eq!(decoded_clip.crop.left, 0.05);

    // Verify keyframes
    let ot = decoded_clip.opacity_track.as_ref().unwrap();
    assert_eq!(ot.keyframes.len(), 2);
    assert_eq!(ot.keyframes[0].value, 0.0);
    assert_eq!(ot.keyframes[1].value, 1.0);

    // Verify multicam and speakers
    let decoded_mc = decoded.multicam_groups.as_ref().unwrap();
    assert_eq!(decoded_mc[0].name, "Interview A/B");
    assert_eq!(decoded_mc[0].members.len(), 2);

    let decoded_sp = decoded.speakers.as_ref().unwrap();
    assert_eq!(decoded_sp[0].name, "Host");

    // Verify JSON field names match Swift CodingKeys
    assert!(json_str.contains("\"timelines\":"));
    assert!(json_str.contains("\"activeTimelineId\":"));
    assert!(json_str.contains("\"multicamGroups\":"));
    assert!(json_str.contains("\"mediaRef\":"));
    assert!(json_str.contains("\"startFrame\":"));
    assert!(json_str.contains("\"durationFrames\":"));
    assert!(json_str.contains("\"flipHorizontal\":"));
}

#[test]
fn test_text_clip_style_roundtrip() {
    let mut style = TextStyle::default();
    style.font_name = "Outfit".to_string();
    style.font_size = 72.0;
    style.is_bold = true;
    style.alignment = TextAlignment::Left;
    style.color = Rgba::from_hex("#FF5500").unwrap();
    style.shadow.enabled = true;
    style.shadow.offset_y = -3.0;

    let json = serde_json::to_string(&style).expect("failed to serialize TextStyle");
    assert!(json.contains("\"fontName\":\"Outfit\""));
    assert!(json.contains("\"fontSize\":72.0"));
    assert!(json.contains("\"isBold\":true"));
    assert!(json.contains("\"alignment\":\"left\""));

    let decoded: TextStyle = serde_json::from_str(&json).expect("failed to deserialize TextStyle");
    assert_eq!(decoded.font_name, "Outfit");
    assert_eq!(decoded.font_size, 72.0);
    assert!(decoded.is_bold);
    assert_eq!(decoded.alignment, TextAlignment::Left);
    assert!(decoded.shadow.enabled);
}

#[test]
fn test_grade_and_hue_curves_roundtrip() {
    let mut grade = GradeCurve::default();
    grade.master.push(CurvePoint::new(0.0, 0.0));
    grade.master.push(CurvePoint::new(0.5, 0.6));
    grade.master.push(CurvePoint::new(1.0, 1.0));

    let json = serde_json::to_string(&grade).expect("failed to serialize GradeCurve");
    let decoded: GradeCurve = serde_json::from_str(&json).expect("failed to deserialize GradeCurve");
    assert_eq!(decoded.master.len(), 3);
    assert_eq!(GradeCurve::eval(&decoded.master, 0.25), 0.3);

    let mut hue = HueCurves::default();
    hue.hue_vs_sat.push(CurvePoint::new(0.2, 0.7));
    let hue_json = serde_json::to_string(&hue).expect("failed to serialize HueCurves");
    assert!(hue_json.contains("\"hueVsSat\":"));
    let decoded_hue: HueCurves = serde_json::from_str(&hue_json).expect("failed to deserialize HueCurves");
    assert_eq!(decoded_hue.hue_vs_sat.len(), 1);
}
