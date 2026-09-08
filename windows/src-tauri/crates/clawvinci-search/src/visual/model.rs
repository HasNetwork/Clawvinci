// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only
// Derived from Sources/PalmierPro/Search/Models/VisualEmbedder.swift (GPLv3).

use crate::error::SearchResult;
use serde::{Deserialize, Serialize};
use std::hash::{DefaultHasher, Hash, Hasher};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelSpec {
    pub model: String,
    pub version: i32,
    #[serde(rename = "embeddingDim")]
    pub embedding_dim: usize,
    #[serde(rename = "imageSize")]
    pub image_size: usize,
    #[serde(rename = "contextLength")]
    pub context_length: usize,
}

impl Default for ModelSpec {
    fn default() -> Self {
        Self {
            model: "siglip2-base-patch16-256".to_string(),
            version: 1,
            embedding_dim: 768,
            image_size: 256,
            context_length: 64,
        }
    }
}

pub trait VisualEmbedder: Send + Sync {
    fn spec(&self) -> &ModelSpec;
    fn encode_text(&self, text: &str) -> SearchResult<Vec<f32>>;
    fn encode_image(&self, rgb: &[u8], width: u32, height: u32) -> SearchResult<Vec<f32>>;
}

/// A fast, deterministic L2-normalized pseudo-semantic embedder for testing and offline fallback.
#[derive(Debug, Clone)]
pub struct MockVisualEmbedder {
    spec: ModelSpec,
}

impl MockVisualEmbedder {
    pub fn new(spec: ModelSpec) -> Self {
        Self { spec }
    }

    fn normalize_l2(mut vec: Vec<f32>) -> Vec<f32> {
        let norm_sq: f32 = vec.iter().map(|x| x * x).sum();
        let norm = norm_sq.sqrt();
        if norm > 1e-8 {
            for x in &mut vec {
                *x /= norm;
            }
        }
        vec
    }
}

impl Default for MockVisualEmbedder {
    fn default() -> Self {
        Self::new(ModelSpec::default())
    }
}

impl VisualEmbedder for MockVisualEmbedder {
    fn spec(&self) -> &ModelSpec {
        &self.spec
    }

    fn encode_text(&self, text: &str) -> SearchResult<Vec<f32>> {
        let dim = self.spec.embedding_dim;
        let mut vector = vec![0.0f32; dim];

        for word in text.split_whitespace() {
            let mut hasher = DefaultHasher::new();
            word.to_lowercase().hash(&mut hasher);
            let h = hasher.finish();

            // Distribute energy across pseudo-random indices
            for k in 0..8 {
                let idx = ((h.wrapping_add(k * 7919)) as usize) % dim;
                let sign = if (h >> k) & 1 == 1 { 1.0f32 } else { -1.0f32 };
                vector[idx] += sign;
            }
        }

        Ok(Self::normalize_l2(vector))
    }

    fn encode_image(&self, rgb: &[u8], width: u32, height: u32) -> SearchResult<Vec<f32>> {
        let dim = self.spec.embedding_dim;
        let mut vector = vec![0.0f32; dim];

        if width == 0 || height == 0 || rgb.is_empty() {
            return Ok(vector);
        }

        // Downsample into 8x8 coarse color patches
        let step_x = (width / 8).max(1) as usize;
        let step_y = (height / 8).max(1) as usize;
        let channels = (rgb.len() / (width as usize * height as usize)).max(3);

        for y in 0..8 {
            for x in 0..8 {
                let px = (y * step_y * width as usize + x * step_x) * channels;
                if px + 2 < rgb.len() {
                    let r = rgb[px];
                    let g = rgb[px + 1];
                    let b = rgb[px + 2];

                    let mut hasher = DefaultHasher::new();
                    (x, y, r / 32, g / 32, b / 32).hash(&mut hasher);
                    let h = hasher.finish();

                    let idx = (h as usize) % dim;
                    vector[idx] += 1.0;
                }
            }
        }

        Ok(Self::normalize_l2(vector))
    }
}
