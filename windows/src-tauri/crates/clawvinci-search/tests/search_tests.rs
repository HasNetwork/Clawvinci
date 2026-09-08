// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only

use clawvinci_search::coordinator::{
    SearchAssetInfo, SearchIndexCoordinator, SearchMediaType,
};
use clawvinci_search::transcription::{
    CutAggressiveness, CutWord, TranscriptCache, TranscriptSearch, TranscriptionResult,
    TranscriptionSegment, TranscriptionWord, WordCutPlanner,
};
use clawvinci_search::visual::{
    EmbeddingHeader, EmbeddingRow, EmbeddingStore, FrameSamplerOptions, FrameSamplerState,
    LumaGrid, MockVisualEmbedder, ModelSpec, VisualEmbedder, VisualModelLoader, VisualSearch,
    SAMPLER_VERSION,
};
use std::fs;
use std::sync::Arc;

#[test]
fn test_transcript_search_terms_and_matching() {
    let terms = TranscriptSearch::terms("Look at the budget, and timeline! ");
    assert_eq!(terms, vec!["look", "at", "the", "budget", "and", "timeline"]);

    let text = "We need to discuss the project budget before next Tuesday.";
    assert!(TranscriptSearch::matches(text, &["budget".to_string()]));
    assert!(TranscriptSearch::matches(
        text,
        &["project".to_string(), "tuesday".to_string()]
    ));
    assert!(!TranscriptSearch::matches(text, &["unrelated".to_string()]));

    let sample_transcript = TranscriptionResult::new(
        "Complete video editing overview.",
        Some("en".into()),
        vec![
            TranscriptionWord::new("Discussing", Some(0.0), Some(0.5)),
            TranscriptionWord::new("the", Some(0.5), Some(0.7)),
            TranscriptionWord::new("budget", Some(0.7), Some(1.2)),
        ],
        vec![
            TranscriptionSegment::new("Welcome to the Clawvinci editor.", 0.0, 2.5),
            TranscriptionSegment::new("Now let's check the quarterly budget.", 2.5, 5.0),
        ],
    );

    let assets = vec![("asset-1".to_string(), sample_transcript)];
    let hits = TranscriptSearch::search("budget", &assets, 10);
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].asset_id, "asset-1");
    assert_eq!(hits[0].start, 2.5);
    assert_eq!(hits[0].end, 5.0);
}

#[test]
fn test_word_cut_planner_and_merge() {
    let words = vec![
        CutWord::new(10, 20, false),
        CutWord::new(25, 35, true),  // Cut
        CutWord::new(36, 45, true),  // Cut run
        CutWord::new(50, 60, false),
        CutWord::new(65, 75, true),  // Cut
    ];

    let cuts = WordCutPlanner::cut_ranges(&words, 0, 100, 4);
    assert!(!cuts.is_empty());

    for cut in &cuts {
        assert!(cut.start >= 0);
        assert!(cut.end <= 100);
        assert!(cut.end > cut.start);
    }

    let aggressiveness = CutAggressiveness::Balanced;
    assert_eq!(aggressiveness.kept_gap_ms(), 150.0);
    assert_eq!(aggressiveness.kept_gap_frames(30.0), 5); // 0.15 * 30 = 4.5 rounded = 5
}

#[test]
fn test_transcript_result_offsetting_and_filtering() {
    let result = TranscriptionResult::new(
        "Testing offset",
        Some("en".into()),
        vec![
            TranscriptionWord::new("Hello", Some(1.0), Some(1.5)),
            TranscriptionWord::new("World", Some(1.5), Some(2.0)),
            TranscriptionWord::new("Later", Some(5.0), Some(5.5)),
        ],
        vec![
            TranscriptionSegment::new("Hello World", 1.0, 2.0),
            TranscriptionSegment::new("Later section", 5.0, 6.0),
        ],
    );

    let shifted = result.offsetting(10.0);
    assert_eq!(shifted.segments[0].start, 11.0);
    assert_eq!(shifted.segments[0].end, 12.0);
    assert_eq!(shifted.words[0].start, Some(11.0));

    let filtered = result.filter_range(0.5, 3.0);
    assert_eq!(filtered.segments.len(), 1);
    assert_eq!(filtered.words.len(), 2);
    assert_eq!(filtered.text, "Hello World");
}

