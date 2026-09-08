// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only
// Derived from Sources/PalmierPro/Generation/Edit/EditAction.swift (GPLv3).

use clawvinci_model::clip_type::ClipType;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum EditActionKind {
    Upscale,
    Edit,
    Rerun,
    LipSync,
    Reframe,
    GenerateMusic,
    GenerateSfx,
    CreateVideo,
    EnhanceDraft,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EditActionAvailability {
    Available,
    Disabled { reason: String },
}

impl EditActionAvailability {
    pub fn is_available(&self) -> bool {
        matches!(self, Self::Available)
    }
}

impl EditActionKind {
    pub fn requires_paid_plan(&self) -> bool {
        matches!(
            self,
            Self::Upscale | Self::Edit | Self::LipSync | Self::Reframe
        )
    }

    pub fn availability(&self, media_type: ClipType, is_generating: bool) -> EditActionAvailability {
        if is_generating {
            return EditActionAvailability::Disabled {
                reason: "Generation in progress".to_string(),
            };
        }

        match self {
            Self::Upscale => {
                if matches!(media_type, ClipType::Video | ClipType::Image) {
                    EditActionAvailability::Available
                } else {
                    EditActionAvailability::Disabled {
                        reason: "Upscale only works on video or image assets".to_string(),
                    }
                }
            }
            Self::Reframe | Self::LipSync => {
                if media_type == ClipType::Video {
                    EditActionAvailability::Available
                } else {
                    EditActionAvailability::Disabled {
                        reason: "Only applicable to video assets".to_string(),
                    }
                }
            }
            Self::GenerateMusic | Self::GenerateSfx => EditActionAvailability::Available,
            Self::CreateVideo => {
                if media_type == ClipType::Image {
                    EditActionAvailability::Available
                } else {
                    EditActionAvailability::Disabled {
                        reason: "Image-to-video requires an image asset".to_string(),
                    }
                }
            }
            Self::Edit | Self::Rerun | Self::EnhanceDraft => EditActionAvailability::Available,
        }
    }

    pub fn available_for(media_type: ClipType, is_generating: bool) -> Vec<Self> {
        let candidates = match media_type {
            ClipType::Image => vec![
                Self::Upscale,
                Self::Edit,
                Self::Rerun,
                Self::CreateVideo,
            ],
            ClipType::Video => vec![
                Self::Upscale,
                Self::Edit,
                Self::Rerun,
                Self::LipSync,
                Self::Reframe,
                Self::EnhanceDraft,
                Self::GenerateMusic,
                Self::GenerateSfx,
            ],
            ClipType::Audio => vec![Self::Edit, Self::Rerun, Self::GenerateMusic],
            _ => Vec::new(),
        };

        candidates
            .into_iter()
            .filter(|a| a.availability(media_type, is_generating).is_available())
            .collect()
    }
}
