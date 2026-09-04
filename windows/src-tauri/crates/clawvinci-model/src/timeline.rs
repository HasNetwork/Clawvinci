// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only
// Derived from Sources/PalmierPro/Models/Timeline.swift (GPLv3).

use crate::blend_mode::BlendMode;
use crate::clip_type::ClipType;
use crate::effect::Effect;
use crate::keyframe::{
    smoothstep, AnimPair, Interpolation, Keyframe, KeyframeInterpolatable, KeyframeTrack,
};
use crate::text_animation::{TextAnimation, WordTiming};
use crate::text_fill_mode::TextFillMode;
use crate::text_style::TextStyle;
use crate::timeline_marker::TimelineMarker;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

fn default_uuid() -> String {
    Uuid::new_v4().to_string()
}
fn default_true() -> bool {
    true
}
fn default_one() -> f64 {
    1.0
}
fn default_fps() -> i32 {
    30
}
fn default_width() -> i32 {
    1920
}
fn default_height() -> i32 {
    1080
}
fn default_timeline_name() -> String {
    "Timeline 1".to_string()
}
fn default_display_height() -> f64 {
    48.0
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ClipLocation {
    pub track_index: usize,
    pub clip_index: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimelineViewState {
    #[serde(default)]
    pub playhead_frame: i64,
    #[serde(default = "default_one")]
    pub zoom_scale: f64,
    #[serde(default)]
    pub scroll_offset_x: f64,
}

impl Default for TimelineViewState {
    fn default() -> Self {
        Self {
            playhead_frame: 0,
            zoom_scale: 1.0,
            scroll_offset_x: 0.0,
        }
    }
}

/// Per-clip crop as edge insets in normalized (0..=1) source coordinates.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
pub struct Crop {
    #[serde(default)]
    pub left: f64,
    #[serde(default)]
    pub top: f64,
    #[serde(default)]
    pub right: f64,
    #[serde(default)]
    pub bottom: f64,
}

impl Crop {
    pub const MINIMUM_VISIBLE_FRACTION: f64 = 0.05;

    pub fn new(left: f64, top: f64, right: f64, bottom: f64) -> Self {
        Self {
            left,
            top,
            right,
            bottom,
        }
    }

    pub fn is_identity(&self) -> bool {
        self.left == 0.0 && self.top == 0.0 && self.right == 0.0 && self.bottom == 0.0
    }

    pub fn visible_width_fraction(&self) -> f64 {
        (1.0 - self.left - self.right).max(0.0)
    }

    pub fn visible_height_fraction(&self) -> f64 {
        (1.0 - self.top - self.bottom).max(0.0)
    }
}

impl KeyframeInterpolatable for Crop {
    fn keyframe_interpolate(a: &Self, b: &Self, t: f64) -> Self {
        Self {
            left: f64::keyframe_interpolate(&a.left, &b.left, t),
            top: f64::keyframe_interpolate(&a.top, &b.top, t),
            right: f64::keyframe_interpolate(&a.right, &b.right, t),
            bottom: f64::keyframe_interpolate(&a.bottom, &b.bottom, t),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CropAspectRatio {
    pub horizontal: f64,
    pub vertical: f64,
}

impl CropAspectRatio {
    pub fn new(horizontal: f64, vertical: f64) -> Option<Self> {
        if horizontal.is_finite()
            && vertical.is_finite()
            && horizontal > 0.0
            && vertical > 0.0
            && (horizontal / vertical).is_finite()
        {
            Some(Self {
                horizontal,
                vertical,
            })
        } else {
            None
        }
    }

    pub fn pixel_aspect(&self) -> f64 {
        self.horizontal / self.vertical
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transform {
    pub center_x: f64,
    pub center_y: f64,
    pub width: f64,
    pub height: f64,
    pub rotation: f64,
    pub rotation_x: f64,
    pub rotation_y: f64,
    pub flip_horizontal: bool,
    pub flip_vertical: bool,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            center_x: 0.5,
            center_y: 0.5,
            width: 1.0,
            height: 1.0,
            rotation: 0.0,
            rotation_x: 0.0,
            rotation_y: 0.0,
            flip_horizontal: false,
            flip_vertical: false,
        }
    }
}

impl Serialize for Transform {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        #[derive(Serialize)]
        #[serde(rename_all = "camelCase")]
        struct Repr {
            center_x: f64,
            center_y: f64,
            width: f64,
            height: f64,
            rotation: f64,
            rotation_x: f64,
            rotation_y: f64,
            flip_horizontal: bool,
            flip_vertical: bool,
        }
        Repr {
            center_x: self.center_x,
            center_y: self.center_y,
            width: self.width,
            height: self.height,
            rotation: self.rotation,
            rotation_x: self.rotation_x,
            rotation_y: self.rotation_y,
            flip_horizontal: self.flip_horizontal,
            flip_vertical: self.flip_vertical,
        }
        .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Transform {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct Repr {
            #[serde(default)]
            center_x: Option<f64>,
            #[serde(default)]
            center_y: Option<f64>,
            #[serde(default = "default_one")]
            width: f64,
            #[serde(default = "default_one")]
            height: f64,
            #[serde(default)]
            rotation: f64,
            #[serde(default)]
            rotation_x: f64,
            #[serde(default)]
            rotation_y: f64,
            #[serde(default)]
            flip_horizontal: bool,
            #[serde(default)]
            flip_vertical: bool,
            // Legacy keys
            #[serde(default)]
            x: Option<f64>,
            #[serde(default)]
            y: Option<f64>,
        }

        let r = Repr::deserialize(deserializer)?;
        let center_x = if let Some(cx) = r.center_x {
            cx
        } else if let Some(old_x) = r.x {
            old_x + r.width - 0.5
        } else {
            0.5
        };

        let center_y = if let Some(cy) = r.center_y {
            cy
        } else if let Some(old_y) = r.y {
            old_y + r.height - 0.5
        } else {
            0.5
        };

        Ok(Self {
            center_x,
            center_y,
            width: r.width,
            height: r.height,
            rotation: r.rotation,
            rotation_x: r.rotation_x,
            rotation_y: r.rotation_y,
            flip_horizontal: r.flip_horizontal,
            flip_vertical: r.flip_vertical,
        })
    }
}

impl Transform {
    pub fn top_left(&self) -> (f64, f64) {
        (self.center_x - self.width / 2.0, self.center_y - self.height / 2.0)
    }

    pub fn center(&self) -> (f64, f64) {
        (self.center_x, self.center_y)
    }

    pub fn snap_to_boundary(value: f64, threshold: f64) -> f64 {
        if value.abs() < threshold {
            return 0.0;
        }
        if (value - 1.0).abs() < threshold {
            return 1.0;
        }
        value
    }

    pub fn snap_to_canvas_edges(&mut self, threshold: f64) {
        let (tl_x, tl_y) = self.top_left();
        let snapped_left = Self::snap_to_boundary(tl_x, threshold);
        let snapped_right = Self::snap_to_boundary(tl_x + self.width, threshold);
        if snapped_left != tl_x {
            self.center_x -= tl_x - snapped_left;
        } else if snapped_right != tl_x + self.width {
            self.center_x -= tl_x + self.width - snapped_right;
        }

        let (tl2_x, tl2_y) = self.top_left();
        let _ = tl2_x;
        let snapped_top = Self::snap_to_boundary(tl2_y, threshold);
        let snapped_bottom = Self::snap_to_boundary(tl2_y + self.height, threshold);
        if snapped_top != tl2_y {
            self.center_y -= tl2_y - snapped_top;
        } else if snapped_bottom != tl2_y + self.height {
            self.center_y -= tl2_y + self.height - snapped_bottom;
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Clip {
    #[serde(default = "default_uuid")]
    pub id: String,
    pub media_ref: String,
    #[serde(default)]
    pub media_type: ClipType,
    #[serde(default)]
    pub source_clip_type: ClipType,
    pub start_frame: i64,
    pub duration_frames: i64,
    #[serde(default)]
    pub trim_start_frame: i64,
    #[serde(default)]
    pub trim_end_frame: i64,
    #[serde(default = "default_one")]
    pub speed: f64,
    #[serde(default = "default_one")]
    pub volume: f64,
    #[serde(default)]
    pub fade_in_frames: i64,
    #[serde(default)]
    pub fade_out_frames: i64,
    #[serde(default)]
    pub fade_in_interpolation: Interpolation,
    #[serde(default)]
    pub fade_out_interpolation: Interpolation,
    #[serde(default = "default_one")]
    pub opacity: f64,
    #[serde(default)]
    pub transform: Transform,
    #[serde(default)]
    pub crop: Crop,
    #[serde(default)]
    pub edge_rounding: f64,
    #[serde(default)]
    pub edge_softness: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub link_group_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub caption_group_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub multicam_group_id: Option<String>,

    // Text clips only
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text_content: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text_style: Option<TextStyle>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text_animation: Option<TextAnimation>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub word_timings: Option<Vec<WordTiming>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text_fill_mode: Option<TextFillMode>,

    // Keyframe tracks
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub opacity_track: Option<KeyframeTrack<f64>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub position_track: Option<KeyframeTrack<AnimPair>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scale_track: Option<KeyframeTrack<AnimPair>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rotation_track: Option<KeyframeTrack<f64>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub crop_track: Option<KeyframeTrack<Crop>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub volume_track: Option<KeyframeTrack<f64>>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub effects: Option<Vec<Effect>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub blend_mode: Option<BlendMode>,
}

impl Clip {
    pub fn new(media_ref: impl Into<String>, start_frame: i64, duration_frames: i64) -> Self {
        Self {
            id: default_uuid(),
            media_ref: media_ref.into(),
            media_type: ClipType::Video,
            source_clip_type: ClipType::Video,
            start_frame,
            duration_frames,
            trim_start_frame: 0,
            trim_end_frame: 0,
            speed: 1.0,
            volume: 1.0,
            fade_in_frames: 0,
            fade_out_frames: 0,
            fade_in_interpolation: Interpolation::Linear,
            fade_out_interpolation: Interpolation::Linear,
            opacity: 1.0,
            transform: Transform::default(),
            crop: Crop::default(),
            edge_rounding: 0.0,
            edge_softness: 0.0,
            link_group_id: None,
            caption_group_id: None,
            multicam_group_id: None,
            text_content: None,
            text_style: None,
            text_animation: None,
            word_timings: None,
            text_fill_mode: None,
            opacity_track: None,
            position_track: None,
            scale_track: None,
            rotation_track: None,
            crop_track: None,
            volume_track: None,
            effects: None,
            blend_mode: None,
        }
    }

    pub fn end_frame(&self) -> i64 {
        self.start_frame + self.duration_frames
    }

    pub fn source_frames_consumed(&self) -> i64 {
        ((self.duration_frames as f64 * self.speed).round()) as i64
    }

    pub fn source_duration_frames(&self) -> i64 {
        self.source_frames_consumed() + self.trim_start_frame + self.trim_end_frame
    }

    fn keyframe_offset(&self, frame: i64) -> i64 {
        frame - self.start_frame
    }

    pub fn opacity_at(&self, frame: i64) -> f64 {
        let base = self.raw_opacity_at(frame);
        if self.media_type != ClipType::Audio
            && (self.fade_in_frames > 0 || self.fade_out_frames > 0)
        {
            base * self.fade_multiplier(frame)
        } else {
            base
        }
    }

    pub fn raw_opacity_at(&self, frame: i64) -> f64 {
        self.opacity_track
            .as_ref()
            .map(|t| t.sample(self.keyframe_offset(frame), self.opacity))
            .unwrap_or(self.opacity)
    }

    pub fn rotation_at(&self, frame: i64) -> f64 {
        self.rotation_track
            .as_ref()
            .map(|t| t.sample(self.keyframe_offset(frame), self.transform.rotation))
            .unwrap_or(self.transform.rotation)
    }

    pub fn top_left_at(&self, frame: i64) -> (f64, f64) {
        if let Some(track) = &self.position_track {
            if track.is_active() {
                let p = track.sample(self.keyframe_offset(frame), AnimPair::new(0.0, 0.0));
                return (p.a, p.b);
            }
        }
        let (cx, cy) = self.transform.center();
        let (w, h) = self.size_at(frame);
        (cx - w / 2.0, cy - h / 2.0)
    }

    pub fn size_at(&self, frame: i64) -> (f64, f64) {
        let fallback = AnimPair::new(self.transform.width, self.transform.height);
        let s = self
            .scale_track
            .as_ref()
            .map(|t| t.sample(self.keyframe_offset(frame), fallback))
            .unwrap_or(fallback);
        (s.a, s.b)
    }

    pub fn transform_at(&self, frame: i64) -> Transform {
        let (tl_x, tl_y) = self.top_left_at(frame);
        let (w, h) = self.size_at(frame);
        let mut t = self.transform;
        t.center_x = tl_x + w / 2.0;
        t.center_y = tl_y + h / 2.0;
        t.width = w;
        t.height = h;
        t.rotation = self.rotation_at(frame);
        t
    }

    pub fn crop_at(&self, frame: i64) -> Crop {
        self.crop_track
            .as_ref()
            .map(|t| t.sample(self.keyframe_offset(frame), self.crop))
            .unwrap_or(self.crop)
    }

    pub fn fade_multiplier(&self, frame: i64) -> f64 {
        let rel = frame - self.start_frame;
        if rel < 0 || rel > self.duration_frames {
            return 0.0;
        }
        let in_mul = if self.fade_in_frames > 0 {
            let t = (rel as f64 / self.fade_in_frames as f64).min(1.0);
            if self.fade_in_interpolation == Interpolation::Smooth {
                smoothstep(t)
            } else {
                t
            }
        } else {
            1.0
        };

        let out_rem = self.duration_frames - rel;
        let out_mul = if self.fade_out_frames > 0 {
            let t = (out_rem as f64 / self.fade_out_frames as f64).min(1.0);
            if self.fade_out_interpolation == Interpolation::Smooth {
                smoothstep(t)
            } else {
                t
            }
        } else {
            1.0
        };

        in_mul.min(out_mul)
    }

    pub fn clamp_keyframes_to_duration(&mut self) {
        fn clamp_track<V: KeyframeInterpolatable + PartialEq>(
            track: Option<KeyframeTrack<V>>,
            duration: i64,
        ) -> Option<KeyframeTrack<V>> {
            let mut track = track?;
            let mut norm = KeyframeTrack::<V>::default();
            for kf in track.keyframes.drain(..) {
                if kf.frame >= 0 && kf.frame <= duration {
                    let mut next = kf;
                    next.frame = next.frame.min((duration - 1).max(0));
                    norm.upsert(next);
                }
            }
            if norm.keyframes.is_empty() {
                None
            } else {
                Some(norm)
            }
        }

        self.opacity_track = clamp_track(self.opacity_track.take(), self.duration_frames);
        self.position_track = clamp_track(self.position_track.take(), self.duration_frames);
        self.scale_track = clamp_track(self.scale_track.take(), self.duration_frames);
        self.rotation_track = clamp_track(self.rotation_track.take(), self.duration_frames);
        self.crop_track = clamp_track(self.crop_track.take(), self.duration_frames);
        self.volume_track = clamp_track(self.volume_track.take(), self.duration_frames);
    }

    pub fn clamp_fades_to_duration(&mut self) {
        self.fade_in_frames = self.fade_in_frames.clamp(0, self.duration_frames);
        self.fade_out_frames = self
            .fade_out_frames
            .clamp(0, self.duration_frames - self.fade_in_frames);
    }

    pub fn set_duration(&mut self, new_duration: i64) {
        self.duration_frames = new_duration;
        self.clamp_keyframes_to_duration();
        self.clamp_fades_to_duration();
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Track {
    #[serde(default = "default_uuid")]
    pub id: String,
    #[serde(rename = "type")]
    pub track_type: ClipType,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default)]
    pub muted: bool,
    #[serde(default)]
    pub hidden: bool,
    #[serde(default = "default_true")]
    pub sync_locked: bool,
    #[serde(default)]
    pub clips: Vec<Clip>,
    #[serde(default = "default_display_height")]
    pub display_height: f64,
}

impl Track {
    pub const MIN_HEIGHT: f64 = 24.0;
    pub const MAX_HEIGHT: f64 = 192.0;

    pub fn new(track_type: ClipType) -> Self {
        Self {
            id: default_uuid(),
            track_type,
            name: None,
            muted: false,
            hidden: false,
            sync_locked: true,
            clips: Vec::new(),
            display_height: default_display_height(),
        }
    }

    pub fn end_frame(&self) -> i64 {
        self.clips.iter().map(|c| c.end_frame()).max().unwrap_or(0)
    }

    pub fn contiguous_clip_ids(&self, from_end: i64, exclude_id: &str) -> HashSet<String> {
        let mut ids = HashSet::new();
        let mut chain_end = from_end;
        let mut sorted = self.clips.clone();
        sorted.sort_by_key(|c| c.start_frame);
        for c in sorted {
            if c.id != exclude_id && c.start_frame >= from_end {
                if c.start_frame != chain_end {
                    break;
                }
                chain_end = c.end_frame();
                ids.insert(c.id);
            }
        }
        ids
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Timeline {
    #[serde(default = "default_uuid")]
    pub id: String,
    #[serde(default = "default_timeline_name")]
    pub name: String,
    #[serde(default = "default_fps")]
    pub fps: i32,
    #[serde(default = "default_width")]
    pub width: i32,
    #[serde(default = "default_height")]
    pub height: i32,
    #[serde(default)]
    pub settings_configured: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub folder_id: Option<String>,
    #[serde(default)]
    pub tracks: Vec<Track>,
    #[serde(default)]
    pub markers: Vec<TimelineMarker>,
}

impl Default for Timeline {
    fn default() -> Self {
        Self {
            id: default_uuid(),
            name: default_timeline_name(),
            fps: 30,
            width: 1920,
            height: 1080,
            settings_configured: false,
            folder_id: None,
            tracks: Vec::new(),
            markers: Vec::new(),
        }
    }
}

impl Timeline {
    pub fn new(fps: i32, width: i32, height: i32) -> Self {
        Self {
            fps,
            width,
            height,
            ..Default::default()
        }
    }

    pub fn total_frames(&self) -> i64 {
        self.tracks.iter().map(|t| t.end_frame()).max().unwrap_or(0)
    }

    pub fn display_frames(&self) -> i64 {
        let max_marker = self
            .markers
            .iter()
            .map(|m| m.start_frame + m.duration_frames.max(1))
            .max()
            .unwrap_or(0);
        self.total_frames().max(max_marker)
    }

    pub fn has_audio_clips(&self) -> bool {
        self.tracks
            .iter()
            .any(|t| t.track_type == ClipType::Audio && !t.clips.is_empty())
    }
}
