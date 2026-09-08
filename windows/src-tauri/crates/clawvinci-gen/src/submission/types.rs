// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only
// Derived from Sources/PalmierPro/Generation/GenerationBackend.swift (GPLv3).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoGenerationParams {
    pub prompt: String,
    pub duration: i32,
    pub aspect_ratio: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolution: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_video_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_frame_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_frame_url: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reference_image_urls: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reference_video_urls: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reference_audio_urls: Vec<String>,
    #[serde(default = "default_true")]
    pub generate_audio: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub draft: Option<bool>,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageGenerationParams {
    pub prompt: String,
    pub aspect_ratio: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolution: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quality: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub image_urls: Vec<String>,
    #[serde(default = "default_one")]
    pub num_images: usize,
}

fn default_one() -> usize {
    1
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioGenerationParams {
    pub prompt: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub voice: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lyrics: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style_instructions: Option<String>,
    #[serde(default)]
    pub instrumental: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_seconds: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub video_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference_image_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference_audio_urls: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpscaleGenerationParams {
    pub media_url: String,
    pub scale_factor: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_resolution: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum BackendGenerationParams {
    Video(VideoGenerationParams),
    Image(ImageGenerationParams),
    Audio(AudioGenerationParams),
    Upscale(UpscaleGenerationParams),
}
