// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only
// Derived from Sources/PalmierPro/Transcription/TranscriptCache.swift (GPLv3).

use crate::error::{SearchError, SearchResult};
use crate::transcription::result::TranscriptionResult;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::RwLock;

pub struct TranscriptCache {
    directory: PathBuf,
    memory: RwLock<HashMap<String, TranscriptionResult>>,
    memory_max: usize,
}

impl TranscriptCache {
    pub fn new(directory: PathBuf) -> Self {
        Self {
            directory,
            memory: RwLock::new(HashMap::new()),
            memory_max: 32,
        }
    }

    /// Derives a 32-character SHA256 hex cache key from file path, mtime, and size.
    pub fn key_for_file(path: &Path, variant: Option<&str>) -> Option<String> {
        let metadata = fs::metadata(path).ok()?;
        let size = metadata.len();
        let mtime = metadata
            .modified()
            .ok()?
            .duration_since(std::time::UNIX_EPOCH)
            .ok()?
            .as_secs_f64();

        let base = format!("{}|{:.3}|{}", path.to_string_lossy(), mtime, size);
        let identity = if let Some(v) = variant {
            format!("{v}|{base}")
        } else {
            base
        };

        let mut hasher = Sha256::new();
        hasher.update(identity.as_bytes());
        let hash = format!("{:x}", hasher.finalize());
        Some(hash.chars().take(32).collect())
    }

    pub fn disk_path(&self, key: &str) -> PathBuf {
        self.directory.join(format!("{key}.json"))
    }

    /// Checks if a transcript is cached either in memory or on disk.
    pub fn has_cached_on_disk(&self, path: &Path, variant: Option<&str>) -> bool {
        if let Some(key) = Self::key_for_file(path, variant) {
            self.disk_path(&key).exists()
        } else {
            false
        }
    }

    /// Retrieves a cached transcript from memory or disk, optionally slicing to `[start, end]`.
    pub fn get(
        &self,
        path: &Path,
        range: Option<(f64, f64)>,
        variant: Option<&str>,
    ) -> Option<TranscriptionResult> {
        let key = Self::key_for_file(path, variant)?;

        // Try memory first
        if let Ok(mem) = self.memory.read() {
            if let Some(cached) = mem.get(&key) {
                return Some(if let Some((start, end)) = range {
                    cached.filter_range(start, end)
                } else {
                    cached.clone()
                });
            }
        }

        // Try disk
        let disk_file = self.disk_path(&key);
        if disk_file.exists() {
            if let Ok(data) = fs::read(&disk_file) {
                if let Ok(loaded) = serde_json::from_slice::<TranscriptionResult>(&data) {
                    if let Ok(mut mem) = self.memory.write() {
                        if mem.len() >= self.memory_max {
                            mem.clear();
                        }
                        mem.insert(key, loaded.clone());
                    }
                    return Some(if let Some((start, end)) = range {
                        loaded.filter_range(start, end)
                    } else {
                        loaded
                    });
                }
            }
        }

        None
    }

    /// Stores a transcript to both in-memory cache and atomic disk JSON.
    pub fn store(
        &self,
        path: &Path,
        result: &TranscriptionResult,
        variant: Option<&str>,
    ) -> SearchResult<()> {
        let key = Self::key_for_file(path, variant)
            .ok_or_else(|| SearchError::CorruptStore("Could not read file metadata".into()))?;

        if let Ok(mut mem) = self.memory.write() {
            if mem.len() >= self.memory_max {
                mem.clear();
            }
            mem.insert(key.clone(), result.clone());
        }

        if !self.directory.exists() {
            fs::create_dir_all(&self.directory)?;
        }

        let disk_file = self.disk_path(&key);
        let temp_file = self.directory.join(format!("{key}.tmp"));
        let json = serde_json::to_vec_pretty(result)?;
        fs::write(&temp_file, json)?;
        fs::rename(&temp_file, &disk_file)?;

        Ok(())
    }

    /// Clears in-memory entries so that disk updates are refreshed.
    pub fn clear_memory(&self) {
        if let Ok(mut mem) = self.memory.write() {
            mem.clear();
        }
    }
}
