// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only
// Derived from Sources/PalmierPro/Transcription/TranscriptionBackend.swift and CloudTranscription.swift (GPLv3).

use crate::error::{SearchError, SearchResult};
use crate::transcription::result::TranscriptionResult;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BackendTranscriptionStatus {
    Queued,
    Running,
    Succeeded,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackendTranscriptionSubmit {
    #[serde(rename = "jobId")]
    pub job_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackendTranscriptionJob {
    pub id: String,
    pub status: BackendTranscriptionStatus,
    #[serde(rename = "errorMessage", skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
}

#[derive(Debug, Clone)]
pub struct TranscriptionBackendConfig {
    pub endpoint: String,
    pub auth_token: Option<String>,
}

impl Default for TranscriptionBackendConfig {
    fn default() -> Self {
        Self {
            endpoint: "https://api.palmier.io".to_string(),
            auth_token: None,
        }
    }
}

pub struct TranscriptionBackend {
    config: TranscriptionBackendConfig,
    client: reqwest::Client,
}

impl TranscriptionBackend {
    pub fn new(config: TranscriptionBackendConfig) -> Self {
        Self {
            config,
            client: reqwest::Client::new(),
        }
    }

    /// Submits an audio storage payload for cloud transcription.
    pub async fn submit(
        &self,
        storage_id: &str,
        duration_seconds: f64,
        language: Option<&str>,
        project_id: Option<&str>,
    ) -> SearchResult<BackendTranscriptionSubmit> {
        let url = format!("{}/api/transcriptions/submit", self.config.endpoint);
        let body = serde_json::json!({
            "storageId": storage_id,
            "durationSeconds": duration_seconds,
            "model": "cloud",
            "languageMode": if language.is_some() { "specific" } else { "auto" },
            "language": language,
            "projectId": project_id,
        });

        let mut req = self.client.post(&url).json(&body);
        if let Some(token) = &self.config.auth_token {
            req = req.bearer_auth(token);
        }

        let resp = req
            .send()
            .await
            .map_err(|e| SearchError::CloudTranscriptionFailed(e.to_string()))?;

        if !resp.status().is_success() {
            return Err(SearchError::CloudTranscriptionFailed(format!(
                "HTTP error {}",
                resp.status()
            )));
        }

        let submit: BackendTranscriptionSubmit = resp
            .json()
            .await
            .map_err(|e| SearchError::CloudTranscriptionFailed(e.to_string()))?;

        Ok(submit)
    }

    /// Polls or queries the current status of a transcription job.
    pub async fn status(&self, job_id: &str) -> SearchResult<BackendTranscriptionJob> {
        let url = format!("{}/api/transcriptions/byId?id={job_id}", self.config.endpoint);
        let mut req = self.client.get(&url);
        if let Some(token) = &self.config.auth_token {
            req = req.bearer_auth(token);
        }

        let resp = req
            .send()
            .await
            .map_err(|e| SearchError::CloudTranscriptionFailed(e.to_string()))?;

        if !resp.status().is_success() {
            return Err(SearchError::CloudTranscriptionFailed(format!(
                "HTTP error {}",
                resp.status()
            )));
        }

        let job: BackendTranscriptionJob = resp
            .json()
            .await
            .map_err(|e| SearchError::CloudTranscriptionFailed(e.to_string()))?;

        Ok(job)
    }

    /// Downloads the final `TranscriptionResult` JSON.
    pub async fn download_result(&self, result_url: &str) -> SearchResult<TranscriptionResult> {
        let resp = self
            .client
            .get(result_url)
            .send()
            .await
            .map_err(|e| SearchError::CloudTranscriptionFailed(e.to_string()))?;

        if !resp.status().is_success() {
            return Err(SearchError::CloudTranscriptionFailed(format!(
                "HTTP error {}",
                resp.status()
            )));
        }

        let result: TranscriptionResult = resp
            .json()
            .await
            .map_err(|e| SearchError::CloudTranscriptionFailed(e.to_string()))?;

        Ok(result)
    }
}