#[test]
fn test_transcript_cache_disk_and_memory() {
    let temp_dir = std::env::temp_dir().join(format!("clawvinci_tc_test_{}", uuid::Uuid::new_v4()));
    let cache = TranscriptCache::new(temp_dir.clone());

    let dummy_file = temp_dir.join("sample.wav");
    fs::create_dir_all(&temp_dir).unwrap();
    fs::write(&dummy_file, b"RIFF dummy audio data").unwrap();

    let transcript = TranscriptionResult::new(
        "Cached audio transcript",
        Some("en".into()),
        vec![TranscriptionWord::new("Cached", Some(0.0), Some(1.0))],
        vec![TranscriptionSegment::new("Cached audio transcript", 0.0, 2.0)],
    );

    assert!(!cache.has_cached_on_disk(&dummy_file, None));
    cache.store(&dummy_file, &transcript, None).unwrap();
    assert!(cache.has_cached_on_disk(&dummy_file, None));

    // Retrieve from cache
    let retrieved = cache.get(&dummy_file, None, None).expect("Cache hit");
    assert_eq!(retrieved.text, "Cached audio transcript");

    // Clear memory and retrieve again from disk
    cache.clear_memory();
    let retrieved_disk = cache.get(&dummy_file, None, None).expect("Disk hit");
    assert_eq!(retrieved_disk.text, "Cached audio transcript");

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_embedding_store_binary_roundtrip() {
    let temp_dir = std::env::temp_dir().join(format!("clawvinci_emb_test_{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&temp_dir).unwrap();
    let embed_file = temp_dir.join("test.embed");

    let header = EmbeddingHeader {
        model: "siglip2-base-patch16-256".to_string(),
        model_version: 1,
        sampler_version: SAMPLER_VERSION,
        dim: 4,
        count: 2,
    };

    let rows = vec![
        EmbeddingRow::new(1.0, 0.0, 3.0),
        EmbeddingRow::new(4.5, 3.0, 6.0),
    ];

    let vectors = vec![
        0.5f32, -0.25f32, 0.125f32, 1.0f32,
        -1.0f32, 0.75f32, 0.0f32, 0.333f32,
    ];

    EmbeddingStore::save(&embed_file, &header, &rows, &vectors).unwrap();

    // Verify header read
    let loaded_header = EmbeddingStore::read_header(&embed_file).unwrap();
    assert_eq!(loaded_header, header);
    assert!(EmbeddingStore::is_current(&embed_file, "siglip2-base-patch16-256", 1, SAMPLER_VERSION));

    // Verify full index load
    let loaded_index = EmbeddingStore::load(&embed_file).unwrap();
    assert_eq!(loaded_index.header, header);
    assert_eq!(loaded_index.rows, rows);
    assert_eq!(loaded_index.vectors.len(), vectors.len());

    for (orig, loaded) in vectors.iter().zip(loaded_index.vectors.iter()) {
        assert!((orig - loaded).abs() < 1e-3, "Float16 conversion accuracy check");
    }

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_luma_grid_and_frame_sampler_cut_detection() {
    // 64x64 black frame
    let black = vec![0u8; 64 * 64 * 3];
    let black_grid = LumaGrid::compute_from_rgba(&black, 64, 64);
    assert_eq!(black_grid[0], 0.0);

    // 64x64 white frame
    let white = vec![255u8; 64 * 64 * 3];
    let white_grid = LumaGrid::compute_from_rgba(&white, 64, 64);
    assert!((white_grid[0] - 255.0).abs() < 1.0);

    let diff = LumaGrid::mean_diff(&black_grid, &white_grid);
    assert!((diff - 255.0).abs() < 1.0);

    // FrameSamplerState
    let mut sampler = FrameSamplerState::new(FrameSamplerOptions::default());
    let dec1 = sampler.evaluate(0.0, black_grid).expect("First frame always kept");
    assert!(dec1.is_new_shot);

    // Same frame 2 seconds later -> no new shot and coverage floor (8.0s) not reached -> not kept
    let dec2 = sampler.evaluate(2.0, black_grid);
    assert!(dec2.is_none());

    // Drastic change to white -> new shot detected
    let dec3 = sampler.evaluate(4.0, white_grid).expect("Scene cut kept");
    assert!(dec3.is_new_shot);
}

#[test]
fn test_visual_search_dot_product_and_ranking() {
    let embedder = MockVisualEmbedder::default();
    let query_vec = embedder.encode_text("sunset over ocean").unwrap();
    assert_eq!(query_vec.len(), 768);

    // Verify unit length
    let norm: f32 = query_vec.iter().map(|x| x * x).sum::<f32>().sqrt();
    assert!((norm - 1.0).abs() < 1e-4);

    let header = EmbeddingHeader {
        model: "siglip2-base-patch16-256".to_string(),
        model_version: 1,
        sampler_version: 1,
        dim: 768,
        count: 2,
    };

    let index1 = clawvinci_search::visual::AssetIndex {
        header: header.clone(),
        rows: vec![
            EmbeddingRow::new(1.0, 0.0, 5.0),
            EmbeddingRow::new(3.0, 0.0, 5.0), // Same shot
        ],
        vectors: {
            let mut v = query_vec.clone(); // Exact match
            v.extend_from_slice(&vec![0.0f32; 768]);
            v
        },
    };

    let index2 = clawvinci_search::visual::AssetIndex {
        header: header.clone(),
        rows: vec![
            EmbeddingRow::new(2.0, 0.0, 10.0),
            EmbeddingRow::new(7.0, 5.0, 10.0),
        ],
        vectors: vec![0.01f32; 768 * 2],
    };

    let indexes = vec![
        ("asset-exact", &index1),
        ("asset-unrelated", &index2),
    ];

    let hits = VisualSearch::search(&query_vec, &indexes, 10, 0.85, Some(0.05));
    assert!(!hits.is_empty());
    assert_eq!(hits[0].asset_id, "asset-exact");
    assert!((hits[0].score - 1.0).abs() < 1e-3);
}

#[test]
fn test_search_coordinator_preflight_and_spoken_search() {
    let temp_dir = std::env::temp_dir().join(format!("clawvinci_coord_test_{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&temp_dir).unwrap();

    let loader = Arc::new(VisualModelLoader::new(temp_dir.join("models")));
    loader.prepare();

    let transcript_cache = Arc::new(TranscriptCache::new(temp_dir.join("transcripts")));
    let coordinator = SearchIndexCoordinator::new(
        temp_dir.join("embeddings"),
        loader,
        transcript_cache.clone(),
    );

    let dummy_video = temp_dir.join("test.mp4");
    fs::write(&dummy_video, b"dummy video content").unwrap();

    let asset = SearchAssetInfo {
        id: "test-video-1".to_string(),
        path: dummy_video.clone(),
        media_type: SearchMediaType::Video,
        duration: 10.0,
        has_audio: true,
    };

    let preflight = coordinator.preflight(&asset, &ModelSpec::default());
    assert!(preflight.needs_visual);
    assert!(preflight.needs_transcript);

    // Populate transcript cache
    let transcript = TranscriptionResult::new(
        "Agentic AI video editor Clawvinci",
        Some("en".into()),
        vec![TranscriptionWord::new("Clawvinci", Some(1.0), Some(2.0))],
        vec![TranscriptionSegment::new("Agentic AI video editor Clawvinci", 0.0, 5.0)],
    );
    transcript_cache.store(&dummy_video, &transcript, None).unwrap();

    let spoken_hits = coordinator.search_spoken("Clawvinci", &[asset], 10, None);
    assert_eq!(spoken_hits.len(), 1);
    assert_eq!(spoken_hits[0].asset_id, "test-video-1");

    let _ = fs::remove_dir_all(&temp_dir);
}
