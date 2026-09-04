// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only
// Derived from Sources/PalmierPro/Models/ClipType.swift (GPLv3).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ClipType {
    #[default]
    Video,
    Audio,
    Image,
    Text,
    Lottie,
    Sequence,
    Subtitle,
}

impl ClipType {
    pub fn track_label(&self) -> &'static str {
        match self {
            Self::Video => "Video",
            Self::Audio => "Audio",
            Self::Image => "Image",
            Self::Text => "Text",
            Self::Lottie => "Lottie",
            Self::Sequence => "Video",
            Self::Subtitle => "Subtitle",
        }
    }

    pub fn track_label_prefix(&self) -> &'static str {
        match self {
            Self::Video => "V",
            Self::Audio => "A",
            Self::Image => "I",
            Self::Text => "T",
            Self::Lottie => "L",
            Self::Sequence => "V",
            Self::Subtitle => "S",
        }
    }

    pub fn is_visual(&self) -> bool {
        !matches!(self, Self::Audio | Self::Subtitle)
    }

    pub fn is_compatible(&self, other: ClipType) -> bool {
        *self == other || (self.is_visual() && other.is_visual())
    }

    pub fn from_file_extension(ext: &str) -> Option<Self> {
        let ext = ext.to_ascii_lowercase();
        match ext.as_str() {
            "mov" | "mp4" | "m4v" => Some(Self::Video),
            "mp3" | "wav" | "aac" | "m4a" | "aiff" | "aif" | "aifc" | "caf" | "flac" => {
                Some(Self::Audio)
            }
            "png" | "jpg" | "jpeg" | "tiff" | "heic" | "webp" => Some(Self::Image),
            "json" | "lottie" => Some(Self::Lottie),
            "srt" | "vtt" => Some(Self::Subtitle),
            _ => None,
        }
    }
}
