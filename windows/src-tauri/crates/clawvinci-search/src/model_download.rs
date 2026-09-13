// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only
// Shared model download infrastructure for Whisper and SigLIP2 ONNX models.

use crate::error::{SearchError, SearchResult};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use tokio::io::AsyncWriteExt;
use tracing::{info, warn};

pub fn default_models_dir() -> PathBuf {
    let base = dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("."));
    base.join("Clawvinci").join("Models")
}

pub struct ModelFileDownload {
    pub url: String,
    pub dest: PathBuf,
    pub expected_sha256: String,
    pub expected_bytes: u64,
}

pub async fn download_and_verify(
    spec: &ModelFileDownload,
    progress: impl Fn(f64),
) -> SearchResult<()> {
    if let Some(parent) = spec.dest.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|e| SearchError::DownloadFailed(format!("Cannot create directory: {e}")))?;
    }

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(600))
        .build()
        .map_err(|e| SearchError::DownloadFailed(format!("HTTP client error: {e}")))?;

    let response = client
        .get(&spec.url)
        .send()
        .await
        .map_err(|e| SearchError::DownloadFailed(format!("Download request failed: {e}")))?;

    if !response.status().is_success() {
        return Err(SearchError::DownloadFailed(format!(
            "HTTP {} from {}",
            response.status(),
            spec.url
        )));
    }

    let content_length = response.content_length().unwrap_or(spec.expected_bytes);

    let tmp_path = spec.dest.with_extension("download");
    let mut file = tokio::fs::File::create(&tmp_path)
        .await
        .map_err(|e| SearchError::DownloadFailed(format!("Cannot create temp file: {e}")))?;

    let mut hasher = Sha256::new();
    let mut downloaded: u64 = 0;
    let mut stream = response.bytes_stream();

    use futures_util::StreamExt;
    while let Some(chunk_result) = stream.next().await {
        let chunk = chunk_result
            .map_err(|e| SearchError::DownloadFailed(format!("Download stream error: {e}")))?;
        file.write_all(&chunk)
            .await
            .map_err(|e| SearchError::DownloadFailed(format!("Write error: {e}")))?;
        hasher.update(&chunk);
        downloaded += chunk.len() as u64;
        if content_length > 0 {
            progress(downloaded as f64 / content_length as f64);
        }
    }

    file.flush()
        .await
        .map_err(|e| SearchError::DownloadFailed(format!("Flush error: {e}")))?;
    drop(file);

    let hash = format!("{:x}", hasher.finalize());
    if hash != spec.expected_sha256 {
        let _ = tokio::fs::remove_file(&tmp_path).await;
        return Err(SearchError::DownloadFailed(format!(
            "SHA256 mismatch for {}: expected {}, got {}",
            spec.dest.display(),
            spec.expected_sha256,
            hash
        )));
    }

    tokio::fs::rename(&tmp_path, &spec.dest)
        .await
        .map_err(|e| SearchError::DownloadFailed(format!("Cannot rename temp file: {e}")))?;

    info!(
        "Model file downloaded and verified: {} ({} bytes)",
        spec.dest.display(),
        downloaded
    );
    Ok(())
}

pub fn verify_file_sha256(path: &Path, expected: &str) -> bool {
    let Ok(bytes) = std::fs::read(path) else {
        return false;
    };
    let hash = format!("{:x}", Sha256::digest(&bytes));
    hash == expected
}
