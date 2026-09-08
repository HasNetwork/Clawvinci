// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only
// Derived from Sources/PalmierPro/Generation/Submission/AudioGenerationSubmission.swift (GPLv3).

use super::types::{AudioGenerationParams, BackendGenerationParams};
use clawvinci_model::media_manifest::GenerationInput;

#[derive(Debug, Clone)]
pub struct AudioGenerationSubmission {
    pub input: GenerationInput,
    pub voice: Option<String>,
    pub lyrics: Option<String>,
    pub style_instructions: Option<String>,
    pub instrumental: bool,
    pub duration_seconds: Option<i32>,
    pub name: Option<String>,
    pub folder_id: Option<String>,
}

impl AudioGenerationSubmission {
    pub fn new(input: GenerationInput) -> Self {
        let voice = input.voice.clone();
        let lyrics = input.lyrics.clone();
        let style = input.style_instructions.clone();
        let instrumental = input.instrumental.unwrap_or(false);
        let duration = if input.duration > 0 {
            Some(input.duration)
        } else {
            None
        };
        Self {
            input,
            voice,
            lyrics,
            style_instructions: style,
            instrumental,
            duration_seconds: duration,
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

    pub fn build_params(
        &self,
        reference_image_url: Option<String>,
        reference_audio_urls: Option<Vec<String>>,
        source_url: Option<String>,
        video_url: Option<String>,
    ) -> BackendGenerationParams {
        BackendGenerationParams::Audio(AudioGenerationParams {
            prompt: self.input.prompt.clone(),
            voice: self.voice.clone(),
            lyrics: self.lyrics.clone(),
            style_instructions: self.style_instructions.clone(),
            instrumental: self.instrumental,
            duration_seconds: self.duration_seconds,
            video_url,
            reference_image_url,
            reference_audio_urls,
            source_url,
        })
    }
}
