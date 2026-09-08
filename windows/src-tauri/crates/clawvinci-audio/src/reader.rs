// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only
// Derived from Sources/PalmierPro/Audio/AudioTrackReader.swift (GPLv3).

use crate::error::{AudioError, AudioResult};
use clawvinci_media::ffmpeg::FfmpegContext;
use std::ops::Range;
use std::path::Path;

/// Reader for decoding audio streams into mono float32 PCM buffers.
pub struct AudioTrackReader;

impl AudioTrackReader {
    /// Reads decoded mono float32 PCM samples at the specified sample rate.
    pub async fn read_mono_floats(
        ctx: &FfmpegContext,
        path: &Path,
        sample_rate: u32,
        range: Option<Range<f64>>,
    ) -> AudioResult<Vec<f32>> {
        if !path.exists() {
            return Err(AudioError::ReadFailed(format!(
                "File not found: {}",
                path.display()
            )));
        }

        let mut cmd = ctx.command_ffmpeg();

        if let Some(ref r) = range {
            if r.start > 0.0 {
                cmd.arg("-ss").arg(format!("{:.4}", r.start));
            }
        }

        cmd.arg("-i").arg(path);

        if let Some(ref r) = range {
            let duration = (r.end - r.start).max(0.0);
            if duration > 0.0 {
                cmd.arg("-t").arg(format!("{:.4}", duration));
            }
        }

        cmd.arg("-vn"); // Strip video
        cmd.arg("-ac").arg("1"); // Downmix to mono
        cmd.arg("-ar").arg(sample_rate.to_string());
        cmd.arg("-f").arg("f32le"); // Raw 32-bit float Little-Endian
        cmd.arg("pipe:1");

        let output = cmd.output().await.map_err(|e| {
            AudioError::ReadFailed(format!("Failed to execute FFmpeg audio decode: {}", e))
        })?;

        if !output.status.success() {
            let err_msg = String::from_utf8_lossy(&output.stderr);
            return Err(AudioError::ReadFailed(format!(
                "FFmpeg audio decode failed: {}",
                err_msg
            )));
        }

        let bytes = output.stdout;
        if bytes.len() % 4 != 0 {
            return Err(AudioError::ReadFailed(
                "PCM output length is not aligned to 4-byte float samples".into(),
            ));
        }

        let samples: Vec<f32> = bytes
            .as_chunks::<4>()
            .0
            .iter()
            .map(|&chunk| f32::from_le_bytes(chunk))
            .collect();

        Ok(samples)
    }
}
