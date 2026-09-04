// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only
// Derived from Sources/PalmierPro/Models/MediaResolver.swift (GPLv3).

use crate::media_manifest::{MediaManifest, MediaManifestEntry, MediaSource};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct MediaResolver {
    manifest: MediaManifest,
    project_dir: Option<PathBuf>,
}

impl MediaResolver {
    pub fn new(manifest: MediaManifest, project_dir: Option<PathBuf>) -> Self {
        Self {
            manifest,
            project_dir,
        }
    }

    pub fn manifest(&self) -> &MediaManifest {
        &self.manifest
    }

    pub fn entry(&self, asset_id: &str) -> Option<&MediaManifestEntry> {
        self.manifest.entries.iter().find(|e| e.id == asset_id)
    }

    pub fn display_name(&self, asset_id: &str) -> String {
        self.entry(asset_id)
            .map(|e| e.name.clone())
            .unwrap_or_else(|| "Offline".to_string())
    }

    pub fn expected_path(&self, asset_id: &str) -> Option<PathBuf> {
        let entry = self.entry(asset_id)?;
        Self::expected_path_for_entry(entry, self.project_dir.as_deref())
    }

    pub fn expected_path_for_entry(
        entry: &MediaManifestEntry,
        project_dir: Option<&Path>,
    ) -> Option<PathBuf> {
        match &entry.source {
            MediaSource::External { absolute_path } => Some(PathBuf::from(absolute_path)),
            MediaSource::Project { relative_path } => {
                project_dir.map(|base| base.join(relative_path))
            }
        }
    }

    pub fn is_missing(&self, asset_id: &str) -> bool {
        match self.expected_path(asset_id) {
            Some(path) => !path.exists(),
            None => true,
        }
    }

    pub fn missing_asset_ids(
        entries: &[MediaManifestEntry],
        project_dir: Option<&Path>,
    ) -> HashSet<String> {
        let mut missing = HashSet::new();
        for entry in entries {
            match Self::expected_path_for_entry(entry, project_dir) {
                Some(path) => {
                    if !path.exists() {
                        missing.insert(entry.id.clone());
                    }
                }
                None => {
                    missing.insert(entry.id.clone());
                }
            }
        }
        missing
    }

    pub fn expected_path_map(&self) -> HashMap<String, PathBuf> {
        let mut map = HashMap::new();
        for entry in &self.manifest.entries {
            if let Some(path) = Self::expected_path_for_entry(entry, self.project_dir.as_deref()) {
                map.insert(entry.id.clone(), path);
            }
        }
        map
    }
}
