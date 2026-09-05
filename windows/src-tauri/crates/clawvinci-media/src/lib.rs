// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only
// Derived from Sources/PalmierPro/Models/MediaResolver.swift (GPLv3).

pub mod concurrency;
pub mod decode;
pub mod encode;
pub mod error;
pub mod ffmpeg;
pub mod probe;
pub mod thumbnail;
pub mod waveform;

pub use concurrency::MediaConcurrencyLimiter;
pub use decode::{DecodeOptions, HwAccelMode, VideoFrame, VideoStreamReader, read_audio_samples};
pub use encode::{EncodeOptions, VideoCodec, VideoStreamWriter};
pub use error::{MediaError, MediaResult};
pub use ffmpeg::FfmpegContext;
pub use probe::{AudioStreamInfo, MediaProbe, VideoStreamInfo, parse_probe_json, probe_media};
pub use thumbnail::{
    ThumbnailCache, ThumbnailFormat, ThumbnailOptions, extract_thumbnail, extract_thumbnail_cached,
};
pub use waveform::{WaveformData, WaveformOptions, downsample_pcm, extract_waveform};
pub use tokio_util::sync::CancellationToken;

pub mod prelude {
    pub use crate::concurrency::MediaConcurrencyLimiter;
    pub use crate::decode::{DecodeOptions, VideoFrame, VideoStreamReader};
    pub use crate::encode::{EncodeOptions, VideoCodec, VideoStreamWriter};
    pub use crate::error::{MediaError, MediaResult};
    pub use crate::ffmpeg::FfmpegContext;
    pub use crate::probe::{MediaProbe, probe_media};
    pub use crate::thumbnail::{ThumbnailCache, ThumbnailOptions, extract_thumbnail};
    pub use crate::waveform::{WaveformData, WaveformOptions, extract_waveform};
    pub use tokio_util::sync::CancellationToken;
}
