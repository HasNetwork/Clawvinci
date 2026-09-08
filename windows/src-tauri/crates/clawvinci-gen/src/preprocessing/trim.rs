// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only
// Derived from Sources/PalmierPro/Generation/Preprocessing/VideoTrimExtractor.swift (GPLv3).

use crate::error::{GenError, GenResult};
use clawvinci_media::ffmpeg::FfmpegContext;
use std::path::{Path, PathBuf};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct TrimmedSource {
    pub source_path: PathBuf,
    pub trim_start_frame: i64,
    pub source_frames_consumed: i64,
    pub fps: f64,
}

impl TrimmedSource {
    pub fn new(
        source_path: impl Into<PathBuf>,
        trim_start_frame: i64,
        source_frames_consumed: i64,
        fps: f64,
    ) -> Self {
        Self {
            source_path: source_path.into(),
            trim_start_frame,
            source_frames_consumed,
            fps,
        }
    }

    pub fn start_time_seconds(&self) -> f64 {
        if self.fps > 0.0 {
            self.trim_start_frame.max(0) as f64 / self.fps
        } else {
            0.0
        }
    }

    pub fn duration_seconds(&self) -> f64 {
        if self.fps > 0.0 {
            self.source_frames_consumed.max(0) as f64 / self.fps
        } else {
            0.0
        }
    }
}

pub struct VideoTrimExtractor;

impl VideoTrimExtractor {
    /// Extracts a slice of a video clip to a temporary (or specified) MP4 file.
    pub async fn extract(trim: &TrimmedSource, dest_path: Option<&Path>) -> GenResult<PathBuf> {
        if trim.fps <= 0.0 {
            return Err(GenError::PreprocessingFailed(format!(
                "Invalid fps: {}",
                trim.fps
            )));
        }
        if trim.source_frames_consumed <= 0 {
            return Err(GenError::PreprocessingFailed(
                "Empty frame range".to_string(),
            ));
        }

        let out_path = match dest_path {
            Some(p) => p.to_path_buf(),
            None => std::env::temp_dir().join(format!("trim-{}.mp4", Uuid::new_v4())),
        };

        let start_sec = trim.start_time_seconds();
        let dur_sec = trim.duration_seconds();

        let ctx = FfmpegContext::discover().await.map_err(GenError::Media)?;
        let mut cmd = ctx.command_ffmpeg();

        cmd.arg("-ss")
            .arg(format!("{start_sec:.3}"))
            .arg("-t")
            .arg(format!("{dur_sec:.3}"))
            .arg("-i")
            .arg(&trim.source_path)
            .arg("-c:v")
            .arg("libx264")
            .arg("-preset")
            .arg("veryfast")
            .arg("-crf")
            .arg("20")
            .arg("-c:a")
            .arg("aac")
            .arg("-y")
            .arg(&out_path);

        let output = cmd.output().await.map_err(|e| {
            GenError::PreprocessingFailed(format!("Failed to execute ffmpeg trim: {e}"))
        })?;

        if !output.status.success() {
            let err = String::from_utf8_lossy(&output.stderr);
            return Err(GenError::PreprocessingFailed(format!(
                "FFmpeg trim exited with error: {err}"
            )));
        }

        Ok(out_path)
    }
}
