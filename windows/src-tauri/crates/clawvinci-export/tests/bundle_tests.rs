// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only

use clawvinci_export::project_bundle::PalmierProjectExporter;
use clawvinci_model::manifest::{MediaManifest, MediaManifestEntry, MediaSource};
use clawvinci_model::project::ProjectFile;
use clawvinci_model::timeline::Timeline;
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
    let project_file = ProjectFile::from_timeline(timeline);

    let mut manifest = MediaManifest::default();
    manifest.entries.push(MediaManifestEntry {
        id: "media-entry-1".to_string(),
        name: "sample1.mp4".to_string(),
        source: MediaSource::External {
            path: dummy_media_1.to_string_lossy().to_string(),
        },
        duration_frames: 100,
        source_fps: Some(30.0),
        width: Some(1920),
        height: Some(1080),
        audio_channels: None,
        audio_sample_rate: None,
    });
    manifest.entries.push(MediaManifestEntry {
        id: "media-entry-2".to_string(),
        name: "sample2.wav".to_string(),
        source: MediaSource::External {
            path: dummy_media_2.to_string_lossy().to_string(),
        },
        duration_frames: 200,
        source_fps: None,
        width: None,
        height: None,
        audio_channels: Some(2),
        audio_sample_rate: Some(48000),
    });

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
