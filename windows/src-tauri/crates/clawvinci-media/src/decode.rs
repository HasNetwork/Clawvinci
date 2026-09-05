// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only
// Derived from Sources/PalmierPro/Preview/VideoEngine.swift (GPLv3).

use crate::error::{MediaError, MediaResult};
use crate::ffmpeg::FfmpegContext;
use std::path::Path;
use tokio::io::AsyncReadExt;
use tokio::process::ChildStdout;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HwAccelMode {
    #[default]
    Auto,
    D3d11va,
    Cuda,
    None,
}

impl HwAccelMode {
    pub fn as_ffmpeg_arg(&self) -> Option<&'static str> {
        match self {
            Self::Auto => Some("auto"),
            Self::D3d11va => Some("d3d11va"),
            Self::Cuda => Some("cuda"),
            Self::None => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct VideoFrame {
    pub width: u32,
    pub height: u32,
    pub frame_index: usize,
    pub data: Vec<u8>, // RGBA 8-bit per channel
}

#[derive(Debug, Clone)]
pub struct DecodeOptions {
    pub start_seconds: f64,
    pub duration_seconds: Option<f64>,
    pub max_frames: Option<usize>,
    pub target_width: u32,
    pub target_height: u32,
    pub hwaccel: HwAccelMode,
}

impl DecodeOptions {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            start_seconds: 0.0,
            duration_seconds: None,
            max_frames: None,
            target_width: width,
            target_height: height,
            hwaccel: HwAccelMode::Auto,
        }
    }

    pub fn with_start(mut self, start_seconds: f64) -> Self {
        self.start_seconds = start_seconds;
        self
    }

    pub fn with_duration(mut self, duration_seconds: f64) -> Self {
        self.duration_seconds = Some(duration_seconds);
        self
    }

    pub fn with_max_frames(mut self, max_frames: usize) -> Self {
        self.max_frames = Some(max_frames);
        self
    }
}

pub struct VideoStreamReader {
    child: tokio::process::Child,
    stdout: ChildStdout,
    width: u32,
    height: u32,
    frame_size_bytes: usize,
    current_frame: usize,
    max_frames: Option<usize>,
}

impl VideoStreamReader {
    pub fn open(
        ctx: &FfmpegContext,
        path: &Path,
        options: &DecodeOptions,
    ) -> MediaResult<Self> {
        if !path.exists() {
            return Err(MediaError::InvalidInput(format!(
                "Media file not found: {}",
                path.display()
            )));
        }

        let mut cmd = ctx.command_ffmpeg();

        if let Some(hw) = options.hwaccel.as_ffmpeg_arg() {
            cmd.arg("-hwaccel").arg(hw);
        }

        if options.start_seconds > 0.0 {
            cmd.arg("-ss").arg(format!("{:.4}", options.start_seconds));
        }

        cmd.arg("-i").arg(path);

        if let Some(dur) = options.duration_seconds {
            cmd.arg("-t").arg(format!("{:.4}", dur));
        }

        if let Some(max) = options.max_frames {
            cmd.arg("-vframes").arg(max.to_string());
        }

        cmd.arg("-vf").arg(format!(
            "scale={}:{}:force_original_aspect_ratio=decrease,pad={}:{}:(ow-iw)/2:(oh-ih)/2:black",
            options.target_width, options.target_height, options.target_width, options.target_height
        ));

        cmd.arg("-f").arg("rawvideo");
        cmd.arg("-pix_fmt").arg("rgba");
        cmd.arg("pipe:1");

        let mut child = cmd.spawn().map_err(|e| {
            MediaError::DecodeFailed(format!("Failed to spawn FFmpeg video decoder: {}", e))
        })?;

        let stdout = child.stdout.take().ok_or_else(|| {
            MediaError::DecodeFailed("Failed to capture decoder stdout pipe".into())
        })?;

        let frame_size_bytes = (options.target_width as usize) * (options.target_height as usize) * 4;

        Ok(Self {
            child,
            stdout,
            width: options.target_width,
            height: options.target_height,
            frame_size_bytes,
            current_frame: 0,
            max_frames: options.max_frames,
        })
    }

    pub async fn read_next_frame(&mut self) -> MediaResult<Option<VideoFrame>> {
        if let Some(max) = self.max_frames {
            if self.current_frame >= max {
                return Ok(None);
            }
        }

        let mut buffer = vec![0u8; self.frame_size_bytes];
        match self.stdout.read_exact(&mut buffer).await {
            Ok(_) => {
                let frame = VideoFrame {
                    width: self.width,
                    height: self.height,
                    frame_index: self.current_frame,
                    data: buffer,
                };
                self.current_frame += 1;
                Ok(Some(frame))
            }
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => Ok(None),
            Err(e) => Err(MediaError::Io(e)),
        }
    }

    pub async fn close(&mut self) -> MediaResult<()> {
        let _ = self.child.kill().await;
        Ok(())
    }
}

impl Drop for VideoStreamReader {
    fn drop(&mut self) {
        let _ = self.child.start_kill();
    }
}

pub async fn read_audio_samples(
    ctx: &FfmpegContext,
    path: &Path,
    start_seconds: f64,
    duration_seconds: f64,
    target_sample_rate: u32,
) -> MediaResult<Vec<f32>> {
    if !path.exists() {
        return Err(MediaError::InvalidInput(format!(
            "Media file not found: {}",
            path.display()
        )));
    }

    let mut cmd = ctx.command_ffmpeg();
    if start_seconds > 0.0 {
        cmd.arg("-ss").arg(format!("{:.4}", start_seconds));
    }
    cmd.arg("-i").arg(path);
    if duration_seconds > 0.0 {
        cmd.arg("-t").arg(format!("{:.4}", duration_seconds));
    }
    cmd.arg("-vn");
    cmd.arg("-ac").arg("1"); // mono
    cmd.arg("-ar").arg(target_sample_rate.to_string());
    cmd.arg("-f").arg("f32le");
    cmd.arg("pipe:1");

    let output = cmd.output().await.map_err(|e| {
        MediaError::DecodeFailed(format!("Failed to execute audio decode: {}", e))
    })?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        return Err(MediaError::DecodeFailed(format!(
            "FFmpeg audio decode failed: {}",
            err
        )));
    }

    let bytes = output.stdout;
    let samples: Vec<f32> = bytes
        .as_chunks::<4>()
        .0
        .iter()
        .map(|&c| f32::from_le_bytes(c))
        .collect();

    Ok(samples)
}
