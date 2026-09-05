// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only
// Derived from Sources/PalmierPro/Export/ExportService.swift (GPLv3).

use crate::error::{MediaError, MediaResult};
use crate::ffmpeg::FfmpegContext;
use std::path::Path;
use tokio::io::AsyncWriteExt;
use tokio::process::ChildStdin;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum VideoCodec {
    #[default]
    H264,
    Hevc,
    ProRes,
}

impl VideoCodec {
    pub fn ffmpeg_encoder(&self) -> &'static str {
        match self {
            Self::H264 => "libx264",
            Self::Hevc => "libx265",
            Self::ProRes => "prores_ks",
        }
    }
}

#[derive(Debug, Clone)]
pub struct EncodeOptions {
    pub width: u32,
    pub height: u32,
    pub fps: f64,
    pub codec: VideoCodec,
    pub crf: Option<u32>,
    pub bitrate_kbps: Option<u32>,
}

impl EncodeOptions {
    pub fn new(width: u32, height: u32, fps: f64) -> Self {
        Self {
            width,
            height,
            fps,
            codec: VideoCodec::H264,
            crf: Some(18), // visually lossless default
            bitrate_kbps: None,
        }
    }

    pub fn with_codec(mut self, codec: VideoCodec) -> Self {
        self.codec = codec;
        self
    }

    pub fn with_crf(mut self, crf: u32) -> Self {
        self.crf = Some(crf);
        self
    }
}

pub struct VideoStreamWriter {
    child: tokio::process::Child,
    stdin: ChildStdin,
    expected_frame_size: usize,
    frames_written: usize,
}

impl VideoStreamWriter {
    pub fn open(
        ctx: &FfmpegContext,
        output_path: &Path,
        options: &EncodeOptions,
    ) -> MediaResult<Self> {
        let mut cmd = ctx.command_ffmpeg();
        cmd.arg("-y"); // overwrite output
        cmd.arg("-f").arg("rawvideo");
        cmd.arg("-pix_fmt").arg("rgba");
        cmd.arg("-s").arg(format!("{}x{}", options.width, options.height));
        cmd.arg("-r").arg(format!("{:.4}", options.fps));
        cmd.arg("-i").arg("pipe:0");

        cmd.arg("-c:v").arg(options.codec.ffmpeg_encoder());

        if let Some(crf) = options.crf {
            if options.codec != VideoCodec::ProRes {
                cmd.arg("-crf").arg(crf.to_string());
            }
        }

        if let Some(br) = options.bitrate_kbps {
            cmd.arg("-b:v").arg(format!("{}k", br));
        }

        // Fast start for web streaming
        if output_path.extension().is_some_and(|ext| ext == "mp4") {
            cmd.arg("-movflags").arg("+faststart");
            cmd.arg("-pix_fmt").arg("yuv420p");
        }

        cmd.arg(output_path);

        let mut child = cmd.spawn().map_err(|e| {
            MediaError::EncodeFailed(format!("Failed to spawn FFmpeg encoder: {}", e))
        })?;

        let stdin = child.stdin.take().ok_or_else(|| {
            MediaError::EncodeFailed("Failed to capture encoder stdin pipe".into())
        })?;

        let expected_frame_size = (options.width as usize) * (options.height as usize) * 4;

        Ok(Self {
            child,
            stdin,
            expected_frame_size,
            frames_written: 0,
        })
    }

    pub async fn write_frame(&mut self, rgba_data: &[u8]) -> MediaResult<()> {
        if rgba_data.len() != self.expected_frame_size {
            return Err(MediaError::InvalidInput(format!(
                "Frame buffer size mismatch: expected {} bytes, got {}",
                self.expected_frame_size,
                rgba_data.len()
            )));
        }

        self.stdin.write_all(rgba_data).await.map_err(|e| {
            MediaError::EncodeFailed(format!("Failed to write frame to encoder stdin: {}", e))
        })?;

        self.frames_written += 1;
        Ok(())
    }

    pub async fn finish(mut self) -> MediaResult<usize> {
        drop(self.stdin); // Close stdin so FFmpeg knows stream has ended

        let status = self.child.wait().await.map_err(|e| {
            MediaError::EncodeFailed(format!("Failed to wait for encoder process: {}", e))
        })?;

        if !status.success() {
            return Err(MediaError::EncodeFailed(format!(
                "FFmpeg encoder exited with status: {}",
                status
            )));
        }

        Ok(self.frames_written)
    }
}
