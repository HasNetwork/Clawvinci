// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only
// Derived from Sources/PalmierPro/Generation/Submission/ImageGenerationSubmission.swift (GPLv3).

use super::types::{BackendGenerationParams, ImageGenerationParams};
use clawvinci_model::media_manifest::GenerationInput;

#[derive(Debug, Clone)]
pub struct ImageGenerationSubmission {
    pub input: GenerationInput,
    pub num_images: usize,
    pub name: Option<String>,
    pub folder_id: Option<String>,
}

impl ImageGenerationSubmission {
    pub fn new(input: GenerationInput, num_images: usize) -> Self {
        Self {
            input,
            num_images: num_images.clamp(1, 4),
            name: None,
            folder_id: None,
        }
    }

    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn with_folder_id(mut self, folder_id: impl Into<String>) -> Self {
        self.folder_id = Some(folder_id.into());
        self
    }

    pub fn build_params(&self, reference_image_urls: Vec<String>) -> BackendGenerationParams {
        BackendGenerationParams::Image(ImageGenerationParams {
            prompt: self.input.prompt.clone(),
            aspect_ratio: self.input.aspect_ratio.clone(),
            resolution: self.input.resolution.clone(),
            quality: self.input.quality.clone(),
            image_urls: reference_image_urls,
            num_images: self.num_images,
        })
    }
}
