// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only
// Derived from Sources/PalmierPro/Preview/CompositionBuilder.swift (GPLv3).

use crate::error::{RenderError, RenderResult};
use clawvinci_model::blend_mode::BlendMode;
use clawvinci_model::clip_type::ClipType;
use clawvinci_model::text_style::TextStyle;
use clawvinci_model::timeline::{Clip, Crop, Timeline, Transform};
use serde::{Deserialize, Serialize};

/// Source description for a visual layer in the composition.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
#[allow(clippy::large_enum_variant)]
pub enum LayerSource {
    Video {
        media_ref: String,
        source_frame: usize,
        source_time_seconds: f64,
    },
    Image {
        media_ref: String,
    },
    Text {
        content: String,
        style: Option<TextStyle>,
    },
    SolidColor {
        r: u8,
        g: u8,
        b: u8,
        a: u8,
    },
    Nested {
        timeline_id: String,
        sub_plan: Box<FramePlan>,
    },
}

/// A single visual layer to be composited onto the frame canvas.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LayerPlan {
    pub clip_id: String,
    pub track_id: String,
    pub track_index: usize,
    pub source: LayerSource,
    pub transform: Transform,
    pub crop: Crop,
    pub opacity: f32,
    pub blend_mode: BlendMode,
    pub natural_width: u32,
    pub natural_height: u32,
}

/// An active audio source contributing to the frame's audio mix.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioClipPlan {
    pub clip_id: String,
    pub track_id: String,
    pub media_ref: String,
    pub source_frame: usize,
    pub source_time_seconds: f64,
    pub volume: f32,
    pub speed: f64,
}

/// Mixed audio instructions for a single frame.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct AudioPlan {
    pub active_clips: Vec<AudioClipPlan>,
    pub master_volume: f32,
}

/// Complete render instructions for a single output frame.
/// This is the shared representation consumed by both live preview and offline export.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FramePlan {
    pub frame_index: usize,
    pub time_seconds: f64,
    pub render_width: u32,
    pub render_height: u32,
    pub fps: i32,
    pub layers: Vec<LayerPlan>,
    pub audio_mix: Option<AudioPlan>,
}

impl FramePlan {
    pub fn is_empty(&self) -> bool {
        self.layers.is_empty()
    }
}

/// Render-graph builder translating a Timeline into per-frame FramePlans.
pub struct CompositionBuilder;

impl CompositionBuilder {
    /// Builds the `FramePlan` for a specific frame index on the timeline.
    pub fn build_frame_plan(
        timeline: &Timeline,
        frame: usize,
    ) -> RenderResult<FramePlan> {
        Self::build_frame_plan_dyn(timeline, frame, &|_| None, &|_| None)
    }

    /// Builds the `FramePlan` using custom source size and nested timeline resolvers.
    pub fn build_frame_plan_with_resolvers<FSize, FNest>(
        timeline: &Timeline,
        frame: usize,
        resolve_source_size: FSize,
        resolve_nested_timeline: FNest,
    ) -> RenderResult<FramePlan>
    where
        FSize: Fn(&str) -> Option<(u32, u32)>,
        FNest: Fn(&str) -> Option<&Timeline>,
    {
        Self::build_frame_plan_dyn(timeline, frame, &resolve_source_size, &resolve_nested_timeline)
    }

    /// Internal implementation using trait objects to avoid infinite monomorphization on recursion.
    fn build_frame_plan_dyn(
        timeline: &Timeline,
        frame: usize,
        resolve_source_size: &dyn Fn(&str) -> Option<(u32, u32)>,
        resolve_nested_timeline: &dyn Fn(&str) -> Option<&Timeline>,
    ) -> RenderResult<FramePlan> {
        if timeline.width <= 0 || timeline.height <= 0 || timeline.fps <= 0 {
            return Err(RenderError::InvalidTimeline(format!(
                "Invalid dimensions or fps: {}x{} @ {}fps",
                timeline.width, timeline.height, timeline.fps
            )));
        }

        let frame_i64 = frame as i64;
        let time_seconds = frame as f64 / timeline.fps as f64;
        let render_w = timeline.width as u32;
        let render_h = timeline.height as u32;

        let mut layers = Vec::new();
        let mut audio_clips = Vec::new();

        for (track_idx, track) in timeline.tracks.iter().enumerate() {
            if track.track_type == ClipType::Audio {
                if track.muted {
                    continue;
                }
                for clip in &track.clips {
                    if clip.start_frame <= frame_i64 && frame_i64 < clip.end_frame() {
                        let audio_plan = Self::build_audio_clip_plan(clip, track_idx, track, frame_i64, timeline.fps);
                        audio_clips.push(audio_plan);
                    }
                }
            } else {
                if track.hidden {
                    continue;
                }
                for clip in &track.clips {
                    if clip.start_frame <= frame_i64 && frame_i64 < clip.end_frame() {
                        let layer = Self::build_layer_plan_dyn(
                            clip,
                            track_idx,
                            track,
                            frame_i64,
                            timeline.fps,
                            render_w,
                            render_h,
                            resolve_source_size,
                            resolve_nested_timeline,
                        )?;
                        layers.push(layer);
                    }
                }
            }
        }

        let audio_mix = if audio_clips.is_empty() {
            None
        } else {
            Some(AudioPlan {
                active_clips: audio_clips,
                master_volume: 1.0,
            })
        };

        Ok(FramePlan {
            frame_index: frame,
            time_seconds,
            render_width: render_w,
            render_height: render_h,
            fps: timeline.fps,
            layers,
            audio_mix,
        })
    }

