// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only
// Derived from Sources/PalmierPro/Search/SearchIndexCoordinator.swift (GPLv3).

use crate::error::SearchResult;
use crate::transcription::cache::TranscriptCache;
use crate::transcription::result::TranscriptionResult;
use crate::transcription::search::{TranscriptHit, TranscriptSearch};
use crate::visual::loader::VisualModelLoader;
use crate::visual::model::{ModelSpec, VisualEmbedder};
use crate::visual::search::{VisualHit, VisualSearch};
use crate::visual::store::{AssetIndex, EmbeddingStore};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SearchMediaType {
    Video,
    Audio,
    Image,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SearchAssetInfo {
    pub id: String,
    pub path: PathBuf,
    pub media_type: SearchMediaType,
    pub duration: f64,
    pub has_audio: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreflightResult {
    pub needs_visual: bool,
    pub needs_transcript: bool,
}

impl PreflightResult {
    pub fn needs_index(&self) -> bool {
        self.needs_visual || self.needs_transcript
    }
}

pub struct SearchIndexCoordinator {
    embed_dir: PathBuf,
    loader: Arc<VisualModelLoader>,
    transcript_cache: Arc<TranscriptCache>,
    loaded_indexes: RwLock<HashMap<String, (String, AssetIndex)>>,
    scheduled_ids: RwLock<HashSet<String>>,
}

impl SearchIndexCoordinator {
    pub fn new(
        embed_dir: PathBuf,
        loader: Arc<VisualModelLoader>,
        transcript_cache: Arc<TranscriptCache>,
    ) -> Self {
        Self {
            embed_dir,
            loader,
            transcript_cache,
            loaded_indexes: RwLock::new(HashMap::new()),
            scheduled_ids: RwLock::new(HashSet::new()),
        }
    }

    /// Preflight check determining if an asset requires visual or transcript indexing.
    pub fn preflight(&self, asset: &SearchAssetInfo, spec: &ModelSpec) -> PreflightResult {
        let needs_visual = match asset.media_type {
            SearchMediaType::Video | SearchMediaType::Image => {
                if let Some(key) = EmbeddingStore::key_for_file(&asset.path) {
                    let path = EmbeddingStore::disk_path(&self.embed_dir, &key);
                    !EmbeddingStore::is_current(&path, &spec.model, spec.version, 1)
                } else {
                    false
                }
            }
            SearchMediaType::Audio => false,
        };

        let needs_transcript = match asset.media_type {
            SearchMediaType::Audio => !self.transcript_cache.has_cached_on_disk(&asset.path, None),
            SearchMediaType::Video if asset.has_audio => {
                !self.transcript_cache.has_cached_on_disk(&asset.path, None)
            }
            _ => false,
        };

        PreflightResult {
            needs_visual,
            needs_transcript,
        }
    }

    /// Searches visual scene embeddings for a textual or visual concept query.
    pub fn search_visual(
        &self,
        query: &str,
        assets: &[SearchAssetInfo],
        limit: usize,
        within_ids: Option<&HashSet<String>>,
    ) -> SearchResult<Vec<VisualHit>> {
        let embedder = match self.loader.embedder() {
            Some(e) => e,
            None => return Ok(Vec::new()),
        };

        let trimmed = query.trim();
        if trimmed.is_empty() {
            return Ok(Vec::new());
        }

        let query_vec = embedder.encode_text(trimmed)?;

        let mut loaded_indexes = self.loaded_indexes.write().unwrap();
        let mut query_indexes: Vec<(String, AssetIndex)> = Vec::new();

        for asset in assets {
            if let Some(within) = within_ids {
                if !within.contains(&asset.id) {
                    continue;
                }
            }

            if asset.media_type != SearchMediaType::Video && asset.media_type != SearchMediaType::Image {
                continue;
            }

            let key = match EmbeddingStore::key_for_file(&asset.path) {
                Some(k) => k,
                None => continue,
            };

            if let Some((cached_key, index)) = loaded_indexes.get(&asset.id) {
                if cached_key == &key {
                    query_indexes.push((asset.id.clone(), index.clone()));
                    continue;
                }
            }

            let path = EmbeddingStore::disk_path(&self.embed_dir, &key);
            if let Ok(index) = EmbeddingStore::load(&path) {
                loaded_indexes.insert(asset.id.clone(), (key, index.clone()));
                query_indexes.push((asset.id.clone(), index));
            }
        }

        let slice_ref: Vec<(&str, &AssetIndex)> = query_indexes
            .iter()
            .map(|(id, idx)| (id.as_str(), idx))
            .collect();

        let hits = VisualSearch::search(&query_vec, &slice_ref, limit, 0.85, Some(0.05));
        Ok(hits)
    }

    /// Searches spoken dialogue across cached transcripts.
    pub fn search_spoken(
        &self,
        query: &str,
        assets: &[SearchAssetInfo],
        limit: usize,
        within_ids: Option<&HashSet<String>>,
    ) -> Vec<TranscriptHit> {
        let mut transcript_assets: Vec<(String, TranscriptionResult)> = Vec::new();

        for asset in assets {
            if let Some(within) = within_ids {
                if !within.contains(&asset.id) {
                    continue;
                }
            }

            if let Some(transcript) = self.transcript_cache.get(&asset.path, None, None) {
                transcript_assets.push((asset.id.clone(), transcript));
            }
        }

        TranscriptSearch::search(query, &transcript_assets, limit)
    }

    pub fn clear_cache(&self) {
        self.loaded_indexes.write().unwrap().clear();
        self.scheduled_ids.write().unwrap().clear();
    }
}
