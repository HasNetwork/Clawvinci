// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only

use clawvinci_model::*;
use std::path::PathBuf;

#[test]
fn test_media_source_json_shape() {
    let project_source = MediaSource::Project {
        relative_path: "media/interview.mp4".to_string(),
    };
    let json = serde_json::to_string(&project_source).expect("serialize project source");
    assert_eq!(json, r#"{"project":{"relativePath":"media/interview.mp4"}}"#);

    let decoded: MediaSource = serde_json::from_str(&json).expect("deserialize project source");
    assert_eq!(decoded, project_source);

    let external_source = MediaSource::External {
        absolute_path: "/Volumes/External/broll.mov".to_string(),
    };
    let ext_json = serde_json::to_string(&external_source).expect("serialize external source");
    assert_eq!(ext_json, r#"{"external":{"absolutePath":"/Volumes/External/broll.mov"}}"#);

    let decoded_ext: MediaSource = serde_json::from_str(&ext_json).expect("deserialize external source");
    assert_eq!(decoded_ext, external_source);
}

#[test]
fn test_media_manifest_roundtrip() {
    let mut manifest = MediaManifest::default();
    manifest.folders.push(MediaFolder::new("Interviews", None));
    let folder_id = manifest.folders[0].id.clone();

    let mut entry = MediaManifestEntry::new(
        "entry-1",
        "A-Cam Interview",
        ClipType::Video,
        MediaSource::Project {
            relative_path: "media/a_cam.mov".to_string(),
        },
        120.5,
    );
    entry.folder_id = Some(folder_id);
    entry.source_width = Some(3840);
    entry.source_height = Some(2160);
    entry.source_fps = Some(23.976);
    entry.has_audio = Some(true);
    manifest.entries.push(entry);

    let json = serde_json::to_string_pretty(&manifest).expect("serialize manifest");
    assert!(json.contains("\"entries\":"));
    assert!(json.contains("\"folders\":"));
    assert!(json.contains("\"sourceFPS\": 23.976"));
    assert!(json.contains("\"relativePath\": \"media/a_cam.mov\""));

    let decoded: MediaManifest = serde_json::from_str(&json).expect("deserialize manifest");
    assert_eq!(decoded.entries.len(), 1);
    assert_eq!(decoded.folders.len(), 1);
    assert_eq!(decoded.entries[0].name, "A-Cam Interview");
    assert_eq!(decoded.entries[0].duration, 120.5);
    assert_eq!(decoded.entries[0].source_width, Some(3840));
}

#[test]
fn test_media_resolver() {
    let mut manifest = MediaManifest::default();
    manifest.entries.push(MediaManifestEntry::new(
        "asset-1",
        "Clip 1",
        ClipType::Video,
        MediaSource::Project {
            relative_path: "media/clip1.mp4".to_string(),
        },
        10.0,
    ));
    manifest.entries.push(MediaManifestEntry::new(
        "asset-2",
        "Clip 2",
        ClipType::Audio,
        MediaSource::External {
            absolute_path: "D:/Media/music.wav".to_string(),
        },
        30.0,
    ));

    let project_dir = PathBuf::from("D:/Projects/MyProject.palmier");
    let resolver = MediaResolver::new(manifest, Some(project_dir));

    assert_eq!(
        resolver.expected_path("asset-1"),
        Some(PathBuf::from("D:/Projects/MyProject.palmier/media/clip1.mp4"))
    );
    assert_eq!(
        resolver.expected_path("asset-2"),
        Some(PathBuf::from("D:/Media/music.wav"))
    );
    assert_eq!(resolver.display_name("asset-1"), "Clip 1");
    assert_eq!(resolver.display_name("non-existent"), "Offline");
}
