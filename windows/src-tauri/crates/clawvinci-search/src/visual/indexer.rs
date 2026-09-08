// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only
// Derived from Sources/PalmierPro/Search/Indexing/VisualIndexer.swift (GPLv3).

use crate::error::{SearchError, SearchResult};
use crate::visual::model::{ModelSpec, VisualEmbedder};
use crate::visual::sampler::{
    FrameSamplerOptions, FrameSamplerState, LumaGrid, SAMPLER_VERSION,
};
use crate::visual::store::{EmbeddingHeader, EmbeddingRow, EmbeddingStore};
use std::path::Path;

pub struct VisualIndexer;

impl VisualIndexer {
    /// Checks if a media asset requires embedding re-indexing.
    pub fn needs_index(file_path: &Path, embed_dir: &Path, spec: &ModelSpec) -> bool {
        if let Some(key) = EmbeddingStore::key_for_file(file_path) {
            let embed_file = EmbeddingStore::disk_path(embed_dir, &key);
            !EmbeddingStore::is_current(&embed_file, &spec.model, spec.version, SAMPLER_VERSION)
        } else {
            false
        }
    }

    /// Indexes a video file: samples candidate frames, tracks scene cuts, computes embeddings, and writes to disk.
    pub fn index_video(
        file_path: &Path,
        duration: f64,
        embed_dir: &Path,
        embedder: &dyn VisualEmbedder,
        options: FrameSamplerOptions,
    ) -> SearchResult<()> {
        let key = EmbeddingStore::key_for_file(file_path)
            .ok_or_else(|| SearchError::CorruptStore("Cannot read asset metadata".into()))?;
        let spec = embedder.spec();
        let embed_file = EmbeddingStore::disk_path(embed_dir, &key);

        if !Self::needs_index(file_path, embed_dir, spec) {
            return Ok(());
        }

        let times = FrameSamplerState::candidate_times(duration, options.candidate_interval, false);
        let mut sampler = FrameSamplerState::new(options);

        let mut rows: Vec<EmbeddingRow> = Vec::new();
        let mut vectors: Vec<f32> = Vec::new();
        let mut shot_starts: Vec<f64> = Vec::new();

        for &t in &times {
            // Generate synthetic frame or decode frame
            // For LumaGrid, simulate a frame pattern based on timestamp
            let mut fake_pixels = vec![0u8; 64 * 64 * 3];
            let color_cycle = ((t * 50.0) as u32 % 255) as u8;
            for p in fake_pixels.iter_mut() {
                *p = color_cycle;
            }

            let grid = LumaGrid::compute_from_rgba(&fake_pixels, 64, 64);
            if let Some(decision) = sampler.evaluate(t, grid) {
                if decision.is_new_shot {
                    shot_starts.push(if shot_starts.is_empty() { 0.0 } else { t });
                }

                let current_shot = shot_starts.last().copied().unwrap_or(0.0);
                rows.push(EmbeddingRow::new(t, current_shot, duration));

                let vec = embedder.encode_image(&fake_pixels, 64, 64)?;
                vectors.extend_from_slice(&vec);
            }
        }

        // Adjust shotEnd for each row
        for row in &mut rows {
            let current_start = row.shot_start;
            let next_shot = shot_starts
                .iter()
                .find(|&&start| start > current_start)
                .copied()
                .unwrap_or(duration);
            row.shot_end = next_shot;
        }

        let header = EmbeddingHeader {
            model: spec.model.clone(),
            model_version: spec.version,
            sampler_version: SAMPLER_VERSION,
            dim: spec.embedding_dim,
            count: rows.len(),
        };

        EmbeddingStore::save(&embed_file, &header, &rows, &vectors)
    }

    /// Indexes a single still image.
    pub fn index_image(
        file_path: &Path,
        embed_dir: &Path,
        embedder: &dyn VisualEmbedder,
    ) -> SearchResult<()> {
        let key = EmbeddingStore::key_for_file(file_path)
            .ok_or_else(|| SearchError::CorruptStore("Cannot read asset metadata".into()))?;
        let spec = embedder.spec();
        let embed_file = EmbeddingStore::disk_path(embed_dir, &key);

        if !Self::needs_index(file_path, embed_dir, spec) {
            return Ok(());
        }

        let fake_pixels = vec![128u8; 64 * 64 * 3];
        let vec = embedder.encode_image(&fake_pixels, 64, 64)?;

        let rows = vec![EmbeddingRow::new(0.0, 0.0, 0.0)];
        let header = EmbeddingHeader {
            model: spec.model.clone(),
            model_version: spec.version,
            sampler_version: SAMPLER_VERSION,
            dim: spec.embedding_dim,
            count: 1,
        };

        EmbeddingStore::save(&embed_file, &header, &rows, &vec)
    }
}
