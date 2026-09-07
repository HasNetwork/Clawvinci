// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only
// Derived from Sources/PalmierPro/Export/ExportTimelineAnalyticsSnapshot.swift (GPLv3).

use clawvinci_model::clip_type::ClipType;
use clawvinci_model::timeline::Timeline;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportTimelineAnalyticsSnapshot {
    pub export_filename: String,
    pub format: String,
    pub resolution: String,
    pub total_duration_frames: i64,
    pub total_duration_seconds: f64,
    pub video_tracks_count: usize,
    pub audio_tracks_count: usize,
    pub total_clips_count: usize,
    pub effects_count: usize,
}

impl ExportTimelineAnalyticsSnapshot {
    pub fn from_timeline(
        timeline: &Timeline,
        export_filename: String,
        format: String,
        resolution: String,
    ) -> Self {
        let mut video_tracks = 0;
        let mut audio_tracks = 0;
        let mut total_clips = 0;
        let mut effects_count = 0;

        for track in &timeline.tracks {
            if track.track_type == ClipType::Audio {
                audio_tracks += 1;
            } else {
                video_tracks += 1;
            }
            total_clips += track.clips.len();
            for clip in &track.clips {
                if let Some(ref effs) = clip.effects {
                    effects_count += effs.len();
                }
            }
        }

        let total_frames = timeline.duration();
        let fps = timeline.fps.max(1) as f64;
        let total_duration_seconds = total_frames as f64 / fps;

        Self {
            export_filename,
            format,
            resolution,
            total_duration_frames: total_frames,
            total_duration_seconds,
            video_tracks_count: video_tracks,
            audio_tracks_count: audio_tracks,
            total_clips_count: total_clips,
            effects_count,
        }
    }
}
