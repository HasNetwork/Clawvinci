// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only
// Derived from Sources/PalmierPro/Audio/WaveformExtractor.swift (GPLv3).

use crate::error::{MediaError, MediaResult};
use crate::ffmpeg::FfmpegContext;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WaveformData {
    pub duration_seconds: f64,
    pub buckets_per_second: usize,
    pub peaks: Vec<f32>,
    pub rms: Vec<f32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WaveformOptions {
    pub buckets_per_second: usize,
    pub target_sample_rate: u32,
}

impl Default for WaveformOptions {
    fn default() -> Self {
        Self {
            buckets_per_second: 100, // 10ms buckets
            target_sample_rate: 44100,
        }
    }
}

pub async fn extract_waveform(
    ctx: &FfmpegContext,
    path: &Path,
    options: &WaveformOptions,
) -> MediaResult<WaveformData> {
    if !path.exists() {
        return Err(MediaError::InvalidInput(format!(
            "Media file not found: {}",
            path.display()
        )));
    }

    let mut cmd = ctx.command_ffmpeg();
    cmd.arg("-i").arg(path);
    cmd.arg("-vn"); // No video
    cmd.arg("-ac").arg("1"); // Downmix to mono
    cmd.arg("-ar").arg(options.target_sample_rate.to_string());
    cmd.arg("-f").arg("f32le"); // Raw 32-bit float Little-Endian PCM
    cmd.arg("pipe:1");

    let output = cmd.output().await.map_err(|e| {
        MediaError::WaveformFailed(format!("Failed to execute waveform extraction: {}", e))
    })?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        return Err(MediaError::WaveformFailed(format!(
            "FFmpeg waveform extraction failed: {}",
            err
        )));
    }

    let bytes = output.stdout;
    if bytes.len() % 4 != 0 {
        return Err(MediaError::WaveformFailed(
            "PCM stream length is not aligned to 4-byte f32 floats".into(),
        ));
    }

    let samples: Vec<f32> = bytes
        .chunks_exact(4)
        .map(|chunk| f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
        .collect();

    Ok(downsample_pcm(
        &samples,
        options.target_sample_rate as usize,
        options.buckets_per_second,
    ))
}

pub fn downsample_pcm(
    samples: &[f32],
    source_sample_rate: usize,
    buckets_per_second: usize,
) -> WaveformData {
    let total_samples = samples.len();
    if total_samples == 0 || source_sample_rate == 0 || buckets_per_second == 0 {
        return WaveformData {
            duration_seconds: 0.0,
            buckets_per_second,
            peaks: Vec::new(),
            rms: Vec::new(),
        };
    }

    let duration_seconds = total_samples as f64 / source_sample_rate as f64;
    let samples_per_bucket = (source_sample_rate as f64 / buckets_per_second as f64).max(1.0);
    let total_buckets = (duration_seconds * buckets_per_second as f64).ceil() as usize;

    let mut peaks = Vec::with_capacity(total_buckets);
    let mut rms_list = Vec::with_capacity(total_buckets);

    for b in 0..total_buckets {
        let start_idx = ((b as f64) * samples_per_bucket).floor() as usize;
        let end_idx = (((b + 1) as f64) * samples_per_bucket)
            .ceil()
            .min(total_samples as f64) as usize;

        if start_idx >= total_samples {
            break;
        }

        let slice = &samples[start_idx..end_idx.max(start_idx + 1).min(total_samples)];
        let mut max_abs = 0.0f32;
        let mut sum_sq = 0.0f64;

        for &sample in slice {
            let abs = sample.abs();
            if abs > max_abs {
                max_abs = abs;
            }
            sum_sq += (sample as f64) * (sample as f64);
        }

        let count = slice.len().max(1);
        let rms = (sum_sq / count as f64).sqrt() as f32;

        peaks.push(max_abs.clamp(0.0, 1.0));
        rms_list.push(rms.clamp(0.0, 1.0));
    }

    WaveformData {
        duration_seconds,
        buckets_per_second,
        peaks,
        rms: rms_list,
    }
}
