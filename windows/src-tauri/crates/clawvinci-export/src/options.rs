// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only
// Derived from Sources/PalmierPro/Export/ExportOptions.swift (GPLv3).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ExportFormat {
    H264,
    H265,
    ProRes,
    HevcHdr,
    Xml,
    Fcpxml,
}

impl ExportFormat {
    pub fn file_extension(&self) -> &'static str {
        match self {
            Self::H264 | Self::H265 => "mp4",
            Self::ProRes | Self::HevcHdr => "mov",
            Self::Xml => "xml",
            Self::Fcpxml => "fcpxml",
        }
    }

    pub fn is_hdr(&self) -> bool {
        matches!(self, Self::HevcHdr)
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::H264 => "H.264",
            Self::H265 => "H.265",
            Self::ProRes => "ProRes",
            Self::HevcHdr => "HEVC 10-bit HDR",
            Self::Xml => "XML",
            Self::Fcpxml => "FCPXML",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum ExportResolution {
    R720p,
    #[default]
    R1080p,
    R1440p,
    R4k,
    MatchTimeline,
    Custom { width: u32, height: u32 },
}

impl ExportResolution {
    pub fn short_side_pixels(&self) -> Option<u32> {
        match self {
            Self::R720p => Some(720),
            Self::R1080p => Some(1080),
            Self::R1440p => Some(1440),
            Self::R4k => Some(2160),
            Self::MatchTimeline | Self::Custom { .. } => None,
        }
    }

    pub fn render_size(&self, canvas_w: u32, canvas_h: u32) -> (u32, u32) {
        if let Self::Custom { width, height } = *self {
            return Self::even_size(width, height);
        }
        guard_render_size(self.short_side_pixels(), canvas_w, canvas_h)
    }

    fn even_size(w: u32, h: u32) -> (u32, u32) {
        let ew = ((w / 2) * 2).max(2);
        let eh = ((h / 2) * 2).max(2);
        (ew, eh)
    }
}

fn guard_render_size(short_side: Option<u32>, canvas_w: u32, canvas_h: u32) -> (u32, u32) {
    match short_side {
        Some(target_short) => {
            let canvas_short = canvas_w.min(canvas_h);
            if canvas_short == 0 {
                return (canvas_w.max(2), canvas_h.max(2));
            }
            let scale = target_short as f64 / canvas_short as f64;
            let w = (canvas_w as f64 * scale).round() as u32;
            let h = (canvas_h as f64 * scale).round() as u32;
            let ew = ((w / 2) * 2).max(2);
            let eh = ((h / 2) * 2).max(2);
            (ew, eh)
        }
        None => {
            let ew = ((canvas_w / 2) * 2).max(2);
            let eh = ((canvas_h / 2) * 2).max(2);
            (ew, eh)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum VideoCodec {
    #[default]
    H264,
    H265,
    ProRes,
    Hdr,
}

impl VideoCodec {
    pub fn container_label(&self) -> &'static str {
        match self {
            Self::H264 | Self::H265 => "MPEG-4 (.mp4)",
            Self::ProRes | Self::Hdr => "QuickTime (.mov)",
        }
    }

    pub fn export_format(&self) -> ExportFormat {
        match self {
            Self::H264 => ExportFormat::H264,
            Self::H265 => ExportFormat::H265,
            Self::ProRes => ExportFormat::ProRes,
            Self::Hdr => ExportFormat::HevcHdr,
        }
    }

    pub fn media_codec(&self) -> clawvinci_media::encode::VideoCodec {
        match self {
            Self::H264 => clawvinci_media::encode::VideoCodec::H264,
            Self::H265 | Self::Hdr => clawvinci_media::encode::VideoCodec::Hevc,
            Self::ProRes => clawvinci_media::encode::VideoCodec::ProRes,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum FCPXMLVersion {
    #[default]
    V1_10,
    V1_11,
    V1_12,
    V1_13,
    V1_14,
}

impl FCPXMLVersion {
    pub fn version_string(&self) -> &'static str {
        match self {
            Self::V1_10 => "1.10",
            Self::V1_11 => "1.11",
            Self::V1_12 => "1.12",
            Self::V1_13 => "1.13",
            Self::V1_14 => "1.14",
        }
    }

    pub fn compatibility_note(&self) -> &'static str {
        match self {
            Self::V1_10 => "DaVinci Resolve 18+, Final Cut Pro 10.6+",
            Self::V1_11 => "DaVinci Resolve 21+, Final Cut Pro 10.7+",
            Self::V1_12 => "DaVinci Resolve 21+, Final Cut Pro 10.8+",
            Self::V1_13 => "DaVinci Resolve 21+, Final Cut Pro 11+",
            Self::V1_14 => "DaVinci Resolve 21+, Final Cut Pro 12+",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum FCPXMLTarget {
    #[default]
    Resolve,
    FinalCutPro,
}

impl FCPXMLTarget {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Resolve => "DaVinci Resolve",
            Self::FinalCutPro => "Final Cut Pro",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TimelineExportFormat {
    Fcpxml,
    Xmeml,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoExportOptions {
    pub resolution: ExportResolution,
    pub codec: VideoCodec,
    pub crf: Option<u32>,
    pub bitrate_kbps: Option<u32>,
    pub is_hdr: bool,
    pub custom_fps: Option<f64>,
}

impl Default for VideoExportOptions {
    fn default() -> Self {
        Self {
            resolution: ExportResolution::R1080p,
            codec: VideoCodec::H264,
            crf: Some(18),
            bitrate_kbps: None,
            is_hdr: false,
            custom_fps: None,
        }
    }
}
