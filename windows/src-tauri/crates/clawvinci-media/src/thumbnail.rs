// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only
// Derived from Sources/PalmierPro/Models/MediaResolver.swift (GPLv3).

use crate::error::{MediaError, MediaResult};
use crate::ffmpeg::FfmpegContext;
use std::collections::{HashMap, VecDeque};
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ThumbnailFormat {
    Png,
    Jpeg,
    RawRgba,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ThumbnailOptions {
    pub timestamp_seconds: f64,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub format: ThumbnailFormat,
}

impl Default for ThumbnailOptions {
    fn default() -> Self {
        Self {
            timestamp_seconds: 0.0,
            width: Some(320),
            height: Some(180),
            format: ThumbnailFormat::Png,
        }
    }
}

impl ThumbnailOptions {
    pub fn at_time(timestamp_seconds: f64) -> Self {
        Self {
            timestamp_seconds,
            ..Default::default()
        }
    }

    pub fn with_size(mut self, width: u32, height: u32) -> Self {
        self.width = Some(width);
        self.height = Some(height);
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct CacheKey {
    asset_id: String,
    timestamp_millis: u64,
    width: u32,
    height: u32,
    format: ThumbnailFormat,
}

#[derive(Clone)]
pub struct ThumbnailCache {
    max_entries: usize,
    entries: Arc<Mutex<HashMap<CacheKey, Vec<u8>>>>,
    lru_order: Arc<Mutex<VecDeque<CacheKey>>>,
}

impl ThumbnailCache {
    pub fn new(max_entries: usize) -> Self {
        Self {
            max_entries: max_entries.max(1),
            entries: Arc::new(Mutex::new(HashMap::new())),
            lru_order: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    async fn get(&self, key: &CacheKey) -> Option<Vec<u8>> {
        let entries = self.entries.lock().await;
        if let Some(val) = entries.get(key) {
            let mut lru = self.lru_order.lock().await;
            if let Some(pos) = lru.iter().position(|k| k == key) {
                lru.remove(pos);
            }
            lru.push_back(key.clone());
            Some(val.clone())
        } else {
            None
        }
    }

    async fn insert(&self, key: CacheKey, data: Vec<u8>) {
        let mut entries = self.entries.lock().await;
        let mut lru = self.lru_order.lock().await;

        if entries.contains_key(&key) {
            if let Some(pos) = lru.iter().position(|k| k == &key) {
                lru.remove(pos);
            }
        } else if entries.len() >= self.max_entries {
            if let Some(oldest) = lru.pop_front() {
                entries.remove(&oldest);
            }
        }

        lru.push_back(key.clone());
        entries.insert(key, data);
    }

    pub async fn get_thumbnail(&self, asset_id: &str, options: &ThumbnailOptions) -> Option<Vec<u8>> {
        let key = CacheKey {
            asset_id: asset_id.to_string(),
            timestamp_millis: (options.timestamp_seconds * 1000.0).round() as u64,
            width: options.width.unwrap_or(0),
            height: options.height.unwrap_or(0),
            format: options.format,
        };
        self.get(&key).await
    }

    pub async fn insert_thumbnail(&self, asset_id: &str, options: &ThumbnailOptions, data: Vec<u8>) {
        let key = CacheKey {
            asset_id: asset_id.to_string(),
            timestamp_millis: (options.timestamp_seconds * 1000.0).round() as u64,
            width: options.width.unwrap_or(0),
            height: options.height.unwrap_or(0),
            format: options.format,
        };
        self.insert(key, data).await;
    }

    pub async fn clear(&self) {
        self.entries.lock().await.clear();
        self.lru_order.lock().await.clear();
    }

    pub async fn len(&self) -> usize {
        self.entries.lock().await.len()
    }

    pub async fn is_empty(&self) -> bool {
        self.entries.lock().await.is_empty()
    }
}

pub async fn extract_thumbnail(
    ctx: &FfmpegContext,
    path: &Path,
    options: &ThumbnailOptions,
) -> MediaResult<Vec<u8>> {
    if !path.exists() {
        return Err(MediaError::InvalidInput(format!(
            "Media file not found: {}",
            path.display()
        )));
    }

    let mut cmd = ctx.command_ffmpeg();
    cmd.arg("-ss").arg(format!("{:.4}", options.timestamp_seconds.max(0.0)));
    cmd.arg("-i").arg(path);
    cmd.arg("-vframes").arg("1");

    if let (Some(w), Some(h)) = (options.width, options.height) {
        cmd.arg("-vf").arg(format!(
            "scale={}:{}:force_original_aspect_ratio=decrease",
            w, h
        ));
    }

    match options.format {
        ThumbnailFormat::Png => {
            cmd.arg("-f").arg("image2pipe").arg("-c:v").arg("png");
        }
        ThumbnailFormat::Jpeg => {
            cmd.arg("-f").arg("image2pipe").arg("-c:v").arg("mjpeg");
        }
        ThumbnailFormat::RawRgba => {
            cmd.arg("-f").arg("rawvideo").arg("-pix_fmt").arg("rgba");
        }
    }

    cmd.arg("pipe:1");

    let output = cmd.output().await.map_err(|e| {
        MediaError::DecodeFailed(format!("Failed to execute thumbnail extraction: {}", e))
    })?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        return Err(MediaError::DecodeFailed(format!(
            "FFmpeg thumbnail extraction failed: {}",
            err
        )));
    }

    if output.stdout.is_empty() {
        return Err(MediaError::DecodeFailed(
            "FFmpeg produced 0 thumbnail bytes".into(),
        ));
    }

    Ok(output.stdout)
}

pub async fn extract_thumbnail_cached(
    ctx: &FfmpegContext,
    cache: &ThumbnailCache,
    asset_id: &str,
    path: &Path,
    options: &ThumbnailOptions,
) -> MediaResult<Vec<u8>> {
    if let Some(cached) = cache.get_thumbnail(asset_id, options).await {
        return Ok(cached);
    }

    let data = extract_thumbnail(ctx, path, options).await?;
    cache.insert_thumbnail(asset_id, options, data.clone()).await;
    Ok(data)
}
