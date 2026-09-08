// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only
// Derived from Sources/PalmierPro/Generation/GenerationService.swift (GPLv3).

use crate::backend::client::GenerationBackendClient;
use crate::backend::types::{BackendGenerationJob, BackendGenerationParams, BackendGenerationStatus};
use crate::error::{GenError, GenResult};
use clawvinci_media::probe::probe_media;
use clawvinci_model::clip_type::ClipType;
use clawvinci_model::media_manifest::{
    GenerationInput, MediaManifest, MediaManifestEntry, MediaSource,
};
use clawvinci_model::timeline::{Clip, Timeline};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use tokio_util::sync::CancellationToken;
use tracing::{error, info};
use uuid::Uuid;

pub struct GenerationService<B: GenerationBackendClient> {
    backend: Arc<B>,
}

impl<B: GenerationBackendClient> GenerationService<B> {
    pub fn new(backend: Arc<B>) -> Self {
        Self { backend }
    }

    pub fn backend(&self) -> &B {
        &self.backend
    }

    /// Creates a placeholder media asset entry and determines its destination path on disk.
    pub fn create_placeholder(
        &self,
        clip_type: ClipType,
        name: &str,
        duration_seconds: f64,
        gen_input: GenerationInput,
        dest_dir: &Path,
        file_extension: &str,
    ) -> (MediaManifestEntry, PathBuf) {
        let id = Uuid::new_v4().to_string();
        let short_id = if id.len() >= 8 { &id[..8] } else { &id };
        let filename = format!("gen-{short_id}.{file_extension}");
        let dest_path = dest_dir.join(&filename);

        let mut entry = MediaManifestEntry::new(
            &id,
            name,
            clip_type,
            MediaSource::Project {
                relative_path: format!("media/{filename}"),
            },
            duration_seconds,
        );
        entry.generation_input = Some(gen_input);
        entry.generation_status = Some("preparing".to_string());

        (entry, dest_path)
    }

    /// Submits a generation job to the backend and associates the returned job ID with the manifest entry.
    pub async fn submit(
        &self,
        model_id: &str,
        params: &BackendGenerationParams,
        project_id: Option<&str>,
        manifest_entry: &mut MediaManifestEntry,
    ) -> GenResult<String> {
        let job_id = self.backend.submit(model_id, params, project_id).await?;
        if let Some(ref mut input) = manifest_entry.generation_input {
            input.backend_job_id = Some(job_id.clone());
        }
        manifest_entry.generation_status = Some("generating".to_string());
        Ok(job_id)
    }

    /// Monitors the generation job until terminal completion or cancellation.
    /// Downloads the result file and probes its media metadata upon success.
    pub async fn monitor_and_finalize(
        &self,
        job_id: &str,
        dest_path: &Path,
        manifest_entry: &mut MediaManifestEntry,
        cancel: &CancellationToken,
    ) -> GenResult<BackendGenerationJob> {
        let mut poll_interval = Duration::from_millis(500);
        let max_interval = Duration::from_secs(4);

        loop {
            if cancel.is_cancelled() {
                manifest_entry.generation_status = Some("cancelled".to_string());
                return Err(GenError::Cancelled);
            }

            let job = self.backend.get_job(job_id).await?;
            match job.status {
                BackendGenerationStatus::Queued | BackendGenerationStatus::Running => {
                    manifest_entry.generation_status = Some(format!(
                        "{}: {:.0}%",
                        match job.status {
                            BackendGenerationStatus::Queued => "queued",
                            _ => "generating",
                        },
                        job.progress * 100.0
                    ));
                    sleep(poll_interval).await;
                    poll_interval = (poll_interval * 3 / 2).min(max_interval);
                }
                BackendGenerationStatus::Succeeded => {
                    info!("Generation job succeeded: {job_id}");
                    if let Some(result_url) = job.result_urls.first() {
                        manifest_entry.generation_status = Some("downloading".to_string());
                        self.backend
                            .download_file(result_url, dest_path, cancel)
                            .await?;

                        // Probe resulting media file for duration and resolution
                        if let Ok(probe) = probe_media(dest_path).await {
                            manifest_entry.duration = probe.duration_seconds;
                            if let Some(v) = probe.video_stream() {
                                manifest_entry.source_width = Some(v.width as i32);
                                manifest_entry.source_height = Some(v.height as i32);
                                manifest_entry.source_fps = Some(v.fps);
                            }
                            manifest_entry.has_audio = Some(probe.has_audio());
                        }

                        if let Some(ref mut input) = manifest_entry.generation_input {
                            input.result_urls = Some(job.result_urls.clone());
                            input.cost_credits = job.cost_credits.map(|c| c as i32);
                            input.refunded_credits = job.refunded_credits.map(|c| c as i32);
                        }
                        manifest_entry.generation_status = Some("ready".to_string());
                    }
                    return Ok(job);
                }
                BackendGenerationStatus::Failed => {
                    let err_msg = job
                        .error_message
                        .clone()
                        .unwrap_or_else(|| "Unknown backend error".to_string());
                    error!("Generation job failed: {job_id}: {err_msg}");
                    manifest_entry.generation_status = Some(format!("failed: {err_msg}"));
                    return Err(GenError::JobFailed(err_msg));
                }
            }
        }
    }

    /// Registers the placeholder in a MediaManifest.
    pub fn register_in_manifest(manifest: &mut MediaManifest, entry: MediaManifestEntry) {
        if let Some(pos) = manifest.entries.iter().position(|e| e.id == entry.id) {
            manifest.entries[pos] = entry;
        } else {
            manifest.entries.push(entry);
        }
    }

    /// Places a generating placeholder or finalized clip on a Timeline track.
    pub fn place_clip_on_timeline(
        timeline: &mut Timeline,
        track_index: usize,
        asset_id: &str,
        name: &str,
        start_frame: i64,
        duration_frames: i64,
    ) -> Result<String, GenError> {
        if track_index >= timeline.tracks.len() {
            return Err(GenError::InvalidInput(format!(
                "Track index {track_index} out of bounds"
            )));
        }

        let clip = Clip::new(
            asset_id,
            name,
            start_frame,
            duration_frames,
            0,
            duration_frames,
        );
        let clip_id = clip.id.clone();
        timeline.tracks[track_index].clips.push(clip);
        timeline.recalculate_duration();
        Ok(clip_id)
    }
}
