// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only
// Derived from Sources/PalmierPro/Models/Keyframe.swift (GPLv3).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Interpolation {
    Linear,
    Hold,
    #[default]
    Smooth,
}

#[inline]
pub fn smoothstep(t: f64) -> f64 {
    t * t * (3.0 - 2.0 * t)
}

pub trait KeyframeInterpolatable: Clone {
    fn keyframe_interpolate(a: &Self, b: &Self, t: f64) -> Self;
}

impl KeyframeInterpolatable for f64 {
    fn keyframe_interpolate(a: &Self, b: &Self, t: f64) -> Self {
        a + (b - a) * t
    }
}

/// Two-component keyframe value used for position (x, y) and scale (width, height).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
pub struct AnimPair {
    pub a: f64,
    pub b: f64,
}

impl AnimPair {
    pub fn new(a: f64, b: f64) -> Self {
        Self { a, b }
    }
}

impl KeyframeInterpolatable for AnimPair {
    fn keyframe_interpolate(a: &Self, b: &Self, t: f64) -> Self {
        Self {
            a: f64::keyframe_interpolate(&a.a, &b.a, t),
            b: f64::keyframe_interpolate(&a.b, &b.b, t),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Keyframe<V> {
    pub frame: i64,
    pub value: V,
    #[serde(default)]
    pub interpolation_out: Interpolation,
}

impl<V> Keyframe<V> {
    pub fn new(frame: i64, value: V, interpolation_out: Interpolation) -> Self {
        Self {
            frame,
            value,
            interpolation_out,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct KeyframeTrack<V> {
    #[serde(default)]
    pub keyframes: Vec<Keyframe<V>>,
}

impl<V: Clone + PartialEq> KeyframeTrack<V> {
    pub fn new(keyframes: Vec<Keyframe<V>>) -> Self {
        Self { keyframes }
    }

    pub fn is_active(&self) -> bool {
        !self.keyframes.is_empty()
    }

    pub fn upsert(&mut self, kf: Keyframe<V>) {
        if let Some(pos) = self.keyframes.iter().position(|k| k.frame == kf.frame) {
            self.keyframes[pos] = kf;
        } else {
            let at = self
                .keyframes
                .iter()
                .position(|k| k.frame > kf.frame)
                .unwrap_or(self.keyframes.len());
            self.keyframes.insert(at, kf);
        }
    }

    pub fn remove(&mut self, at_frame: i64) {
        self.keyframes.retain(|k| k.frame != at_frame);
    }

    pub fn move_keyframe(&mut self, from_frame: i64, to_frame: i64) {
        if let Some(pos) = self.keyframes.iter().position(|k| k.frame == from_frame) {
            if to_frame != from_frame && self.keyframes.iter().any(|k| k.frame == to_frame) {
                return;
            }
            let mut kf = self.keyframes.remove(pos);
            kf.frame = to_frame;
            self.upsert(kf);
        }
    }

    pub fn frames_in_range(&self, lower: i64, upper: i64) -> Vec<i64> {
        self.keyframes
            .iter()
            .filter(|k| k.frame >= lower && k.frame <= upper)
            .map(|k| k.frame)
            .collect()
    }
}

impl<V: KeyframeInterpolatable + PartialEq> KeyframeTrack<V> {
    pub fn sample(&self, frame: i64, fallback: V) -> V {
        if self.keyframes.is_empty() {
            return fallback;
        }
        if self.keyframes.len() == 1 || frame <= self.keyframes[0].frame {
            return self.keyframes[0].value.clone();
        }
        let last = self.keyframes.last().unwrap();
        if frame >= last.frame {
            return last.value.clone();
        }

        let b_idx = match self.keyframes.iter().position(|k| k.frame > frame) {
            Some(idx) => idx,
            None => return last.value.clone(),
        };

        let a = &self.keyframes[b_idx - 1];
        let b = &self.keyframes[b_idx];

        let span = (b.frame - a.frame) as f64;
        let raw = if span == 0.0 {
            0.0
        } else {
            (frame - a.frame) as f64 / span
        };

        match a.interpolation_out {
            Interpolation::Hold => a.value.clone(),
            Interpolation::Linear => V::keyframe_interpolate(&a.value, &b.value, raw),
            Interpolation::Smooth => V::keyframe_interpolate(&a.value, &b.value, smoothstep(raw)),
        }
    }

    pub fn rebased(&self, offset: i64, fallback: V) -> Option<Self> {
        if !self.is_active() {
            return None;
        }
        let boundary = self.sample(offset, fallback);
        let mut kfs: Vec<Keyframe<V>> = self
            .keyframes
            .iter()
            .filter(|k| k.frame >= offset)
            .map(|k| Keyframe::new(k.frame - offset, k.value.clone(), k.interpolation_out))
            .collect();

        if kfs.first().map(|k| k.frame) != Some(0) {
            let interp = self
                .keyframes
                .iter()
                .filter(|k| k.frame < offset)
                .last()
                .map(|k| k.interpolation_out)
                .unwrap_or(Interpolation::Smooth);
            kfs.insert(0, Keyframe::new(0, boundary, interp));
        }

        if kfs.is_empty() {
            None
        } else {
            Some(Self::new(kfs))
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AnimatableProperty {
    Opacity,
    Position,
    Scale,
    Rotation,
    Crop,
    Blur,
    Volume,
}
