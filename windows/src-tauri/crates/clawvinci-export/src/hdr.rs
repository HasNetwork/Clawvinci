// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only
// Derived from Sources/PalmierPro/Export/HDRVideoExporter.swift (GPLv3).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum HdrTransfer {
    #[default]
    Hlg,
    Pq,
}

impl HdrTransfer {
    pub fn ffmpeg_trc(&self) -> &'static str {
        match self {
            Self::Hlg => "arib-std-b67",
            Self::Pq => "smpte2084",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HdrConfig {
    pub transfer: HdrTransfer,
    pub primaries: String,
    pub matrix: String,
    pub pixel_format: String,
    pub profile: String,
}

impl Default for HdrConfig {
    fn default() -> Self {
        Self {
            transfer: HdrTransfer::Hlg,
            primaries: "bt2020".to_string(),
            matrix: "bt2020nc".to_string(),
            pixel_format: "yuv420p10le".to_string(),
            profile: "main10".to_string(),
        }
    }
}

impl HdrConfig {
    pub fn new(transfer: HdrTransfer) -> Self {
        Self {
            transfer,
            ..Default::default()
        }
    }

    /// Generates the FFmpeg command line arguments required for HDR10/HLG 10-bit encoding.
    pub fn ffmpeg_args(&self) -> Vec<String> {
        let mut args = vec![
            "-pix_fmt".to_string(),
            self.pixel_format.clone(),
            "-color_primaries".to_string(),
            self.primaries.clone(),
            "-color_trc".to_string(),
            self.transfer.ffmpeg_trc().to_string(),
            "-colorspace".to_string(),
            self.matrix.clone(),
            "-profile:v".to_string(),
            self.profile.clone(),
        ];

        match self.transfer {
            HdrTransfer::Pq => {
                args.push("-x265-params".to_string());
                args.push("hdr10=1:repeat-headers=1".to_string());
            }
            HdrTransfer::Hlg => {
                args.push("-x265-params".to_string());
                args.push("repeat-headers=1".to_string());
            }
        }

        args
    }
}
