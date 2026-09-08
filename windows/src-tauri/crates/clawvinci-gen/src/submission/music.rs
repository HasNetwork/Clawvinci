// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only
// Derived from Sources/PalmierPro/Generation/Submission/MusicGenerationSubmission.swift (GPLv3).

use super::types::{AudioGenerationParams, BackendGenerationParams};
use clawvinci_model::media_manifest::GenerationInput;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MusicMode {
    TextToMusic,
    VideoToMusic,
}

#[derive(Debug, Clone)]
pub struct MusicGenerationSubmission {
    pub mode: MusicMode,
    pub input: GenerationInput,
    pub duration_seconds: i32,
    pub name: Option<String>,
    pub start_frame: i64,
}

impl MusicGenerationSubmission {
    pub fn new(mode: MusicMode, input: GenerationInput, duration_seconds: i32, start_frame: i64) -> Self {
        Self {
            mode,
            input,
            duration_seconds: duration_seconds.max(1),
            name: None,
            start_frame,
        }
    }

    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn build_params(&self, video_url: Option<String>) -> BackendGenerationParams {
        BackendGenerationParams::Audio(AudioGenerationParams {
            prompt: self.input.prompt.clone(),
            voice: None,
            lyrics: self.input.lyrics.clone(),
            style_instructions: self.input.style_instructions.clone(),
            instrumental: self.input.instrumental.unwrap_or(false),
            duration_seconds: Some(self.duration_seconds),
            video_url,
            reference_image_url: None,
            reference_audio_urls: None,
            source_url: None,
        })
    }
}
