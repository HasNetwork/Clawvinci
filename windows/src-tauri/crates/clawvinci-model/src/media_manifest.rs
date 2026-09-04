// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only
// Derived from Sources/PalmierPro/Models/MediaManifest.swift and MediaFolder.swift (GPLv3).

use crate::clip_type::ClipType;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

fn default_uuid() -> String {
    Uuid::new_v4().to_string()
}
fn default_manifest_version() -> i32 {
    2
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaFolder {
    #[serde(default = "default_uuid")]
    pub id: String,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_folder_id: Option<String>,
}

impl MediaFolder {
    pub fn new(name: impl Into<String>, parent_folder_id: Option<String>) -> Self {
        Self {
            id: default_uuid(),
            name: name.into(),
            parent_folder_id,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MediaSource {
    #[serde(rename = "external")]
    External {
        #[serde(rename = "absolutePath")]
        absolute_path: String,
    },
    #[serde(rename = "project")]
    Project {
        #[serde(rename = "relativePath")]
        relative_path: String,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct MediaImportInput {
    #[serde(
        rename = "sourceURL",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub source_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct UpscaleSettings {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scale: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerationInput {
    pub prompt: String,
    pub model: String,
    pub duration: i32,
    pub aspect_ratio: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resolution: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub upscale_settings: Option<UpscaleSettings>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub upscale_source_width: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub upscale_source_height: Option<i32>,
    #[serde(
        rename = "upscaleSourceFPS",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub upscale_source_fps: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quality: Option<String>,
    #[serde(
        rename = "imageURLs",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub image_urls: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub num_images: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub voice: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lyrics: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub style_instructions: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub instrumental: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_language: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub multilingual: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub audio_input: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub generate_audio: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub draft: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uses_source_video: Option<bool>,
    #[serde(
        rename = "referenceImageURLs",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub reference_image_urls: Option<Vec<String>>,
    #[serde(
        rename = "referenceVideoURLs",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub reference_video_urls: Option<Vec<String>>,
    #[serde(
        rename = "referenceAudioURLs",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub reference_audio_urls: Option<Vec<String>>,
    #[serde(
        rename = "imageURLAssetIds",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub image_url_asset_ids: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reference_image_asset_ids: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reference_video_asset_ids: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reference_audio_asset_ids: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub backend_job_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_index: Option<i32>,
    #[serde(
        rename = "resultURLs",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub result_urls: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cost_credits: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub refunded_credits: Option<i32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaManifestEntry {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub entry_type: ClipType,
    pub source: MediaSource,
    pub duration: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub generation_input: Option<GenerationInput>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_width: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_height: Option<i32>,
    #[serde(
        rename = "sourceFPS",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub source_fps: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub has_audio: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub folder_id: Option<String>,
    #[serde(
        rename = "cachedRemoteURL",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub cached_remote_url: Option<String>,
    #[serde(
        rename = "cachedRemoteURLExpiresAt",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub cached_remote_url_expires_at: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub generation_status: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub import_input: Option<MediaImportInput>,
}

impl MediaManifestEntry {
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        entry_type: ClipType,
        source: MediaSource,
        duration: f64,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            entry_type,
            source,
            duration,
            generation_input: None,
            source_width: None,
            source_height: None,
            source_fps: None,
            has_audio: None,
            folder_id: None,
            cached_remote_url: None,
            cached_remote_url_expires_at: None,
            generation_status: None,
            import_input: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaManifest {
    #[serde(default = "default_manifest_version")]
    pub version: i32,
    #[serde(default)]
    pub entries: Vec<MediaManifestEntry>,
    #[serde(default)]
    pub folders: Vec<MediaFolder>,
}

impl Default for MediaManifest {
    fn default() -> Self {
        Self {
            version: 2,
            entries: Vec::new(),
            folders: Vec::new(),
        }
    }
}
