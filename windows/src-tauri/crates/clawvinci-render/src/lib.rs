// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only
//! Clawvinci Render & Playback Engine (Phase 4-5):
//! Render-graph builder, shared frame compositor, real-time playback synchronization.

pub mod audio;
pub mod compositor;
pub mod engine;
pub mod error;
pub mod plan;

pub use audio::{AudioClock, ScrubAudioEngine};
pub use compositor::composite_frame;
pub use engine::{PlaybackEngine, PlaybackStateSnapshot, PlaybackStatus, SeekMode};
pub use error::{RenderError, RenderResult};
pub use plan::{AudioClipPlan, AudioPlan, CompositionBuilder, FramePlan, LayerPlan, LayerSource};
