// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only
// Derived from Sources/PalmierPro/Generation/Preprocessing/ImageConverter.swift (GPLv3).

use crate::error::{GenError, GenResult};
use clawvinci_media::ffmpeg::FfmpegContext;
use std::path::{Path, PathBuf};
use uuid::Uuid;

pub struct ImageConverter;

impl ImageConverter {
    pub fn requires_conversion(path: &Path) -> bool {
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            matches!(
                ext.to_lowercase().as_str(),
                "heic" | "heif" | "tiff" | "tif" | "bmp"
            )
        } else {
            false
        }
    }

    /// Converts an input reference image to JPEG format.
    pub async fn convert_to_jpeg(path: &Path, dest_path: Option<&Path>) -> GenResult<PathBuf> {
        let out_path = match dest_path {
            Some(p) => p.to_path_buf(),
            None => std::env::temp_dir().join(format!("img-{}.jpg", Uuid::new_v4())),
        };

        let ctx = FfmpegContext::discover().await.map_err(GenError::Media)?;
        let mut cmd = ctx.command_ffmpeg();

        cmd.arg("-i")
            .arg(path)
            .arg("-q:v")
            .arg("2")
            .arg("-y")
            .arg(&out_path);

        let output = cmd.output().await.map_err(|e| {
            GenError::PreprocessingFailed(format!("Failed to execute ffmpeg image conversion: {e}"))
        })?;

        if !output.status.success() {
            let err = String::from_utf8_lossy(&output.stderr);
            return Err(GenError::PreprocessingFailed(format!(
                "FFmpeg image conversion failed: {err}"
            )));
        }

        Ok(out_path)
    }
}
