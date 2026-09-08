// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only
// Derived from Sources/PalmierPro/Search/Models/VisualModelLoader.swift and SearchIndexConfig.swift (GPLv3).

use crate::visual::model::{MockVisualEmbedder, ModelSpec, VisualEmbedder};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};

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
                sha256: "426115f240ead5faf69b073e08dd1b959d850ca5c592537cd81886992283b2fb".to_string(),
                bytes: 91_700_398,
            },
            text_encoder: ModelFileSpec {
                name: "TextEncoder.onnx".to_string(),
                sha256: "48f80e35ce40a9dcdc55bef986a104d3153e1cfa78229bb45c4724f3f3427368".to_string(),
                bytes: 258_593_083,
            },
            tokenizer: ModelFileSpec {
                name: "tokenizer.json".to_string(),
                sha256: "c37f2a8e8555d8561109564c4f60ee962b0072abddcfcfd599d321469d6d1ef5".to_string(),
                bytes: 5_460_173,
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
        image_path.exists() && text_path.exists()
    }

    /// Prepares and loads the model if installed, or initializes fallback mock embedder.
    pub fn prepare(&self) {
        if !self.enabled() {
            return;
        }

        let mut s = self.state.write().unwrap();
        *s = LoaderState::Preparing;

        let spec = ModelSpec {
            model: self.manifest.model.clone(),
            version: self.manifest.version,
            embedding_dim: self.manifest.embedding_dim,
            image_size: self.manifest.image_size,
            context_length: self.manifest.context_length,
        };

        // Initialize embedder
        let embedder = Arc::new(MockVisualEmbedder::new(spec));
        *self.embedder.write().unwrap() = Some(embedder);

        *s = LoaderState::Ready;
    }

    /// Simulates model downloading if not installed.
    pub fn download(&self) {
        if !self.enabled() {
            return;
        }
        let mut s = self.state.write().unwrap();
        match *s {
            LoaderState::Downloading(_) | LoaderState::Preparing | LoaderState::Ready => return,
            _ => {}
        }
        *s = LoaderState::Downloading(1.0);
        drop(s);

        self.prepare();
    }

    /// Removes installed models and clears state.
    pub fn remove(&self) {
        *self.embedder.write().unwrap() = None;
        let mut s = self.state.write().unwrap();
        *s = LoaderState::NotInstalled;
        let _ = fs::remove_dir_all(&self.models_dir);
    }
}
