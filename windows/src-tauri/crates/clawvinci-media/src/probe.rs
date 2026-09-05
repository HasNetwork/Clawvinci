// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only
// Derived from Sources/PalmierPro/Models/MediaResolver.swift (GPLv3).

use crate::error::{MediaError, MediaResult};
use crate::ffmpeg::FfmpegContext;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoStreamInfo {
    pub index: usize,
    pub codec_name: String,
    pub width: u32,
    pub height: u32,
    pub fps: f64,
    pub fps_numerator: i32,
    pub fps_denominator: i32,
    pub duration_frames: i64,
    pub pixel_format: String,
    pub color_space: Option<String>,
    pub color_primaries: Option<String>,
    pub color_transfer: Option<String>,
    pub rotation: i32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioStreamInfo {
    pub index: usize,
    pub codec_name: String,
    pub sample_rate: u32,
    pub channels: u32,
    pub channel_layout: String,
    pub bitrate: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaProbe {
    pub file_path: PathBuf,
    pub format_name: String,
    pub duration_seconds: f64,
    pub size_bytes: u64,
    pub bitrate: u64,
    pub video_streams: Vec<VideoStreamInfo>,
    pub audio_streams: Vec<AudioStreamInfo>,
}

impl MediaProbe {
    pub fn primary_video(&self) -> Option<&VideoStreamInfo> {
        self.video_streams.first()
    }

    pub fn primary_audio(&self) -> Option<&AudioStreamInfo> {
        self.audio_streams.first()
    }

    pub fn duration_frames(&self, target_fps: f64) -> i64 {
        if let Some(video) = self.primary_video() {
            if (video.fps - target_fps).abs() < 0.01 && video.duration_frames > 0 {
                return video.duration_frames;
            }
        }
        (self.duration_seconds * target_fps).round() as i64
    }
}

// Internal raw deserialization structs from ffprobe JSON
#[derive(Deserialize)]
struct FfprobeOutput {
    streams: Option<Vec<FfprobeStream>>,
    format: Option<FfprobeFormat>,
}

#[derive(Deserialize)]
struct FfprobeFormat {
    format_name: Option<String>,
    duration: Option<String>,
    size: Option<String>,
    bit_rate: Option<String>,
}

#[derive(Deserialize)]
struct FfprobeStream {
    index: usize,
    codec_type: Option<String>,
    codec_name: Option<String>,
    width: Option<u32>,
    height: Option<u32>,
    r_frame_rate: Option<String>,
    nb_frames: Option<String>,
    pix_fmt: Option<String>,
    color_space: Option<String>,
    color_primaries: Option<String>,
    color_transfer: Option<String>,
    sample_rate: Option<String>,
    channels: Option<u32>,
    channel_layout: Option<String>,
    bit_rate: Option<String>,
    side_data_list: Option<Vec<FfprobeSideData>>,
    tags: Option<serde_json::Value>,
}

#[derive(Deserialize)]
struct FfprobeSideData {
    rotation: Option<i32>,
}

pub async fn probe_media(ctx: &FfmpegContext, path: &Path) -> MediaResult<MediaProbe> {
    if !path.exists() {
        return Err(MediaError::InvalidInput(format!(
            "Media file not found: {}",
            path.display()
        )));
    }

    let mut cmd = ctx.command_ffprobe();
    cmd.arg("-v")
        .arg("quiet")
        .arg("-print_format")
        .arg("json")
        .arg("-show_format")
        .arg("-show_streams")
        .arg(path);

    let output = cmd.output().await.map_err(|e| {
        MediaError::ProbeFailed(format!("Failed to execute ffprobe: {}", e))
    })?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        return Err(MediaError::ProbeFailed(format!(
            "ffprobe exited with error: {}",
            err
        )));
    }

    let json_text = String::from_utf8_lossy(&output.stdout);
    parse_probe_json(path, &json_text)
}

pub fn parse_probe_json(path: &Path, json_text: &str) -> MediaResult<MediaProbe> {
    let parsed: FfprobeOutput = serde_json::from_str(json_text)?;

    let format = parsed
        .format
        .ok_or_else(|| MediaError::ProbeFailed("Missing format information in ffprobe output".into()))?;

    let duration_seconds = format
        .duration
        .as_deref()
        .and_then(|d| d.parse::<f64>().ok())
        .unwrap_or(0.0);

    let size_bytes = format
        .size
        .as_deref()
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(0);

    let bitrate = format
        .bit_rate
        .as_deref()
        .and_then(|b| b.parse::<u64>().ok())
        .unwrap_or(0);

    let mut video_streams = Vec::new();
    let mut audio_streams = Vec::new();

    if let Some(streams) = parsed.streams {
        for s in streams {
            let codec_type = s.codec_type.as_deref().unwrap_or_default();
            if codec_type == "video" {
                let (fps_num, fps_den, fps) = parse_rational_fps(s.r_frame_rate.as_deref());
                let nb_frames = s
                    .nb_frames
                    .as_deref()
                    .and_then(|n| n.parse::<i64>().ok())
                    .unwrap_or_else(|| (duration_seconds * fps).round() as i64);

                let rotation = s
                    .side_data_list
                    .as_ref()
                    .and_then(|sd| sd.iter().find_map(|d| d.rotation))
                    .or_else(|| {
                        s.tags.as_ref().and_then(|t| {
                            t.get("rotate")
                                .and_then(|r| r.as_str())
                                .and_then(|r| r.parse::<i32>().ok())
                        })
                    })
                    .unwrap_or(0);

                video_streams.push(VideoStreamInfo {
                    index: s.index,
                    codec_name: s.codec_name.unwrap_or_else(|| "unknown".to_string()),
                    width: s.width.unwrap_or(0),
                    height: s.height.unwrap_or(0),
                    fps,
                    fps_numerator: fps_num,
                    fps_denominator: fps_den,
                    duration_frames: nb_frames,
                    pixel_format: s.pix_fmt.unwrap_or_else(|| "yuv420p".to_string()),
                    color_space: s.color_space,
                    color_primaries: s.color_primaries,
                    color_transfer: s.color_transfer,
                    rotation,
                });
            } else if codec_type == "audio" {
                let sample_rate = s
                    .sample_rate
                    .as_deref()
                    .and_then(|sr| sr.parse::<u32>().ok())
                    .unwrap_or(48000);

                let channels = s.channels.unwrap_or(2);
                let channel_layout = s
                    .channel_layout
                    .unwrap_or_else(|| if channels == 1 { "mono".into() } else { "stereo".into() });

                let audio_bitrate = s
                    .bit_rate
                    .as_deref()
                    .and_then(|b| b.parse::<u64>().ok())
                    .unwrap_or(0);

                audio_streams.push(AudioStreamInfo {
                    index: s.index,
                    codec_name: s.codec_name.unwrap_or_else(|| "unknown".to_string()),
                    sample_rate,
                    channels,
                    channel_layout,
                    bitrate: audio_bitrate,
                });
            }
        }
    }

    Ok(MediaProbe {
        file_path: path.to_path_buf(),
        format_name: format.format_name.unwrap_or_else(|| "unknown".to_string()),
        duration_seconds,
        size_bytes,
        bitrate,
        video_streams,
        audio_streams,
    })
}

fn parse_rational_fps(r: Option<&str>) -> (i32, i32, f64) {
    if let Some(r_str) = r {
        let parts: Vec<&str> = r_str.split('/').collect();
        if parts.len() == 2 {
            if let (Ok(num), Ok(den)) = (parts[0].parse::<i32>(), parts[1].parse::<i32>()) {
                if den > 0 {
                    let fps = num as f64 / den as f64;
                    return (num, den, fps);
                }
            }
        } else if let Ok(val) = r_str.parse::<f64>() {
            return (val.round() as i32, 1, val);
        }
    }
    (30, 1, 30.0)
}
