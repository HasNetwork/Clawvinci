// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only
// Derived from Sources/PalmierPro/Models/MediaResolver.swift (GPLv3).

use crate::error::{MediaError, MediaResult};
use std::path::{Path, PathBuf};
use std::process::Stdio;
use tokio::process::Command;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;

#[derive(Debug, Clone)]
pub struct FfmpegContext {
    pub ffmpeg_path: PathBuf,
    pub ffprobe_path: PathBuf,
    pub version: String,
}

impl FfmpegContext {
    pub async fn discover() -> MediaResult<Self> {
        let (ffmpeg, ffprobe) = Self::find_binaries()?;
        let version = Self::probe_version(&ffmpeg).await?;
        Ok(Self {
            ffmpeg_path: ffmpeg,
            ffprobe_path: ffprobe,
            version,
        })
    }

    pub fn with_paths(ffmpeg_path: PathBuf, ffprobe_path: PathBuf) -> Self {
        Self {
            ffmpeg_path,
            ffprobe_path,
            version: "custom".to_string(),
        }
    }

    pub fn command_ffmpeg(&self) -> Command {
        let mut cmd = Command::new(&self.ffmpeg_path);
        cmd.stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        #[cfg(windows)]
        cmd.creation_flags(CREATE_NO_WINDOW);
        cmd
    }

    pub fn command_ffprobe(&self) -> Command {
        let mut cmd = Command::new(&self.ffprobe_path);
        cmd.stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        #[cfg(windows)]
        cmd.creation_flags(CREATE_NO_WINDOW);
        cmd
    }

    fn find_binaries() -> MediaResult<(PathBuf, PathBuf)> {
        // 1. Check explicit environment override
        if let Ok(env_path) = std::env::var("FFMPEG_PATH") {
            let p = PathBuf::from(env_path);
            if p.exists() {
                let probe = p.with_file_name("ffprobe.exe");
                let probe_fallback = p.with_file_name("ffprobe");
                let ffprobe = if probe.exists() {
                    probe
                } else if probe_fallback.exists() {
                    probe_fallback
                } else {
                    p.clone()
                };
                return Ok((p, ffprobe));
            }
        }

        // 2. Check common Windows installation paths
        let candidate_dirs = [
            r"D:\Program Files\ffmpeg\bin",
            r"C:\Program Files\ffmpeg\bin",
            r"D:\ffmpeg\bin",
            r"C:\ffmpeg\bin",
            r"C:\ProgramData\chocolatey\bin",
        ];

        for dir in &candidate_dirs {
            let ffmpeg = Path::new(dir).join("ffmpeg.exe");
            let ffprobe = Path::new(dir).join("ffprobe.exe");
            if ffmpeg.exists() && ffprobe.exists() {
                return Ok((ffmpeg, ffprobe));
            }
        }

        // 3. Search in system PATH
        if let Ok(path_var) = std::env::var("PATH") {
            let exe_name_ffmpeg = if cfg!(windows) { "ffmpeg.exe" } else { "ffmpeg" };
            let exe_name_probe = if cfg!(windows) { "ffprobe.exe" } else { "ffprobe" };
            for part in std::env::split_paths(&path_var) {
                let ffmpeg = part.join(exe_name_ffmpeg);
                let ffprobe = part.join(exe_name_probe);
                if ffmpeg.is_file() && ffprobe.is_file() {
                    return Ok((ffmpeg, ffprobe));
                }
            }
        }

        // Fallback: assume ffmpeg / ffprobe available in PATH
        let ffmpeg = PathBuf::from(if cfg!(windows) { "ffmpeg.exe" } else { "ffmpeg" });
        let ffprobe = PathBuf::from(if cfg!(windows) { "ffprobe.exe" } else { "ffprobe" });
        Ok((ffmpeg, ffprobe))
    }

    async fn probe_version(ffmpeg: &Path) -> MediaResult<String> {
        let mut cmd = Command::new(ffmpeg);
        cmd.arg("-version");
        #[cfg(windows)]
        cmd.creation_flags(CREATE_NO_WINDOW);

        match cmd.output().await {
            Ok(output) if output.status.success() => {
                let text = String::from_utf8_lossy(&output.stdout);
                let first_line = text.lines().next().unwrap_or("FFmpeg unknown version");
                Ok(first_line.trim().to_string())
            }
            Ok(output) => {
                let err = String::from_utf8_lossy(&output.stderr);
                Err(MediaError::FfmpegNotFound(format!(
                    "FFmpeg returned non-zero exit code: {}",
                    err
                )))
            }
            Err(e) => Err(MediaError::FfmpegNotFound(format!(
                "Failed to execute FFmpeg at {}: {}",
                ffmpeg.display(),
                e
            ))),
        }
    }
}
