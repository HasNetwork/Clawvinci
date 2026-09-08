// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only
// Derived from Sources/PalmierPro/Generation/Preprocessing/AudioTrackExtractor.swift (GPLv3).

use crate::error::{GenError, GenResult};
use crate::preprocessing::trim::TrimmedSource;
use clawvinci_media::ffmpeg::FfmpegContext;
use std::path::{Path, PathBuf};
use uuid::Uuid;

pub struct AudioTrackExtractor;

impl AudioTrackExtractor {
    /// Extracts the audio track from a video/audio source to a WAV or M4A file.
    pub async fn extract(
        source_path: &Path,
        trim: Option<&TrimmedSource>,
        dest_path: Option<&Path>,
    ) -> GenResult<PathBuf> {
        let out_path = match dest_path {
            Some(p) => p.to_path_buf(),
            None => std::env::temp_dir().join(format!("audio-{}.wav", Uuid::new_v4())),
        };

        let ctx = FfmpegContext::discover().await.map_err(GenError::Media)?;
        let mut cmd = ctx.command_ffmpeg();

        if let Some(t) = trim {
            let start = t.start_time_seconds();
            let dur = t.duration_seconds();
            cmd.arg("-ss")
                .arg(format!("{start:.3}"))
                .arg("-t")
                .arg(format!("{dur:.3}"));
        }

        cmd.arg("-i")
            .arg(source_path)
            .arg("-vn")
            .arg("-acodec")
            .arg("pcm_s16le")
            .arg("-ar")
            .arg("44100")
            .arg("-ac")
            .arg("2")
            .arg("-y")
            .arg(&out_path);

        let output = cmd.output().await.map_err(|e| {
            GenError::PreprocessingFailed(format!("Failed to execute ffmpeg audio extraction: {e}"))
        })?;

        if !output.status.success() {
            let err = String::from_utf8_lossy(&output.stderr);
            return Err(GenError::PreprocessingFailed(format!(
                "FFmpeg audio extraction exited with error: {err}"
            )));
        }

        Ok(out_path)
    }
}
