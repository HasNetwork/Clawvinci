// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only
// Derived from Sources/PalmierPro/Search/Indexing/EmbeddingStore.swift (GPLv3).

use crate::error::{SearchError, SearchResult};
use half::f16;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};

pub const EMBEDDING_STORE_MAGIC: &[u8; 8] = b"PALMEMB1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EmbeddingHeader {
    pub model: String,
    #[serde(rename = "modelVersion")]
    pub model_version: i32,
    #[serde(rename = "samplerVersion")]
    pub sampler_version: i32,
    pub dim: usize,
    pub count: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EmbeddingRow {
    pub time: f64,
    #[serde(rename = "shotStart")]
    pub shot_start: f64,
    #[serde(rename = "shotEnd")]
    pub shot_end: f64,
}

impl EmbeddingRow {
    pub fn new(time: f64, shot_start: f64, shot_end: f64) -> Self {
        Self {
            time,
            shot_start,
            shot_end,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AssetIndex {
    pub header: EmbeddingHeader,
    pub rows: Vec<EmbeddingRow>,
    /// Flat row-major `count * dim` vectors in float32.
    pub vectors: Vec<f32>,
}

pub struct EmbeddingStore;

impl EmbeddingStore {
    /// Generates a deterministic SHA256 hex cache key from file identity (path, mtime, size).
    pub fn key_for_file(path: &Path) -> Option<String> {
        let metadata = fs::metadata(path).ok()?;
        let size = metadata.len();
        let mtime = metadata
            .modified()
            .ok()?
            .duration_since(std::time::UNIX_EPOCH)
            .ok()?
            .as_secs_f64();

        let identity = format!("{}|{:.3}|{}", path.to_string_lossy(), mtime, size);
        let mut hasher = Sha256::new();
        hasher.update(identity.as_bytes());
        let hash = format!("{:x}", hasher.finalize());
        Some(hash.chars().take(32).collect())
    }

    pub fn disk_path(directory: &Path, key: &str) -> PathBuf {
        directory.join(format!("{key}.embed"))
    }

    /// Reads only the header from a stored `.embed` file.
    pub fn read_header(path: &Path) -> SearchResult<EmbeddingHeader> {
        let data = fs::read(path)?;
        if data.len() < 12 || &data[..8] != EMBEDDING_STORE_MAGIC {
            return Err(SearchError::CorruptStore("Invalid embedding file magic".into()));
        }

        let json_len = u32::from_le_bytes(data[8..12].try_into().unwrap()) as usize;
        if data.len() < 12 + json_len {
            return Err(SearchError::CorruptStore("Truncated embedding header".into()));
        }

        let header: EmbeddingHeader = serde_json::from_slice(&data[12..12 + json_len])?;
        Ok(header)
    }

    /// Checks if a stored embedding index exists and matches current model specifications.
    pub fn is_current(
        path: &Path,
        model: &str,
        model_version: i32,
        sampler_version: i32,
    ) -> bool {
        if let Ok(header) = Self::read_header(path) {
            header.model == model
                && header.model_version == model_version
                && header.sampler_version == sampler_version
        } else {
            false
        }
    }

    /// Serializes an asset embedding index to the `.embed` binary format.
    pub fn save(
        path: &Path,
        header: &EmbeddingHeader,
        rows: &[EmbeddingRow],
        vectors: &[f32],
    ) -> SearchResult<()> {
        if rows.len() != header.count || vectors.len() != header.count * header.dim {
            return Err(SearchError::CorruptStore(format!(
                "Dimension mismatch: expected {} vectors of dim {}, got rows={}, vectors={}",
                header.count,
                header.dim,
                rows.len(),
                vectors.len()
            )));
        }

        let json_bytes = serde_json::to_vec(header)?;
        let json_len = json_bytes.len() as u32;

        let row_bytes = 24 + header.dim * 2;
        let mut buffer = Vec::with_capacity(8 + 4 + json_bytes.len() + header.count * row_bytes);

        // Magic + JSON length + JSON
        buffer.extend_from_slice(EMBEDDING_STORE_MAGIC);
        buffer.extend_from_slice(&json_len.to_le_bytes());
        buffer.extend_from_slice(&json_bytes);

        // Write rows
        for (i, row) in rows.iter().enumerate() {
            buffer.extend_from_slice(&row.time.to_le_bytes());
            buffer.extend_from_slice(&row.shot_start.to_le_bytes());
            buffer.extend_from_slice(&row.shot_end.to_le_bytes());

            let vector_offset = i * header.dim;
            for d in 0..header.dim {
                let half_val = f16::from_f32(vectors[vector_offset + d]);
                buffer.extend_from_slice(&half_val.to_le_bytes());
            }
        }

        if let Some(parent) = path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)?;
            }
        }

        // Atomic write
        let temp_path = path.with_extension("tmp");
        fs::write(&temp_path, &buffer)?;
        fs::rename(&temp_path, path)?;

        Ok(())
    }

    /// Deserializes a `.embed` file into an `AssetIndex`.
    pub fn load(path: &Path) -> SearchResult<AssetIndex> {
        let data = fs::read(path)?;
        if data.len() < 12 || &data[..8] != EMBEDDING_STORE_MAGIC {
            return Err(SearchError::CorruptStore("Invalid embedding file magic".into()));
        }

        let json_len = u32::from_le_bytes(data[8..12].try_into().unwrap()) as usize;
        let mut offset = 12;
        if data.len() < offset + json_len {
            return Err(SearchError::CorruptStore("Truncated embedding header".into()));
        }

        let header: EmbeddingHeader = serde_json::from_slice(&data[offset..offset + json_len])?;
        offset += json_len;

        let row_bytes = 24 + header.dim * 2;
        let expected_total = offset + header.count * row_bytes;
        if data.len() != expected_total {
            return Err(SearchError::CorruptStore(format!(
                "Payload length mismatch: expected {expected_total} bytes, got {}",
                data.len()
            )));
        }

        let mut rows = Vec::with_capacity(header.count);
        let mut vectors = vec![0.0f32; header.count * header.dim];

        for i in 0..header.count {
            let base = offset + i * row_bytes;
            let time = f64::from_le_bytes(data[base..base + 8].try_into().unwrap());
            let shot_start = f64::from_le_bytes(data[base + 8..base + 16].try_into().unwrap());
            let shot_end = f64::from_le_bytes(data[base + 16..base + 24].try_into().unwrap());

            rows.push(EmbeddingRow {
                time,
                shot_start,
                shot_end,
            });

            let v_base = base + 24;
            let dest_base = i * header.dim;
            for d in 0..header.dim {
                let half_bytes = [data[v_base + d * 2], data[v_base + d * 2 + 1]];
                let half_val = f16::from_le_bytes(half_bytes);
                vectors[dest_base + d] = half_val.to_f32();
            }
        }

        Ok(AssetIndex {
            header,
            rows,
            vectors,
        })
    }
}
