// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only
// Derived from Sources/PalmierPro/Export/ExportService.swift (GPLv3).

use crate::error::{ExportError, ExportResult};
use crate::options::VideoExportOptions;
use clawvinci_media::decode::VideoFrame;
use clawvinci_media::encode::{EncodeOptions, VideoStreamWriter};
use clawvinci_media::ffmpeg::FfmpegContext;
use clawvinci_model::timeline::Timeline;
use clawvinci_render::compositor::composite_frame;
use clawvinci_render::plan::CompositionBuilder;
use std::collections::HashMap;
use std::path::Path;
use tokio_util::sync::CancellationToken;
use tracing::info;

pub struct ExportService;

impl ExportService {
    /// Offline frame-by-frame batch rendering of a Timeline to a video file.
    /// Reuses the authoritative `CompositionBuilder` and `composite_frame` pipeline.
    pub async fn render_timeline_to_file<F>(
        timeline: &Timeline,
        options: &VideoExportOptions,
        output_path: &Path,
        sources: &HashMap<String, VideoFrame>,
        cancel_token: CancellationToken,
        progress_callback: F,
    ) -> ExportResult<usize>
    where
        F: Fn(f64) + Send + Sync + 'static,
    {
        let (render_w, render_h) = options
            .resolution
            .render_size(timeline.width as u32, timeline.height as u32);

        let fps = options.custom_fps.unwrap_or(timeline.fps as f64);
        let total_frames = timeline.total_frames().max(1) as usize;

        let mut encode_opts = EncodeOptions::new(render_w, render_h, fps);
        encode_opts.codec = options.codec.media_codec();
        encode_opts.crf = options.crf;
        encode_opts.bitrate_kbps = options.bitrate_kbps;

        let ffmpeg_ctx = FfmpegContext::discover().await.map_err(ExportError::MediaError)?;
        let mut writer = VideoStreamWriter::open(&ffmpeg_ctx, output_path, &encode_opts)
            .map_err(ExportError::MediaError)?;

        // Prepare a scaled timeline so CompositionBuilder targets the requested render resolution
        let mut render_timeline = timeline.clone();
        render_timeline.width = render_w as i32;
        render_timeline.height = render_h as i32;

        info!(
            "Starting export render: {}x{} @ {:.2}fps ({} total frames) to {:?}",
            render_w, render_h, fps, total_frames, output_path
        );

        for frame_idx in 0..total_frames {
            if cancel_token.is_cancelled() {
                drop(writer);
                let _ = tokio::fs::remove_file(output_path).await;
                return Err(ExportError::Cancelled);
            }

            // Build FramePlan for current frame index
            let plan = CompositionBuilder::build_frame_plan(
                &render_timeline,
                frame_idx,
            )?;

            // Authoritative composite_frame shared with preview engine
            let composited = composite_frame(&plan, sources)?;

            // Write raw RGBA frame to FFmpeg encoder stdin
            if let Err(e) = writer.write_frame(&composited.data).await {
                drop(writer);
                let _ = tokio::fs::remove_file(output_path).await;
                return Err(ExportError::MediaError(e));
            }

            let progress = (frame_idx + 1) as f64 / total_frames as f64;
            progress_callback(progress);
        }

        let frames_written = writer.finish().await.map_err(ExportError::MediaError)?;
        info!(
            "Export render completed successfully: {} frames written to {:?}",
            frames_written, output_path
        );

        Ok(frames_written)
    }
}