    /// Builds a slice of FramePlans for a contiguous range of frames [start_frame, end_frame).
    pub fn build_range(
        timeline: &Timeline,
        start_frame: usize,
        end_frame: usize,
    ) -> RenderResult<Vec<FramePlan>> {
        let mut plans = Vec::with_capacity(end_frame.saturating_sub(start_frame));
        for f in start_frame..end_frame {
            plans.push(Self::build_frame_plan(timeline, f)?);
        }
        Ok(plans)
    }

    #[allow(clippy::too_many_arguments)]
    fn build_layer_plan_dyn(
        clip: &Clip,
        track_idx: usize,
        track: &clawvinci_model::timeline::Track,
        frame: i64,
        fps: i32,
        render_w: u32,
        render_h: u32,
        resolve_source_size: &dyn Fn(&str) -> Option<(u32, u32)>,
        resolve_nested_timeline: &dyn Fn(&str) -> Option<&Timeline>,
    ) -> RenderResult<LayerPlan> {
        let rel_frame = frame - clip.start_frame;
        let speed = if clip.speed == 0.0 { 1.0 } else { clip.speed };
        let source_in = clip.trim_start_frame;
        let source_frame = (source_in as f64 + (rel_frame as f64 * speed)).max(0.0) as usize;
        let source_time_seconds = source_frame as f64 / fps.max(1) as f64;

        let base_opacity = clip.opacity_at(frame);
        let fade_mul = clip.fade_multiplier(frame);
        let opacity = (base_opacity * fade_mul).clamp(0.0, 1.0) as f32;

        let transform = clip.transform_at(frame);
        let crop = clip.crop_at(frame);
        let blend_mode = clip.blend_mode.unwrap_or(BlendMode::Normal);

        let natural_size = resolve_source_size(&clip.media_ref).unwrap_or((render_w, render_h));

        let source = match clip.media_type {
            ClipType::Text => LayerSource::Text {
                content: clip.text_content.clone().unwrap_or_default(),
                style: clip.text_style.clone(),
            },
            ClipType::Image => LayerSource::Image {
                media_ref: clip.media_ref.clone(),
            },
            ClipType::Sequence => {
                if let Some(nested) = resolve_nested_timeline(&clip.media_ref) {
                    let sub_plan = Self::build_frame_plan_dyn(
                        nested,
                        source_frame,
                        resolve_source_size,
                        resolve_nested_timeline,
                    )?;
                    LayerSource::Nested {
                        timeline_id: clip.media_ref.clone(),
                        sub_plan: Box::new(sub_plan),
                    }
                } else {
                    LayerSource::Video {
                        media_ref: clip.media_ref.clone(),
                        source_frame,
                        source_time_seconds,
                    }
                }
            }
            _ => LayerSource::Video {
                media_ref: clip.media_ref.clone(),
                source_frame,
                source_time_seconds,
            },
        };

        Ok(LayerPlan {
            clip_id: clip.id.clone(),
            track_id: track.id.clone(),
            track_index: track_idx,
            source,
            transform,
            crop,
            opacity,
            blend_mode,
            natural_width: natural_size.0,
            natural_height: natural_size.1,
        })
    }

    fn build_audio_clip_plan(
        clip: &Clip,
        _track_idx: usize,
        track: &clawvinci_model::timeline::Track,
        frame: i64,
        fps: i32,
    ) -> AudioClipPlan {
        let rel_frame = frame - clip.start_frame;
        let speed = if clip.speed == 0.0 { 1.0 } else { clip.speed };
        let source_in = clip.trim_start_frame;
        let source_frame = (source_in as f64 + (rel_frame as f64 * speed)).max(0.0) as usize;
        let source_time_seconds = source_frame as f64 / fps.max(1) as f64;

        let fade_mul = clip.fade_multiplier(frame);
        let volume = (clip.volume * fade_mul).clamp(0.0, 4.0) as f32;

        AudioClipPlan {
            clip_id: clip.id.clone(),
            track_id: track.id.clone(),
            media_ref: clip.media_ref.clone(),
            source_frame,
            source_time_seconds,
            volume,
            speed,
        }
    }
}

