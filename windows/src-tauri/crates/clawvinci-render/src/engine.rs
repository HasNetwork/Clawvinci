// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only
// Derived from Sources/PalmierPro/Preview/VideoEngine.swift (GPLv3).

use crate::audio::{AudioClock, ScrubAudioEngine};
use crate::compositor::composite_frame;
use crate::error::RenderResult;
use crate::plan::{CompositionBuilder, FramePlan};
use clawvinci_media::decode::VideoFrame;
use clawvinci_model::timeline::Timeline;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// Current playback lifecycle status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum PlaybackStatus {
    #[default]
    Stopped,
    Playing,
    Paused,
    Seeking,
    Scrubbing,
}

/// Mode governing how playhead positioning is resolved.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum SeekMode {
    #[default]
    Exact,
    InteractiveScrub,
    StepForward,
    StepBackward,
}

/// Serializable snapshot of current playback engine state for UI consumption.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaybackStateSnapshot {
    pub status: PlaybackStatus,
    pub current_frame: usize,
    pub time_seconds: f64,
    pub fps: i32,
    pub total_frames: usize,
    pub playback_rate: f64,
    pub generation: u64,
}

/// Real-time playback engine driving timeline composition, clock sync, and frame delivery.
pub struct PlaybackEngine {
    timeline: Timeline,
    status: PlaybackStatus,
    current_frame: usize,
    playback_rate: f64,
    rebuild_generation: Arc<AtomicU64>,
    clock: AudioClock,
    cached_sources: HashMap<String, VideoFrame>,
    last_committed_frame: Option<VideoFrame>,
}

impl PlaybackEngine {
    pub fn new(timeline: Timeline) -> Self {
        Self {
            timeline,
            status: PlaybackStatus::Stopped,
            current_frame: 0,
            playback_rate: 1.0,
            rebuild_generation: Arc::new(AtomicU64::new(1)),
            clock: AudioClock::new(),
            cached_sources: HashMap::new(),
            last_committed_frame: None,
        }
    }

    /// Sets a new timeline (e.g. after an edit or project load).
    /// Bumps the monotonic rebuild generation counter to immediately invalidate in-flight renders.
    pub fn set_timeline(&mut self, timeline: Timeline) -> u64 {
        self.timeline = timeline;
        self.bump_generation()
    }

    pub fn timeline(&self) -> &Timeline {
        &self.timeline
    }

    pub fn bump_generation(&self) -> u64 {
        self.rebuild_generation.fetch_add(1, Ordering::SeqCst) + 1
    }

    pub fn current_generation(&self) -> u64 {
        self.rebuild_generation.load(Ordering::SeqCst)
    }

    pub fn is_stale_generation(&self, generation: u64) -> bool {
        generation < self.current_generation()
    }

    pub fn status(&self) -> PlaybackStatus {
        self.status
    }

    pub fn current_frame(&self) -> usize {
        self.current_frame
    }

    pub fn playback_rate(&self) -> f64 {
        self.playback_rate
    }

    pub fn set_playback_rate(&mut self, rate: f64) {
        self.playback_rate = rate;
    }

    /// Registers or updates a decoded source frame for a media ref or clip id.
    pub fn insert_source_frame(&mut self, key: impl Into<String>, frame: VideoFrame) {
        self.cached_sources.insert(key.into(), frame);
    }

    pub fn clear_cached_sources(&mut self) {
        self.cached_sources.clear();
    }

    /// Starts playback from current playhead position.
    pub fn play(&mut self) {
        if self.status == PlaybackStatus::Playing {
            return;
        }
        let time = self.current_frame as f64 / self.timeline.fps.max(1) as f64;
        self.clock.start(time);
        self.status = PlaybackStatus::Playing;
    }

    /// Pauses playback at current position.
    pub fn pause(&mut self) {
        if self.status != PlaybackStatus::Playing {
            return;
        }
        self.clock.pause();
        self.status = PlaybackStatus::Paused;
    }

    /// Toggles play/pause state.
    pub fn toggle_play(&mut self) {
        if self.status == PlaybackStatus::Playing {
            self.pause();
        } else {
            self.play();
        }
    }

