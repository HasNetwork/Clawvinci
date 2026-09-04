// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only
// Derived from Sources/PalmierPro/Models/TextAnimation.swift (GPLv3).

use crate::text_style::Rgba;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WordTiming {
    pub text: String,
    pub start_frame: i64,
    pub end_frame: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum AnimationPreset {
    #[default]
    None,
    PopIn,
    SlideUp,
    Typewriter,
    WordReveal,
    WordSlide,
    HighlightPop,
    HighlightBlock,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextAnimation {
    #[serde(default)]
    pub preset: AnimationPreset,
    #[serde(default = "default_per_word_frames")]
    pub per_word_frames: i64,
    #[serde(default)]
    pub highlight: Option<Rgba>,
}

fn default_per_word_frames() -> i64 {
    6
}

impl Default for TextAnimation {
    fn default() -> Self {
        Self {
            preset: AnimationPreset::None,
            per_word_frames: 6,
            highlight: None,
        }
    }
}
