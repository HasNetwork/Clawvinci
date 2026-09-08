// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only
// Derived from Sources/PalmierPro/Generation/Catalog/ModelPreferences.swift (GPLv3).

use super::models::ModelModality;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelPreferences {
    pub disabled_ids: HashSet<String>,
    pub default_video_model: String,
    pub default_image_model: String,
    pub default_speech_model: String,
    pub default_music_model: String,
    pub default_upscale_model: String,
}

impl Default for ModelPreferences {
    fn default() -> Self {
        Self {
            disabled_ids: HashSet::new(),
            default_video_model: "seedance-v1-fast".to_string(),
            default_image_model: "nano-banana-pro".to_string(),
            default_speech_model: "eleven-multilingual-v2".to_string(),
            default_music_model: "suno-v3-5".to_string(),
            default_upscale_model: "real-esrgan-4x".to_string(),
        }
    }
}

impl ModelPreferences {
    pub fn is_enabled(&self, id: &str) -> bool {
        !self.disabled_ids.contains(id)
    }

    pub fn set_enabled(&mut self, id: impl Into<String>, enabled: bool) {
        let id = id.into();
        if enabled {
            self.disabled_ids.remove(&id);
        } else {
            self.disabled_ids.insert(id);
        }
    }

    pub fn default_for_modality(&self, modality: ModelModality) -> &str {
        match modality {
            ModelModality::Video => &self.default_video_model,
            ModelModality::Image => &self.default_image_model,
            ModelModality::Audio => &self.default_speech_model,
            ModelModality::Music => &self.default_music_model,
            ModelModality::Upscale => &self.default_upscale_model,
        }
    }
}
