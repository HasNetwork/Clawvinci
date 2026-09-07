// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only
// Derived from Sources/PalmierPro/Export/ExportQueue.swift (GPLv3).

use crate::error::{ExportError, ExportResult};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum ExportJobSource {
    #[default]
    Manual,
    Agent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum ExportJobStatus {
    #[default]
    Queued,
    Preparing,
    Rendering,
    Canceling,
    Completed,
    Failed,
    Canceled,
}

impl ExportJobStatus {
    pub fn is_running(&self) -> bool {
        matches!(self, Self::Preparing | Self::Rendering | Self::Canceling)
    }

    pub fn is_finished(&self) -> bool {
        matches!(self, Self::Completed | Self::Failed | Self::Canceled)
    }

    pub fn is_pending(&self) -> bool {
        matches!(self, Self::Queued) || self.is_running()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportJob {
    pub id: Uuid,
    pub project_id: String,
    pub filename: String,
    pub source: ExportJobSource,
    pub output_path: PathBuf,
    pub created_at: DateTime<Utc>,
    pub status: ExportJobStatus,
    pub progress: f64,
    pub error: Option<String>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportQueueSubmission {
    pub job_id: Uuid,
    pub started: bool,
    pub queue_position: usize,
}

#[derive(Default)]
pub struct ExportQueue {
    jobs: Vec<ExportJob>,
    active_id: Option<Uuid>,
    active_cancel_token: Option<CancellationToken>,
}

impl ExportQueue {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn jobs(&self) -> &[ExportJob] {
        &self.jobs
    }

    pub fn get_job(&self, id: Uuid) -> Option<&ExportJob> {
        self.jobs.iter().find(|j| j.id == id)
    }

    pub fn get_job_mut(&mut self, id: Uuid) -> Option<&mut ExportJob> {
        self.jobs.iter_mut().find(|j| j.id == id)
    }

    pub fn is_destination_reserved(&self, path: &Path) -> bool {
        self.jobs
            .iter()
            .any(|j| j.status.is_pending() && j.output_path == path)
    }

    pub fn enqueue(
        &mut self,
        project_id: String,
        output_path: PathBuf,
        source: ExportJobSource,
        warnings: Vec<String>,
    ) -> ExportResult<ExportQueueSubmission> {
        if self.is_destination_reserved(&output_path) {
            let fname = output_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("output")
                .to_string();
            return Err(ExportError::DestinationInUse(fname));
        }

        let id = Uuid::new_v4();
        let filename = output_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("output")
            .to_string();

        let job = ExportJob {
            id,
            project_id,
            filename,
            source,
            output_path,
            created_at: Utc::now(),
            status: ExportJobStatus::Queued,
            progress: 0.0,
            error: None,
            warnings,
        };

        self.jobs.push(job);

        let queue_pos = self
            .jobs
            .iter()
            .filter(|j| j.status == ExportJobStatus::Queued)
            .position(|j| j.id == id)
            .unwrap_or(0)
            + 1;

        let started = self.active_id.is_none();

        Ok(ExportQueueSubmission {
            job_id: id,
            started,
            queue_position: queue_pos,
        })
    }

    pub fn next_waiting_job(&mut self) -> Option<(Uuid, CancellationToken)> {
        if self.active_id.is_some() {
            return None;
        }

        if let Some(job) = self.jobs.iter_mut().find(|j| j.status == ExportJobStatus::Queued) {
            job.status = ExportJobStatus::Preparing;
            let id = job.id;
            let cancel_token = CancellationToken::new();
            self.active_id = Some(id);
            self.active_cancel_token = Some(cancel_token.clone());
            Some((id, cancel_token))
        } else {
            None
        }
    }

    pub fn update_progress(&mut self, id: Uuid, progress: f64) {
        if let Some(job) = self.get_job_mut(id) {
            if job.status.is_pending() {
                job.progress = progress.clamp(0.0, 1.0);
                if job.status == ExportJobStatus::Preparing {
                    job.status = ExportJobStatus::Rendering;
                }
            }
        }
    }

    pub fn finish_job(
        &mut self,
        id: Uuid,
        status: ExportJobStatus,
        error: Option<String>,
        extra_warnings: Vec<String>,
    ) {
        if let Some(job) = self.get_job_mut(id) {
            job.status = status;
            if status == ExportJobStatus::Completed {
                job.progress = 1.0;
            }
            job.error = error;
            job.warnings.extend(extra_warnings);
        }

        if self.active_id == Some(id) {
            self.active_id = None;
            self.active_cancel_token = None;
        }
    }

    pub fn cancel(&mut self, id: Uuid) -> bool {
        if let Some(job) = self.get_job_mut(id) {
            match job.status {
                ExportJobStatus::Queued => {
                    job.status = ExportJobStatus::Canceled;
                    true
                }
                ExportJobStatus::Preparing | ExportJobStatus::Rendering => {
                    job.status = ExportJobStatus::Canceling;
                    if let Some(ref token) = self.active_cancel_token {
                        token.cancel();
                    }
                    true
                }
                ExportJobStatus::Canceling => true,
                _ => false,
            }
        } else {
            false
        }
    }

    pub fn remove(&mut self, id: Uuid) {
        if let Some(pos) = self.jobs.iter().position(|j| j.id == id) {
            if self.jobs[pos].status.is_finished() {
                self.jobs.remove(pos);
            }
        }
    }

    pub fn clear_finished(&mut self, project_id: Option<&str>) {
        self.jobs.retain(|j| {
            if !j.status.is_finished() {
                return true;
            }
            if let Some(pid) = project_id {
                j.project_id != pid
            } else {
                false
            }
        });
    }
}
