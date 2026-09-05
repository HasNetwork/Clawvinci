// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only

use clawvinci_media::*;
use std::path::Path;

#[tokio::test]
async fn test_thumbnail_cache_insertion_and_retrieval() {
    let cache = ThumbnailCache::new(3);

    assert_eq!(cache.len().await, 0);
    assert!(cache.is_empty().await);

    let opts1 = ThumbnailOptions::at_time(0.0).with_size(320, 180);
    let opts2 = ThumbnailOptions::at_time(1.0).with_size(320, 180);
    let opts3 = ThumbnailOptions::at_time(2.0).with_size(320, 180);
    let opts4 = ThumbnailOptions::at_time(3.0).with_size(320, 180);

    let data1 = vec![1, 2, 3, 4];
    let data2 = vec![5, 6, 7, 8];
    let data3 = vec![9, 10, 11, 12];
    let data4 = vec![13, 14, 15, 16];

    let asset_id = "asset-001";
    cache.insert_thumbnail(asset_id, &opts1, data1.clone()).await;
    cache.insert_thumbnail(asset_id, &opts2, data2.clone()).await;
    cache.insert_thumbnail(asset_id, &opts3, data3.clone()).await;

    assert_eq!(cache.len().await, 3);
    assert!(!cache.is_empty().await);

    // Verify all 3 are present
    assert_eq!(cache.get_thumbnail(asset_id, &opts1).await, Some(data1));
    assert_eq!(cache.get_thumbnail(asset_id, &opts2).await, Some(data2));
    assert_eq!(cache.get_thumbnail(asset_id, &opts3).await, Some(data3));

    // Insert 4th item -> capacity is 3, so oldest (opts1 was accessed, so opts2 or oldest) is evicted
    cache.insert_thumbnail(asset_id, &opts4, data4.clone()).await;
    assert_eq!(cache.len().await, 3);
    assert_eq!(cache.get_thumbnail(asset_id, &opts4).await, Some(data4));

    // Clear cache
    cache.clear().await;
    assert_eq!(cache.len().await, 0);
    assert!(cache.is_empty().await);
}

#[tokio::test]
async fn test_thumbnail_missing_file_error() {
    let ctx = FfmpegContext::with_paths("ffmpeg".into(), "ffprobe".into());
    let missing_path = Path::new("missing_video_for_thumb.mp4");
    let opts = ThumbnailOptions::default();

    let result = extract_thumbnail(&ctx, missing_path, &opts).await;
    assert!(result.is_err());
    match result.unwrap_err() {
        MediaError::InvalidInput(msg) => {
            assert!(msg.contains("Media file not found"));
        }
        other => panic!("Expected InvalidInput, got: {:?}", other),
    }
}
