// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only
// Derived from Sources/PalmierPro/Generation/GenerationBackend.swift (GPLv3).

use super::types::{BackendGenerationJob, BackendGenerationParams, BackendGenerationStatus};
use crate::error::{GenError, GenResult};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

pub trait GenerationBackendClient: Send + Sync {
    fn submit(
        &self,
        model: &str,
        params: &BackendGenerationParams,
        project_id: Option<&str>,
    ) -> impl std::future::Future<Output = GenResult<String>> + Send;

    fn get_job(
        &self,
        job_id: &str,
    ) -> impl std::future::Future<Output = GenResult<BackendGenerationJob>> + Send;

    fn upload_reference(
        &self,
        file_path: &Path,
        content_type: &str,
    ) -> impl std::future::Future<Output = GenResult<String>> + Send;

    fn download_file(
        &self,
        url: &str,
        dest_path: &Path,
        cancel: &CancellationToken,
    ) -> impl std::future::Future<Output = GenResult<()>> + Send;
}

/// HTTP REST backend client implementation for remote generative services.
#[derive(Clone)]
pub struct HttpGenerationBackend {
    base_url: String,
    api_token: Option<String>,
    client: reqwest::Client,
}

impl HttpGenerationBackend {
    pub fn new(base_url: impl Into<String>, api_token: Option<String>) -> Self {
        Self {
            base_url: base_url.into().trim_end_matches('/').to_string(),
            api_token,
            client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(60))
                .build()
                .unwrap_or_default(),
        }
    }

    fn headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        if let Some(token) = &self.api_token {
            if let Ok(val) = HeaderValue::from_str(&format!("Bearer {token}")) {
                headers.insert(AUTHORIZATION, val);
            }
        }
        headers
    }
}

impl GenerationBackendClient for HttpGenerationBackend {
    async fn submit(
        &self,
        model: &str,
        params: &BackendGenerationParams,
        project_id: Option<&str>,
    ) -> GenResult<String> {
        let url = format!("{}/generations/submit", self.base_url);
        let payload = serde_json::json!({
            "model": model,
            "params": params,
            "projectId": project_id,
        });

        let response = self
            .client
            .post(&url)
            .headers(self.headers())
            .json(&payload)
            .send()
            .await
            .map_err(|e| GenError::Network(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(GenError::Backend(format!(
                "Backend submission returned {status}: {text}"
            )));
        }

        let body: serde_json::Value = response
            .json()
            .await
            .map_err(|e| GenError::Network(format!("Failed to parse response: {e}")))?;

        body.get("jobId")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| GenError::Backend("Response missing 'jobId'".to_string()))
    }

    async fn get_job(&self, job_id: &str) -> GenResult<BackendGenerationJob> {
        let url = format!("{}/generations/{}", self.base_url, job_id);
        let response = self
            .client
            .get(&url)
            .headers(self.headers())
            .send()
            .await
            .map_err(|e| GenError::Network(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(GenError::Backend(format!(
                "Failed to get job status {status}: {text}"
            )));
        }

        response
            .json::<BackendGenerationJob>()
            .await
            .map_err(|e| GenError::Network(format!("Failed to parse job JSON: {e}")))
    }

    async fn upload_reference(&self, file_path: &Path, content_type: &str) -> GenResult<String> {
        let file_bytes = tokio::fs::read(file_path).await?;
        let url = format!("{}/uploads/reference", self.base_url);

        let mut headers = self.headers();
        if let Ok(ct) = HeaderValue::from_str(content_type) {
            headers.insert(CONTENT_TYPE, ct);
        }

        let response = self
            .client
            .post(&url)
            .headers(headers)
            .body(file_bytes)
            .send()
            .await
            .map_err(|e| GenError::Network(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(GenError::Backend(format!(
                "Failed to upload reference {status}: {text}"
            )));
        }

        let body: serde_json::Value = response
            .json()
            .await
            .map_err(|e| GenError::Network(format!("Failed to parse upload response: {e}")))?;

        body.get("url")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| GenError::Backend("Upload response missing 'url'".to_string()))
    }

    async fn download_file(
        &self,
        url: &str,
        dest_path: &Path,
        cancel: &CancellationToken,
    ) -> GenResult<()> {
        let mut response = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|e| GenError::Network(e.to_string()))?;

        if !response.status().is_success() {
            return Err(GenError::Backend(format!(
                "Download returned HTTP {}",
                response.status()
            )));
        }

        if let Some(parent) = dest_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        let mut file = tokio::fs::File::create(dest_path).await?;
        use tokio::io::AsyncWriteExt;

        while let Some(chunk) = response
            .chunk()
            .await
            .map_err(|e| GenError::Network(e.to_string()))?
        {
            if cancel.is_cancelled() {
                let _ = tokio::fs::remove_file(dest_path).await;
                return Err(GenError::Cancelled);
            }
            file.write_all(&chunk).await?;
        }

        file.flush().await?;
        Ok(())
    }
}

