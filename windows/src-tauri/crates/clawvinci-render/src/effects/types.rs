// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only
// Derived from Sources/PalmierPro/Compositing/EffectRegistry.swift (GPLv3).

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Specification for a single effect parameter.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct EffectParamSpec {
    pub key: &'static str,
    pub label: &'static str,
    pub min: f64,
    pub max: f64,
    pub default_value: f64,
    pub unit: &'static str,
}

impl EffectParamSpec {
    pub const fn new(
        key: &'static str,
        label: &'static str,
        min: f64,
        max: f64,
        default_value: f64,
        unit: &'static str,
    ) -> Self {
        Self {
            key,
            label,
            min,
            max,
            default_value,
            unit,
        }
    }

    pub fn clamp(&self, val: f64) -> f64 {
        val.clamp(self.min, self.max)
    }
}

/// Parameter values resolved for a single frame.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolvedEffectParams {
    pub values: HashMap<String, f64>,
    pub strings: HashMap<String, String>,
    pub frame: usize,
    pub spatial_scale: f64,
}

impl ResolvedEffectParams {
    pub fn new(
        values: HashMap<String, f64>,
        strings: HashMap<String, String>,
        frame: usize,
        spatial_scale: f64,
    ) -> Self {
        Self {
            values,
            strings,
            frame,
            spatial_scale: if spatial_scale <= 0.0 { 1.0 } else { spatial_scale },
        }
    }

    pub fn value(&self, key: &str) -> f64 {
        self.values.get(key).copied().unwrap_or(0.0)
    }

    pub fn string(&self, key: &str) -> Option<&str> {
        self.strings.get(key).map(|s| s.as_str())
    }
}

/// Static descriptor defining an effect's metadata, parameters, and render kernel.
pub struct EffectDescriptor {
    pub id: &'static str,
    pub display_name: &'static str,
    pub category: &'static str,
    pub params: &'static [EffectParamSpec],
    pub linearizes: bool,
    pub resource_key: Option<&'static str>,
    pub apply: fn(pixels: &mut [u8], width: u32, height: u32, params: &ResolvedEffectParams),
}

impl EffectDescriptor {
    pub const fn new(
        id: &'static str,
        display_name: &'static str,
        category: &'static str,
        params: &'static [EffectParamSpec],
        linearizes: bool,
        resource_key: Option<&'static str>,
        apply: fn(pixels: &mut [u8], width: u32, height: u32, params: &ResolvedEffectParams),
    ) -> Self {
        Self {
            id,
            display_name,
            category,
            params,
            linearizes,
            resource_key,
            apply,
        }
    }
}