    /// Seeks to a target frame with the specified seek mode.
    pub fn seek(&mut self, frame: usize, mode: SeekMode) -> RenderResult<FramePlan> {
        let max_frame = self.timeline.total_frames().max(0) as usize;
        let clamped = frame.min(max_frame);

        self.current_frame = clamped;
        let time = clamped as f64 / self.timeline.fps.max(1) as f64;
        self.clock.seek(time);

        self.status = match mode {
            SeekMode::InteractiveScrub => PlaybackStatus::Scrubbing,
            _ => PlaybackStatus::Paused,
        };

        CompositionBuilder::build_frame_plan(&self.timeline, clamped)
    }

    /// Performs a single frame step (delta: +1 forward, -1 backward).
    /// Returns the updated FramePlan and generated scrub audio grain.
    pub fn step(&mut self, delta: i64) -> RenderResult<(FramePlan, Vec<f32>)> {
        let current = self.current_frame as i64;
        let target = (current + delta).max(0) as usize;
        let mode = if delta >= 0 {
            SeekMode::StepForward
        } else {
            SeekMode::StepBackward
        };

        let plan = self.seek(target, mode)?;
        let audio_grain = ScrubAudioEngine::synthesize_step_grain(
            delta >= 0,
            ScrubAudioEngine::DEFAULT_SAMPLE_RATE,
        );

        Ok((plan, audio_grain))
    }

    /// Performs an interactive scrub to `frame`.
    /// Returns the FramePlan and velocity-proportional scrub audio.
    pub fn scrub(&mut self, frame: usize) -> RenderResult<(FramePlan, Vec<f32>)> {
        let delta = frame as i64 - self.current_frame as i64;
        let plan = self.seek(frame, SeekMode::InteractiveScrub)?;
        let audio_burst = ScrubAudioEngine::synthesize_scrub_burst(
            delta,
            ScrubAudioEngine::DEFAULT_SAMPLE_RATE,
        );

        Ok((plan, audio_burst))
    }

    /// Advances the playhead clock during playback.
    /// Returns `Some(new_frame)` if the frame index changed, or `None` if held.
    pub fn tick_clock(&mut self) -> Option<usize> {
        if self.status != PlaybackStatus::Playing {
            return None;
        }

        let new_frame = self.clock.current_frame(self.timeline.fps);
        let max_frame = self.timeline.total_frames().max(0) as usize;

        if new_frame >= max_frame {
            self.current_frame = max_frame;
            self.pause();
            self.status = PlaybackStatus::Stopped;
            return Some(max_frame);
        }

        if new_frame != self.current_frame {
            self.current_frame = new_frame;
            Some(new_frame)
        } else {
            None
        }
    }

    /// Renders the current frame synchronously using the shared compositor.
    pub fn render_current_frame(&self) -> RenderResult<VideoFrame> {
        let plan = CompositionBuilder::build_frame_plan(&self.timeline, self.current_frame)?;
        composite_frame(&plan, &self.cached_sources)
    }

    /// Commits an asynchronously rendered frame if its generation matches current.
    /// If the generation is stale (e.g. an edit occurred during render), drops the frame and returns false.
    pub fn commit_rendered_frame(&mut self, generation: u64, frame: VideoFrame) -> bool {
        if self.is_stale_generation(generation) {
            return false;
        }
        self.last_committed_frame = Some(frame);
        true
    }

    pub fn last_committed_frame(&self) -> Option<&VideoFrame> {
        self.last_committed_frame.as_ref()
    }

    /// Creates an immutable state snapshot for frontend consumption.
    pub fn snapshot(&self) -> PlaybackStateSnapshot {
        PlaybackStateSnapshot {
            status: self.status,
            current_frame: self.current_frame,
            time_seconds: self.current_frame as f64 / self.timeline.fps.max(1) as f64,
            fps: self.timeline.fps,
            total_frames: self.timeline.total_frames().max(0) as usize,
            playback_rate: self.playback_rate,
            generation: self.current_generation(),
        }
    }
}
