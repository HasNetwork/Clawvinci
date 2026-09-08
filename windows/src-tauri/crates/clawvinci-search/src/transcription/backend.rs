// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only
// BYOK transcription client supporting OpenAI-compatible Whisper endpoints.

use crate::error::{SearchError, SearchResult};
use crate::transcription::result::{TranscriptionResult, TranscriptionSegment, TranscriptionWord};
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TranscriptionBackendConfig {
    pub endpoint: Option<String>,
    pub api_key: Option<String>,
    pub model: Option<String>,
}

impl Default for TranscriptionBackendConfig {
    fn default() -> Self {
        Self {
            endpoint: None,
            api_key: None,
            model: None,
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

    pub fn config(&self) -> &TranscriptionBackendConfig {
        &self.config
    }

    pub fn is_configured(&self) -> bool {
        self.config
            .endpoint
            .as_deref()
            .map_or(false, |s| !s.trim().is_empty())
    }

    /// Transcribes raw audio bytes against an OpenAI-compatible Whisper REST API (`/v1/audio/transcriptions`).
    pub async fn transcribe_audio(
        &self,
        audio_bytes: Vec<u8>,
        filename: &str,
        language: Option<&str>,
    ) -> SearchResult<TranscriptionResult> {
        let endpoint = match &self.config.endpoint {
            Some(e) if !e.trim().is_empty() => e.trim().trim_end_matches('/'),
            _ => {
                return Err(SearchError::CloudTranscriptionFailed(
                    "No BYOK transcription endpoint configured. Please configure an OpenAI-compatible endpoint or use local on-device transcription.".to_string(),
                ));
            }
        };

        let url = if endpoint.ends_with("/audio/transcriptions") {
            endpoint.to_string()
        } else if endpoint.ends_with("/v1") {
            format!("{endpoint}/audio/transcriptions")
        } else {
            format!("{endpoint}/v1/audio/transcriptions")
        };

        let file_part = reqwest::multipart::Part::bytes(audio_bytes)
            .file_name(filename.to_string())
            .mime_str("audio/wav")
            .map_err(|e| SearchError::CloudTranscriptionFailed(format!("Failed to configure audio payload: {e}")))?;

        let model_name = self.config.model.as_deref().unwrap_or("whisper-1");

        let mut form = reqwest::multipart::Form::new()
            .part("file", file_part)
            .text("model", model_name.to_string())
            .text("response_format", "verbose_json".to_string())
            .text("timestamp_granularities[]", "word".to_string())
            .text("timestamp_granularities[]", "segment".to_string());

        if let Some(lang) = language {
            form = form.text("language", lang.to_string());
        }

        let mut req = self.client.post(&url).multipart(form);
        if let Some(token) = &self.config.api_key {
            req = req.bearer_auth(token);
        }

        let resp = req
            .send()
            .await
            .map_err(|e| SearchError::CloudTranscriptionFailed(e.to_string()))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let err_body = resp.text().await.unwrap_or_default();
            return Err(SearchError::CloudTranscriptionFailed(format!(
                "HTTP error {status}: {err_body}"
            )));
        }

        let json_val: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| SearchError::CloudTranscriptionFailed(format!("Failed to parse response JSON: {e}")))?;

        Self::parse_whisper_response(&json_val)
    }

    /// Parses OpenAI-compatible `verbose_json` response into domain `TranscriptionResult`.
    pub fn parse_whisper_response(json: &serde_json::Value) -> SearchResult<TranscriptionResult> {
        let text = json
            .get("text")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .trim()
            .to_string();

        let language = json
            .get("language")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let mut words = Vec::new();
        if let Some(words_arr) = json.get("words").and_then(|v| v.as_array()) {
            for w in words_arr {
                let w_text = w.get("word").and_then(|v| v.as_str()).unwrap_or("").trim().to_string();
                let start = w.get("start").and_then(|v| v.as_f64());
                let end = w.get("end").and_then(|v| v.as_f64());
                if !w_text.is_empty() {
                    words.push(TranscriptionWord::new(w_text, start, end));
                }
            }
        }

        let mut segments = Vec::new();
        if let Some(segments_arr) = json.get("segments").and_then(|v| v.as_array()) {
            for s in segments_arr {
                let s_text = s.get("text").and_then(|v| v.as_str()).unwrap_or("").trim().to_string();
                let start = s.get("start").and_then(|v| v.as_f64()).unwrap_or(0.0);
                let end = s.get("end").and_then(|v| v.as_f64()).unwrap_or(0.0);
                if !s_text.is_empty() {
                    segments.push(TranscriptionSegment::new(s_text, start, end));
                }
            }
        }

        Ok(TranscriptionResult::new(text, language, words, segments))
    }

    /// Submits an audio storage payload for async cloud transcription.
    pub async fn submit(
        &self,
        storage_id: &str,
        duration_seconds: f64,
        language: Option<&str>,
        project_id: Option<&str>,
    ) -> SearchResult<BackendTranscriptionSubmit> {
        let endpoint = match &self.config.endpoint {
            Some(e) if !e.trim().is_empty() => e.trim().trim_end_matches('/'),
            _ => {
                return Err(SearchError::CloudTranscriptionFailed(
                    "No BYOK transcription endpoint configured.".to_string(),
                ));
            }
        };

        let url = format!("{endpoint}/api/transcriptions/submit");
        let body = serde_json::json!({
            "storageId": storage_id,
            "durationSeconds": duration_seconds,
            "model": self.config.model.as_deref().unwrap_or("cloud"),
            "languageMode": if language.is_some() { "specific" } else { "auto" },
            "language": language,
            "projectId": project_id,
        });

        let mut req = self.client.post(&url).json(&body);
        if let Some(token) = &self.config.api_key {
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
        let endpoint = match &self.config.endpoint {
            Some(e) if !e.trim().is_empty() => e.trim().trim_end_matches('/'),
            _ => {
                return Err(SearchError::CloudTranscriptionFailed(
                    "No BYOK transcription endpoint configured.".to_string(),
                ));
            }
        };

        let url = format!("{endpoint}/api/transcriptions/byId?id={job_id}");
        let mut req = self.client.get(&url);
        if let Some(token) = &self.config.api_key {
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
        let mut req = self.client.get(result_url);
        if let Some(token) = &self.config.api_key {
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

        let result: TranscriptionResult = resp
            .json()
            .await
            .map_err(|e| SearchError::CloudTranscriptionFailed(e.to_string()))?;

        Ok(result)
    }
}
