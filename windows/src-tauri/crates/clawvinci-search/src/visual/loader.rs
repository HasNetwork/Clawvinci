// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only
// Derived from Sources/PalmierPro/Search/Models/VisualModelLoader.swift and SearchIndexConfig.swift (GPLv3).

use crate::model_download::{download_and_verify, ModelFileDownload};
use crate::visual::model::{ModelSpec, VisualEmbedder};
use crate::visual::onnx_embedder::OnnxVisualEmbedder;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};
use tracing::{error, info};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum LoaderState {
    Unknown,
    NotInstalled,
    Downloading(f64),
    Preparing,
    Ready,
    Failed(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelFileSpec {
    pub name: String,
    pub sha256: String,
    pub bytes: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelManifest {
    pub model: String,
    pub version: i32,
    #[serde(rename = "embeddingDim")]
    pub embedding_dim: usize,
    #[serde(rename = "imageSize")]
    pub image_size: usize,
    #[serde(rename = "contextLength")]
    pub context_length: usize,
    pub image_encoder: ModelFileSpec,
    pub text_encoder: ModelFileSpec,
    pub tokenizer: ModelFileSpec,
}

const SIGLIP2_HF_BASE: &str =
    "https://huggingface.co/google/siglip2-base-patch16-256/resolve/main/onnx";

impl Default for ModelManifest {
    fn default() -> Self {
        Self {
            model: "siglip2-base-patch16-256".to_string(),
            version: 1,
            embedding_dim: 768,
            image_size: 256,
            context_length: 64,
            image_encoder: ModelFileSpec {
                name: "ImageEncoder.onnx".to_string(),
                sha256: "426115f240ead5faf69b073e08dd1b959d850ca5c592537cd81886992283b2fb"
                    .to_string(),
                bytes: 91_700_398,
                url: Some(format!("{SIGLIP2_HF_BASE}/image_encoder.onnx")),
            },
            text_encoder: ModelFileSpec {
                name: "TextEncoder.onnx".to_string(),
                sha256: "48f80e35ce40a9dcdc55bef986a104d3153e1cfa78229bb45c4724f3f3427368"
                    .to_string(),
                bytes: 258_593_083,
                url: Some(format!("{SIGLIP2_HF_BASE}/text_encoder.onnx")),
            },
            tokenizer: ModelFileSpec {
                name: "tokenizer.json".to_string(),
                sha256: "c37f2a8e8555d8561109564c4f60ee962b0072abddcfcfd599d321469d6d1ef5"
                    .to_string(),
                bytes: 5_460_173,
                url: Some(
                    "https://huggingface.co/google/siglip2-base-patch16-256/resolve/main/tokenizer.json"
                        .to_string(),
                ),
            },
        }
    }
}

pub struct VisualModelLoader {
    models_dir: PathBuf,
    manifest: ModelManifest,
    state: RwLock<LoaderState>,
    enabled: AtomicBool,
    embedder: RwLock<Option<Arc<dyn VisualEmbedder>>>,
}

impl VisualModelLoader {
    pub fn new(models_dir: PathBuf) -> Self {
        let manifest = ModelManifest::default();
        Self {
            models_dir,
            manifest,
            state: RwLock::new(LoaderState::Unknown),
            enabled: AtomicBool::new(true),
            embedder: RwLock::new(None),
        }
    }

    pub fn models_dir(&self) -> &PathBuf {
        &self.models_dir
    }

    pub fn manifest(&self) -> &ModelManifest {
        &self.manifest
    }

    pub fn state(&self) -> LoaderState {
        self.state.read().unwrap().clone()
    }

    pub fn is_ready(&self) -> bool {
        matches!(self.state(), LoaderState::Ready)
    }

    pub fn enabled(&self) -> bool {
        self.enabled.load(Ordering::Relaxed)
    }

    pub fn set_enabled(&self, val: bool) {
        self.enabled.store(val, Ordering::Relaxed);
        if !val {
            *self.embedder.write().unwrap() = None;
            let mut s = self.state.write().unwrap();
            if matches!(*s, LoaderState::Ready | LoaderState::Preparing) {
                *s = LoaderState::Unknown;
            }
        }
    }

    pub fn embedder(&self) -> Option<Arc<dyn VisualEmbedder>> {
        self.embedder.read().unwrap().clone()
    }

    pub fn is_installed(&self) -> bool {
        let image_path = self.models_dir.join(&self.manifest.image_encoder.name);
        let text_path = self.models_dir.join(&self.manifest.text_encoder.name);
        let tok_path = self.models_dir.join(&self.manifest.tokenizer.name);
        image_path.exists() && text_path.exists() && tok_path.exists()
    }

    pub fn prepare(&self) {
        if !self.enabled() {
            return;
        }

        let mut s = self.state.write().unwrap();
        *s = LoaderState::Preparing;
        drop(s);

        if !self.is_installed() {
            let mut s = self.state.write().unwrap();
            *s = LoaderState::NotInstalled;
            info!("SigLIP2 models not installed at {}", self.models_dir.display());
            return;
        }

        let spec = ModelSpec {
            model: self.manifest.model.clone(),
            version: self.manifest.version,
            embedding_dim: self.manifest.embedding_dim,
            image_size: self.manifest.image_size,
            context_length: self.manifest.context_length,
        };

        let image_path = self.models_dir.join(&self.manifest.image_encoder.name);
        let text_path = self.models_dir.join(&self.manifest.text_encoder.name);
        let tok_path = self.models_dir.join(&self.manifest.tokenizer.name);

        match OnnxVisualEmbedder::load(spec, &image_path, &text_path, &tok_path) {
            Ok(embedder) => {
                info!("SigLIP2 ONNX embedder loaded successfully");
                *self.embedder.write().unwrap() = Some(Arc::new(embedder));
                *self.state.write().unwrap() = LoaderState::Ready;
            }
            Err(e) => {
                error!("Failed to load SigLIP2 ONNX embedder: {e}");
                *self.state.write().unwrap() =
                    LoaderState::Failed(format!("ONNX load failed: {e}"));
            }
        }
    }

    pub async fn download(&self) {
        if !self.enabled() {
            return;
        }
        {
            let s = self.state.read().unwrap();
            match *s {
                LoaderState::Downloading(_) | LoaderState::Preparing | LoaderState::Ready => return,
                _ => {}
            }
        }

        *self.state.write().unwrap() = LoaderState::Downloading(0.0);

        let files = [
            &self.manifest.image_encoder,
            &self.manifest.text_encoder,
            &self.manifest.tokenizer,
        ];

        let total_bytes: u64 = files.iter().map(|f| f.bytes).sum();
        let mut downloaded_bytes: u64 = 0;

        for file_spec in &files {
            let url = match &file_spec.url {
                Some(u) => u.clone(),
                None => {
                    *self.state.write().unwrap() = LoaderState::Failed(format!(
                        "No download URL for {}",
                        file_spec.name
                    ));
                    return;
                }
            };

            let spec = ModelFileDownload {
                url,
                dest: self.models_dir.join(&file_spec.name),
                expected_sha256: file_spec.sha256.clone(),
                expected_bytes: file_spec.bytes,
            };

            let base_bytes = downloaded_bytes;
            let state_ref = &self.state;
            let result = download_and_verify(&spec, |file_progress| {
                let current = base_bytes + (file_spec.bytes as f64 * file_progress) as u64;
                let overall = current as f64 / total_bytes as f64;
                *state_ref.write().unwrap() = LoaderState::Downloading(overall);
            })
            .await;

            match result {
                Ok(()) => {
                    downloaded_bytes += file_spec.bytes;
                }
                Err(e) => {
                    error!("Model download failed for {}: {e}", file_spec.name);
                    *self.state.write().unwrap() =
                        LoaderState::Failed(format!("Download failed: {e}"));
                    return;
                }
            }
        }

        self.prepare();
    }

    pub fn remove(&self) {
        *self.embedder.write().unwrap() = None;
        let mut s = self.state.write().unwrap();
        *s = LoaderState::NotInstalled;
        let _ = fs::remove_dir_all(&self.models_dir);
    }
}
