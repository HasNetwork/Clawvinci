// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only
// Derived from Sources/PalmierPro/Export/PalmierProjectExporter.swift (GPLv3).

use crate::error::{ExportError, ExportResult};
use clawvinci_model::{MediaManifest, MediaManifestEntry, MediaSource, ProjectFile};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct MissingMedia {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ProjectBundleReport {
    pub collected: Vec<String>,
    pub copied_internal: usize,
    pub missing: Vec<MissingMedia>,
    pub total_bytes: u64,
}

impl ProjectBundleReport {
    pub fn warnings(&self) -> Vec<String> {
        if self.missing.is_empty() {
            Vec::new()
        } else {
            let files = if self.missing.len() == 1 {
                "media file was"
            } else {
                "media files were"
            };
            vec![format!(
                "{} {} missing and could not be included in the bundle.",
                self.missing.len(),
                files
            )]
        }
    }
}

pub struct PalmierProjectExporter;

impl PalmierProjectExporter {
    pub async fn export(
        project_file: &ProjectFile,
        manifest: &MediaManifest,
        source_project_path: Option<&Path>,
        dest_path: &Path,
        cancel_token: CancellationToken,
        progress_callback: Option<Box<dyn Fn(f64) + Send + Sync + 'static>>,
    ) -> ExportResult<ProjectBundleReport> {
        let parent = dest_path.parent().unwrap_or_else(|| Path::new("."));
        tokio::fs::create_dir_all(parent).await?;

        let staging_name = format!(".palmier-export-{}.partial", Uuid::new_v4());
        let staging_dir = parent.join(&staging_name);
        let media_dir = staging_dir.join("media");
        tokio::fs::create_dir_all(&media_dir).await?;

        struct StagingCleanup(PathBuf);
        impl Drop for StagingCleanup {
            fn drop(&mut self) {
                if self.0.exists() {
                    let _ = std::fs::remove_dir_all(&self.0);
                }
            }
        }
        let cleanup_guard = StagingCleanup(staging_dir.clone());

        let mut report = ProjectBundleReport::default();
        let mut new_entries = Vec::new();
        let mut rel_path_by_src: HashMap<String, String> = HashMap::new();
        let total = manifest.entries.len().max(1);

        for (index, entry) in manifest.entries.iter().enumerate() {
            if cancel_token.is_cancelled() {
                return Err(ExportError::Cancelled);
            }

            let source_path_opt = match &entry.source {
                MediaSource::External { path } => {
                    let p = PathBuf::from(path);
                    if p.exists() {
                        Some(p)
                    } else {
                        None
                    }
                }
                MediaSource::Project { relative_path } => {
                    if let Some(src_proj) = source_project_path {
                        let p = src_proj.join(relative_path);
                        if p.exists() {
                            Some(p)
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                }
            };

            guard_copy_media(
                entry,
                source_path_opt,
                &media_dir,
                &mut rel_path_by_src,
                &mut report,
                &mut new_entries,
            ).await?;

            if let Some(ref cb) = progress_callback {
                cb((index + 1) as f64 / total as f64);
            }
        }

        if cancel_token.is_cancelled() {
            return Err(ExportError::Cancelled);
        }

        // Update manifest entries
        let mut new_manifest = manifest.clone();
        new_manifest.entries = new_entries;

        // Write project.json and manifest.json
        let project_json_path = staging_dir.join("project.json");
        let manifest_json_path = staging_dir.join("manifest.json");

        let proj_data = serde_json::to_string_pretty(project_file)?;
        let manifest_data = serde_json::to_string_pretty(&new_manifest)?;

        tokio::fs::write(&project_json_path, proj_data).await?;
        tokio::fs::write(&manifest_json_path, manifest_data).await?;

        // Copy optional thumbnail / chat session if present in source project
        if let Some(src_proj) = source_project_path {
            let thumb = src_proj.join("thumbnail.png");
            if thumb.exists() {
                let _ = tokio::fs::copy(&thumb, staging_dir.join("thumbnail.png")).await;
            }
        }

        if cancel_token.is_cancelled() {
            return Err(ExportError::Cancelled);
        }

        // Atomic install: if dest_path exists, remove it first, then rename staging_dir to dest_path
        if dest_path.exists() {
            if dest_path.is_dir() {
                tokio::fs::remove_dir_all(dest_path).await?;
            } else {
                tokio::fs::remove_file(dest_path).await?;
            }
        }

        tokio::fs::rename(&staging_dir, dest_path).await?;

        // Prevent cleanup guard from deleting the installed directory
        std::mem::forget(cleanup_guard);

        Ok(report)
    }
}

async fn guard_copy_media(
    entry: &MediaManifestEntry,
    source_path_opt: Option<PathBuf>,
    media_dir: &Path,
    rel_path_by_src: &mut HashMap<String, String>,
    report: &mut ProjectBundleReport,
    new_entries: &mut Vec<MediaManifestEntry>,
) -> ExportResult<()> {
    match source_path_opt {
        Some(src_path) => {
            let key = src_path.to_string_lossy().to_string();
            let relative_path = if let Some(existing) = rel_path_by_src.get(&key) {
                existing.clone()
            } else {
                let preferred_name = filename_for_entry(entry, &src_path);
                let dest_file = unique_path_in_dir(media_dir, &preferred_name);
                let bytes_copied = tokio::fs::copy(&src_path, &dest_file).await?;
                let file_name = dest_file
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("media.bin");
                let rel = format!("media/{}", file_name);
                rel_path_by_src.insert(key, rel.clone());
                report.total_bytes += bytes_copied;
                if matches!(entry.source, MediaSource::Project { .. }) {
                    report.copied_internal += 1;
                }
                rel
            };

            if matches!(entry.source, MediaSource::External { .. }) {
                report.collected.push(entry.id.clone());
            }

            let mut rewritten = entry.clone();
            rewritten.source = MediaSource::Project { relative_path };
            new_entries.push(rewritten);
        }
        None => {
            report.missing.push(MissingMedia {
                id: entry.id.clone(),
                name: entry.name.clone(),
            });
            new_entries.push(entry.clone());
        }
    }
    Ok(())
}

fn filename_for_entry(entry: &MediaManifestEntry, src_path: &Path) -> String {
    match &entry.source {
        MediaSource::Project { .. } => src_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("media.bin")
            .to_string(),
        MediaSource::External { .. } => {
            let ext = src_path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("");
            let prefix: String = entry.id.chars().take(8).collect();
            if ext.is_empty() {
                format!("import-{}", prefix)
            } else {
                format!("import-{}.{}", prefix, ext)
            }
        }
    }
}

fn unique_path_in_dir(dir: &Path, preferred_name: &str) -> PathBuf {
    let candidate = dir.join(preferred_name);
    if !candidate.exists() {
        return candidate;
    }

    let p = Path::new(preferred_name);
    let stem = p.file_stem().and_then(|s| s.to_str()).unwrap_or("file");
    let ext = p.extension().and_then(|e| e.to_str()).unwrap_or("");

    let mut n = 1;
    loop {
        let name = if ext.is_empty() {
            format!("{}-{}", stem, n)
        } else {
            format!("{}-{}.{}", stem, n, ext)
        };
        let c = dir.join(&name);
        if !c.exists() {
            return c;
        }
        n += 1;
    }
}
