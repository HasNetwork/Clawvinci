// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only
// Derived from Sources/PalmierPro/Generation/Submission/VideoGenerationSubmission.swift (GPLv3).

use super::types::{BackendGenerationParams, VideoGenerationParams};
use clawvinci_model::media_manifest::GenerationInput;

#[derive(Debug, Clone)]
pub struct VideoGenerationSubmission {
    pub input: GenerationInput,
    pub placeholder_duration: f64,
    pub name: Option<String>,
    pub folder_id: Option<String>,
}

impl VideoGenerationSubmission {
    pub fn new(input: GenerationInput, placeholder_duration: f64) -> Self {
        Self {
            input,
            placeholder_duration,
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

    /// Builds backend parameters using uploaded reference URLs.
    pub fn build_params(
        &self,
        start_frame_url: Option<String>,
        end_frame_url: Option<String>,
        source_video_url: Option<String>,
        reference_image_urls: Vec<String>,
        reference_video_urls: Vec<String>,
        reference_audio_urls: Vec<String>,
    ) -> BackendGenerationParams {
        BackendGenerationParams::Video(VideoGenerationParams {
            prompt: self.input.prompt.clone(),
            duration: self.input.duration,
            aspect_ratio: self.input.aspect_ratio.clone(),
            resolution: self.input.resolution.clone(),
            source_video_url,
            start_frame_url,
            end_frame_url,
            reference_image_urls,
            reference_video_urls,
            reference_audio_urls,
            generate_audio: self.input.generate_audio.unwrap_or(true),
            draft: self.input.draft,
        })
    }
}
