// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only

use clawvinci_export::project_bundle::PalmierProjectExporter;
use clawvinci_model::{
    ClipType, MediaManifest, MediaManifestEntry, MediaSource, ProjectFile, Timeline,
};
use tokio_util::sync::CancellationToken;

#[tokio::test]
async fn test_palmier_project_exporter_bundle_roundtrip() {
    let temp_root = std::env::temp_dir().join(format!("clawvinci_test_{}", uuid::Uuid::new_v4()));
    let src_media_dir = temp_root.join("src_media");
    let dest_bundle_dir = temp_root.join("ExportedProject.palmier");

    tokio::fs::create_dir_all(&src_media_dir).await.unwrap();

    let dummy_media_1 = src_media_dir.join("sample1.mp4");
    let dummy_media_2 = src_media_dir.join("sample2.wav");
    tokio::fs::write(&dummy_media_1, b"FAKE_VIDEO_BYTES_12345").await.unwrap();
    tokio::fs::write(&dummy_media_2, b"FAKE_AUDIO_BYTES_67890").await.unwrap();

    let mut timeline = Timeline::new(30, 1920, 1080);
    timeline.name = "Bundle Test".to_string();
    let project_file = ProjectFile::new(vec![timeline]);

    let mut manifest = MediaManifest::default();
    manifest.entries.push(MediaManifestEntry::new(
        "media-entry-1",
        "sample1.mp4",
        ClipType::Video,
        MediaSource::External {
            absolute_path: dummy_media_1.to_string_lossy().to_string(),
        },
        10.0,
    ));
    manifest.entries.push(MediaManifestEntry::new(
        "media-entry-2",
        "sample2.wav",
        ClipType::Audio,
        MediaSource::External {
            absolute_path: dummy_media_2.to_string_lossy().to_string(),
        },
        5.0,
    ));

    let cancel_token = CancellationToken::new();
    let report = PalmierProjectExporter::export(
        &project_file,
        &manifest,
        None,
        &dest_bundle_dir,
        cancel_token,
        None,
    )
    .await
    .unwrap();

    assert_eq!(report.collected.len(), 2);
    assert_eq!(report.missing.len(), 0);
    assert!(report.total_bytes > 0);

    // Verify bundle contents
    assert!(dest_bundle_dir.join("project.json").exists());
    assert!(dest_bundle_dir.join("manifest.json").exists());
    assert!(dest_bundle_dir.join("media").is_dir());

    let manifest_content = tokio::fs::read_to_string(dest_bundle_dir.join("manifest.json"))
        .await
        .unwrap();
    let read_manifest: MediaManifest = serde_json::from_str(&manifest_content).unwrap();

    for entry in &read_manifest.entries {
        match &entry.source {
            MediaSource::Project { relative_path } => {
                assert!(relative_path.starts_with("media/"));
                let bundled_file = dest_bundle_dir.join(relative_path);
                assert!(bundled_file.exists());
            }
            _ => panic!("Expected MediaSource::Project after bundle export"),
        }
    }

    // Cleanup
    let _ = tokio::fs::remove_dir_all(&temp_root).await;
}

#[tokio::test]
async fn test_palmier_format_fixture_decode_roundtrip() {
    let temp_root =
        std::env::temp_dir().join(format!("clawvinci_fixture_{}", uuid::Uuid::new_v4()));
    let bundle_dir = temp_root.join("Fixture.palmier");
    let media_dir = bundle_dir.join("media");
    tokio::fs::create_dir_all(&media_dir).await.unwrap();

    // Build a ProjectFile with a non-trivial timeline
    let mut timeline = Timeline::new(24, 3840, 2160);
    timeline.name = "Fixture Timeline".to_string();

    let clip = clawvinci_model::timeline::Clip::new("asset-001", 0, 120);
    timeline.tracks[0].clips.push(clip);

    let project_file = ProjectFile::new(vec![timeline]);

    // Build a MediaManifest with two entries (video + audio)
    let mut manifest = MediaManifest::default();
    manifest.entries.push(MediaManifestEntry::new(
        "asset-001",
        "Intro.mp4",
        ClipType::Video,
        MediaSource::Project {
            relative_path: "media/Intro.mp4".to_string(),
        },
        5.0,
    ));
    manifest.entries.push(MediaManifestEntry::new(
        "asset-002",
        "Narration.wav",
        ClipType::Audio,
        MediaSource::Project {
            relative_path: "media/Narration.wav".to_string(),
        },
        12.5,
    ));

    // Serialize to .palmier bundle files
    let project_json = serde_json::to_string_pretty(&project_file).unwrap();
    let manifest_json = serde_json::to_string_pretty(&manifest).unwrap();

    tokio::fs::write(bundle_dir.join("project.json"), &project_json)
        .await
        .unwrap();
    tokio::fs::write(bundle_dir.join("manifest.json"), &manifest_json)
        .await
        .unwrap();

    // Decode back from fixture files
    let decoded_project_str = tokio::fs::read_to_string(bundle_dir.join("project.json"))
        .await
        .unwrap();
    let decoded_project: ProjectFile = serde_json::from_str(&decoded_project_str).unwrap();

    let decoded_manifest_str = tokio::fs::read_to_string(bundle_dir.join("manifest.json"))
        .await
        .unwrap();
    let decoded_manifest: MediaManifest = serde_json::from_str(&decoded_manifest_str).unwrap();

    // Verify decoded objects match originals
    assert_eq!(decoded_project, project_file);
    assert_eq!(decoded_manifest.entries.len(), manifest.entries.len());

    for (original, decoded) in manifest.entries.iter().zip(decoded_manifest.entries.iter()) {
        assert_eq!(original.id, decoded.id);
        assert_eq!(original.name, decoded.name);
        assert_eq!(original.clip_type, decoded.clip_type);
        assert_eq!(original.duration, decoded.duration);
        assert_eq!(original.source, decoded.source);
    }

    // Verify timeline data integrity
    assert_eq!(decoded_project.timelines.len(), 1);
    let decoded_tl = &decoded_project.timelines[0];
    assert_eq!(decoded_tl.name, "Fixture Timeline");
    assert_eq!(decoded_tl.fps, 24);
    assert_eq!(decoded_tl.width, 3840);
    assert_eq!(decoded_tl.height, 2160);
    assert_eq!(decoded_tl.tracks[0].clips.len(), 1);
    assert_eq!(decoded_tl.tracks[0].clips[0].media_ref, "asset-001");
    assert_eq!(decoded_tl.tracks[0].clips[0].start_frame, 0);
    assert_eq!(decoded_tl.tracks[0].clips[0].duration_frames, 120);

    let _ = tokio::fs::remove_dir_all(&temp_root).await;
}
