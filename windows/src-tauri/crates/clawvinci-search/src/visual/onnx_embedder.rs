// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only
// Real SigLIP2 visual embedder using ONNX Runtime inference.

use crate::error::{SearchError, SearchResult};
use crate::visual::model::{ModelSpec, VisualEmbedder};
use ndarray::Array4;
use ort::session::Session;
use ort::value::Tensor;
use std::path::Path;
use tokenizers::Tokenizer;

pub struct OnnxVisualEmbedder {
    spec: ModelSpec,
    image_session: Session,
    text_session: Session,
    tokenizer: Tokenizer,
}

impl OnnxVisualEmbedder {
    pub fn load(
        spec: ModelSpec,
        image_encoder_path: &Path,
        text_encoder_path: &Path,
        tokenizer_path: &Path,
    ) -> SearchResult<Self> {
        let image_session = Session::builder()
            .and_then(|mut b| b.commit_from_file(image_encoder_path))
            .map_err(|e| {
                SearchError::ModelNotReady(format!("Failed to load image encoder ONNX: {e}"))
            })?;

        let text_session = Session::builder()
            .and_then(|mut b| b.commit_from_file(text_encoder_path))
            .map_err(|e| {
                SearchError::ModelNotReady(format!("Failed to load text encoder ONNX: {e}"))
            })?;

        let tokenizer = Tokenizer::from_file(tokenizer_path).map_err(|e| {
            SearchError::ModelNotReady(format!("Failed to load tokenizer: {e}"))
        })?;

        Ok(Self {
            spec,
            image_session,
            text_session,
            tokenizer,
        })
    }

    fn normalize_l2(vec: &mut [f32]) {
        let norm_sq: f32 = vec.iter().map(|x| x * x).sum();
        let norm = norm_sq.sqrt();
        if norm > 1e-8 {
            for x in vec.iter_mut() {
                *x /= norm;
            }
        }
    }
}

// ort::Session is Send+Sync in ort v2; tokenizers::Tokenizer is Send+Sync.
unsafe impl Send for OnnxVisualEmbedder {}
unsafe impl Sync for OnnxVisualEmbedder {}

impl VisualEmbedder for OnnxVisualEmbedder {
    fn spec(&self) -> &ModelSpec {
        &self.spec
    }

    fn encode_text(&self, text: &str) -> SearchResult<Vec<f32>> {
        let encoding = self
            .tokenizer
            .encode(text, true)
            .map_err(|e| SearchError::AnalysisFailed(format!("Tokenization failed: {e}")))?;

        let mut input_ids: Vec<i64> = encoding.get_ids().iter().map(|&id| id as i64).collect();

        let ctx_len = self.spec.context_length;
        input_ids.truncate(ctx_len);
        while input_ids.len() < ctx_len {
            input_ids.push(0);
        }

        let input_array =
            ndarray::Array2::from_shape_vec((1, ctx_len), input_ids).map_err(|e| {
                SearchError::AnalysisFailed(format!("Failed to create input tensor: {e}"))
            })?;

        let input_tensor = Tensor::from_array(input_array).map_err(|e| {
            SearchError::AnalysisFailed(format!("Failed to create ONNX tensor: {e}"))
        })?;

        let outputs = self
            .text_session
            .run(ort::inputs![input_tensor])
            .map_err(|e| {
                SearchError::AnalysisFailed(format!("Text encoder inference failed: {e}"))
            })?;

        let output_view = outputs[0].try_extract_array::<f32>().map_err(|e| {
            SearchError::AnalysisFailed(format!("Failed to extract output tensor: {e}"))
        })?;

        let mut embedding: Vec<f32> = output_view.iter().copied().collect();
        embedding.truncate(self.spec.embedding_dim);
        Self::normalize_l2(&mut embedding);
        Ok(embedding)
    }

    fn encode_image(&self, rgb: &[u8], width: u32, height: u32) -> SearchResult<Vec<f32>> {
        if width == 0 || height == 0 || rgb.is_empty() {
            return Ok(vec![0.0; self.spec.embedding_dim]);
        }

        let size = self.spec.image_size;
        let mut pixel_values = Array4::<f32>::zeros((1, 3, size, size));

        let channels = (rgb.len() / (width as usize * height as usize)).max(3);
        for y in 0..size {
            for x in 0..size {
                let src_x = ((x as f64 * width as f64 / size as f64) as u32).min(width - 1);
                let src_y = ((y as f64 * height as f64 / size as f64) as u32).min(height - 1);
                let idx = (src_y as usize * width as usize + src_x as usize) * channels;
                if idx + 2 < rgb.len() {
                    pixel_values[[0, 0, y, x]] = rgb[idx] as f32 / 127.5 - 1.0;
                    pixel_values[[0, 1, y, x]] = rgb[idx + 1] as f32 / 127.5 - 1.0;
                    pixel_values[[0, 2, y, x]] = rgb[idx + 2] as f32 / 127.5 - 1.0;
                }
            }
        }

        let input_tensor = Tensor::from_array(pixel_values).map_err(|e| {
            SearchError::AnalysisFailed(format!("Failed to create ONNX tensor: {e}"))
        })?;

        let outputs = self
            .image_session
            .run(ort::inputs![input_tensor])
            .map_err(|e| {
                SearchError::AnalysisFailed(format!("Image encoder inference failed: {e}"))
            })?;

        let output_view = outputs[0].try_extract_array::<f32>().map_err(|e| {
            SearchError::AnalysisFailed(format!("Failed to extract output tensor: {e}"))
        })?;

        let mut embedding: Vec<f32> = output_view.iter().copied().collect();
        embedding.truncate(self.spec.embedding_dim);
        Self::normalize_l2(&mut embedding);
        Ok(embedding)
    }
}
