// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only
// Derived from Sources/PalmierPro/Editor/ViewModel/EditorViewModel+Speakers.swift (GPLv3).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpeakerRegistryEntry {
    pub id: i64,
    pub name: String,
    pub color: Vec<f64>,
    pub centroid: Vec<f32>,
}

impl SpeakerRegistryEntry {
    pub fn new(id: i64, name: impl Into<String>, color: Vec<f64>, centroid: Vec<f32>) -> Self {
        Self {
            id,
            name: name.into(),
            color,
            centroid,
        }
    }
}
