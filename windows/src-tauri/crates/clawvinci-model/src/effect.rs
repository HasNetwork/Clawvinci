// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only
// Derived from Sources/PalmierPro/Models/Effect.swift (GPLv3).

use crate::keyframe::KeyframeTrack;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct EffectParam {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub string: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub track: Option<KeyframeTrack<f64>>,
}

impl EffectParam {
    pub fn from_value(value: f64) -> Self {
        Self {
            value: Some(value),
            string: None,
            track: None,
        }
    }

    pub fn from_string(string: impl Into<String>) -> Self {
        Self {
            value: None,
            string: Some(string.into()),
            track: None,
        }
    }

    pub fn from_track(track: KeyframeTrack<f64>) -> Self {
        Self {
            value: None,
            string: None,
            track: Some(track),
        }
    }

    pub fn resolved(&self, offset: i64, default_value: f64) -> f64 {
        if let Some(track) = &self.track {
            if track.is_active() {
                return track.sample(offset, self.value.unwrap_or(default_value));
            }
        }
        self.value.unwrap_or(default_value)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Effect {
    #[serde(default = "default_uuid")]
    pub id: String,
    #[serde(rename = "type")]
    pub effect_type: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub params: HashMap<String, EffectParam>,
}

fn default_uuid() -> String {
    Uuid::new_v4().to_string()
}
fn default_true() -> bool {
    true
}

impl Effect {
    pub const GAUSSIAN_BLUR_TYPE: &'static str = "blur.gaussian";
    pub const GAUSSIAN_BLUR_RADIUS_KEY: &'static str = "radius";
    pub const DENOISE_EFFECT_TYPE: &'static str = "audio.denoise";

    pub fn new(effect_type: impl Into<String>) -> Self {
        Self {
            id: default_uuid(),
            effect_type: effect_type.into(),
            enabled: true,
            params: HashMap::new(),
        }
    }

    pub fn make(effect_type: impl Into<String>, values: &[(&str, f64)]) -> Self {
        let mut params = HashMap::new();
        for (k, v) in values {
            params.insert(k.to_string(), EffectParam::from_value(*v));
        }
        Self {
            id: default_uuid(),
            effect_type: effect_type.into(),
            enabled: true,
            params,
        }
    }
}
