// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only

use clawvinci_export::fcpxml::FCPXMLExporter;
use clawvinci_export::options::{FCPXMLTarget, FCPXMLVersion};
use clawvinci_model::clip_type::ClipType;
use clawvinci_model::text_style::TextStyle;
use clawvinci_model::timeline::{Clip, Timeline, Track, Transform};
use std::collections::HashMap;

#[test]
fn test_fcpxml_exporter_renders_valid_fcpxml() {
    let mut timeline = Timeline::new(30, 1920, 1080);
    timeline.name = "FCPXML Project".to_string();

    // Track 1 (Primary Spine)
    let mut v1 = Track::new(ClipType::Video);
    v1.name = Some("V1".to_string());
    let mut c1 = Clip::new("shot-1", 0, 60);
    c1.transform = Transform {
        width: 1.1,
        height: 1.1,
        center_x: 0.55,
        center_y: 0.5,
        ..Default::default()
    };
    v1.clips.push(c1);
    timeline.tracks.push(v1);

    // Track 2 (Overlay / Lane 1)
    let mut v2 = Track::new(ClipType::Text);
    v2.name = Some("V2".to_string());
    let mut c2 = Clip::new("text-1", 15, 30);
    c2.media_type = ClipType::Text;
    c2.text_content = Some("Breaking News".to_string());
    c2.text_style = Some(TextStyle {
        font_family: "Roboto".to_string(),
        font_size: 64.0,
        ..Default::default()
    });
    v2.clips.push(c2);
    timeline.tracks.push(v2);

    // Track 3 (Audio / Lane -1)
    let mut a1 = Track::new(ClipType::Audio);
    a1.name = Some("A1".to_string());
    let mut c3 = Clip::new("voice-1", 0, 60);
    c3.media_type = ClipType::Audio;
    c3.volume = 0.8;
    a1.clips.push(c3);
    timeline.tracks.push(a1);

    let mut media_paths = HashMap::new();
    media_paths.insert("shot-1".to_string(), "C:/media/shot1.mp4".to_string());
    media_paths.insert("voice-1".to_string(), "C:/media/voice.wav".to_string());

    let xml = FCPXMLExporter::render(
        &timeline,
        &media_paths,
        FCPXMLVersion::V1_10,
        FCPXMLTarget::Resolve,
    );

    assert!(xml.starts_with("<?xml version=\"1.0\" encoding=\"UTF-8\"?>"));
    assert!(xml.contains("<fcpxml version=\"1.10\">"));
    assert!(xml.contains("<resources>"));
    assert!(xml.contains("<format id=\"r1\""));
    assert!(xml.contains("frameDuration=\"1/30s\""));
    assert!(xml.contains("<asset id=\"r2\""));
    assert!(xml.contains("file://localhost/C:/media/shot1.mp4"));
    assert!(xml.contains("<spine>"));
    assert!(xml.contains("shot-1"));
    assert!(xml.contains("adjust-transform"));
    assert!(xml.contains("<title"));
    assert!(xml.contains("Breaking News"));
    assert!(xml.contains("font=\"Roboto\""));
    assert!(xml.contains("adjust-volume"));
}