/// In-memory mock backend client for deterministic, offline testing.
#[derive(Clone, Default)]
pub struct MockGenerationBackend {
    jobs: Arc<Mutex<HashMap<String, BackendGenerationJob>>>,
    upload_counter: Arc<Mutex<usize>>,
}

impl MockGenerationBackend {
    pub fn new() -> Self {
        Self {
            jobs: Arc::new(Mutex::new(HashMap::new())),
            upload_counter: Arc::new(Mutex::new(0)),
        }
    }

    pub async fn register_job(&self, job: BackendGenerationJob) {
        let mut map = self.jobs.lock().await;
        map.insert(job.id.clone(), job);
    }

    pub async fn set_job_status(
        &self,
        job_id: &str,
        status: BackendGenerationStatus,
        progress: f64,
        result_urls: Vec<String>,
        error_message: Option<String>,
    ) {
        let mut map = self.jobs.lock().await;
        if let Some(job) = map.get_mut(job_id) {
            job.status = status;
            job.progress = progress;
            job.result_urls = result_urls;
            job.error_message = error_message;
        }
    }
}

impl GenerationBackendClient for MockGenerationBackend {
    async fn submit(
        &self,
        _model: &str,
        _params: &BackendGenerationParams,
        _project_id: Option<&str>,
    ) -> GenResult<String> {
        let job_id = format!("mock-job-{}", Uuid::new_v4());
        let mut map = self.jobs.lock().await;
        map.insert(
            job_id.clone(),
            BackendGenerationJob {
                id: job_id.clone(),
                status: BackendGenerationStatus::Succeeded,
                progress: 1.0,
                result_urls: vec![format!("https://mock.clawvinci.com/results/{job_id}.mp4")],
                error_message: None,
                cost_credits: Some(10),
                refunded_credits: None,
                completed_at: Some(1725000000.0),
            },
        );
        Ok(job_id)
    }

    async fn get_job(&self, job_id: &str) -> GenResult<BackendGenerationJob> {
        let map = self.jobs.lock().await;
        map.get(job_id)
            .cloned()
            .ok_or_else(|| GenError::Backend(format!("Mock job '{job_id}' not found")))
    }

    async fn upload_reference(&self, file_path: &Path, _content_type: &str) -> GenResult<String> {
        let mut counter = self.upload_counter.lock().await;
        *counter += 1;
        let filename = file_path
            .file_name()
            .and_then(|f| f.to_str())
            .unwrap_or("ref");
        Ok(format!(
            "https://mock.clawvinci.com/uploads/{}-{}",
            *counter, filename
        ))
    }

    async fn download_file(
        &self,
        _url: &str,
        dest_path: &Path,
        cancel: &CancellationToken,
    ) -> GenResult<()> {
        if cancel.is_cancelled() {
            return Err(GenError::Cancelled);
        }

        if let Some(parent) = dest_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        // Write synthetic minimal bytes so file exists on disk
        let synthetic_data = b"MOCK_GENERATED_MEDIA_PAYLOAD";
        tokio::fs::write(dest_path, synthetic_data).await?;
        Ok(())
    }
}
