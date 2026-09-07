// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only

use clawvinci_export::xml::{format_smpte_timecode, XMLExporter};
use clawvinci_model::clip_type::ClipType;
use clawvinci_model::timeline::{Clip, Crop, Timeline, Track, Transform};
use std::collections::HashMap;

#[test]
fn test_smpte_timecode_formatting() {
    // 24 fps Non-drop-frame
    assert_eq!(format_smpte_timecode(0, 24, false), "00:00:00:00");
    assert_eq!(format_smpte_timecode(24, 24, false), "00:00:01:00");
    assert_eq!(format_smpte_timecode(24 * 60, 24, false), "00:01:00:00");
    assert_eq!(format_smpte_timecode(24 * 3600, 24, false), "01:00:00:00");

    // 30 fps Drop-frame
    let tc_df = format_smpte_timecode(30, 30, true);
    assert!(tc_df.contains(';'));
}

#[test]
fn test_xml_exporter_renders_valid_xmeml() {
    let mut timeline = Timeline::new(30, 1920, 1080);
    timeline.name = "Test Sequence".to_string();

    let mut video_track = Track::new(ClipType::Video);
    video_track.name = Some("Video Track".to_string());
    let mut clip1 = Clip::new("media-1", 0, 90);
    clip1.transform = Transform {
        width: 1.25,
        rotation: 15.0,
        center_x: 0.6,
        center_y: 0.4,
        ..Default::default()
    };
    clip1.crop = Crop::new(0.05, 0.1, 0.05, 0.1);
    clip1.opacity = 0.85;
    video_track.clips.push(clip1);
    timeline.tracks.push(video_track);

    let mut audio_track = Track::new(ClipType::Audio);
    audio_track.name = Some("Audio Track".to_string());
    let mut audio_clip = Clip::new("media-audio", 0, 120);
    audio_clip.volume = 0.5;
    audio_track.clips.push(audio_clip);
    timeline.tracks.push(audio_track);

    let mut media_paths = HashMap::new();
    media_paths.insert("media-1".to_string(), "C:/media/landscape.mp4".to_string());
    media_paths.insert("media-audio".to_string(), "C:/media/music.wav".to_string());

    let xml = XMLExporter::render(&timeline, &media_paths);

    assert!(xml.starts_with("<?xml version=\"1.0\" encoding=\"UTF-8\"?>"));
    assert!(xml.contains("<!DOCTYPE xmeml>"));
    assert!(xml.contains("<xmeml version=\"4\">"));
    assert!(xml.contains("<name>Test Sequence</name>"));
    assert!(xml.contains("<timebase>30</timebase>"));
    assert!(xml.contains("<width>1920</width>"));
    assert!(xml.contains("<height>1080</height>"));
    assert!(xml.contains("media-1"));
    assert!(xml.contains("media-audio"));
    assert!(xml.contains("<effectid>basic</effectid>"));
    assert!(xml.contains("<effectid>crop</effectid>"));
    assert!(xml.contains("<effectid>opacity</effectid>"));
    assert!(xml.contains("<effectid>audiolevels</effectid>"));
    assert!(xml.contains("file://localhost/C:/media/landscape.mp4"));
}
