// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only
// Derived from Sources/PalmierPro/Models/GradeCurve.swift and HueCurves.swift (GPLv3).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CurvePoint {
    pub x: f64,
    pub y: f64,
}

impl CurvePoint {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
}

/// Master (Rec.709 luma) + per-channel R/G/B tone curves.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct GradeCurve {
    #[serde(default)]
    pub master: Vec<CurvePoint>,
    #[serde(default)]
    pub red: Vec<CurvePoint>,
    #[serde(default)]
    pub green: Vec<CurvePoint>,
    #[serde(default)]
    pub blue: Vec<CurvePoint>,
}

impl GradeCurve {
    pub fn identity_points() -> [CurvePoint; 2] {
        [CurvePoint::new(0.0, 0.0), CurvePoint::new(1.0, 1.0)]
    }

    pub fn is_identity(&self) -> bool {
        let id = Self::identity_points();
        let check = |pts: &[CurvePoint]| pts.is_empty() || pts == id;
        check(&self.master) && check(&self.red) && check(&self.green) && check(&self.blue)
    }

    /// Piecewise-linear interpolation, clamped flat outside the point range.
    pub fn eval(pts: &[CurvePoint], x: f64) -> f64 {
        let fallback = Self::identity_points();
        let slice = if pts.is_empty() { &fallback[..] } else { pts };
        let mut sorted: Vec<CurvePoint> = slice.to_vec();
        sorted.sort_by(|a, b| a.x.partial_cmp(&b.x).unwrap_or(std::cmp::Ordering::Equal));

        let first = sorted.first().unwrap();
        let last = sorted.last().unwrap();

        if x <= first.x {
            return first.y;
        }
        if x >= last.x {
            return last.y;
        }

        for i in 1..sorted.len() {
            if x <= sorted[i].x {
                let a = sorted[i - 1];
                let b = sorted[i];
                let span = b.x - a.x;
                let t = if span == 0.0 { 0.0 } else { (x - a.x) / span };
                return a.y + (b.y - a.y) * t;
            }
        }
        x
    }
}

/// Hue curves: maps source hue (0..1, cyclic) to hue, saturation, luminance adjustments.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct HueCurves {
    #[serde(default)]
    pub hue_vs_hue: Vec<CurvePoint>,
    #[serde(default)]
    pub hue_vs_sat: Vec<CurvePoint>,
    #[serde(default)]
    pub hue_vs_lum: Vec<CurvePoint>,
}

impl HueCurves {
    pub const NEUTRAL_Y: f64 = 0.5;
    pub const EFFECT_TYPE: &'static str = "color.hueCurves";

    pub fn default_points() -> Vec<CurvePoint> {
        (0..6)
            .map(|i| CurvePoint::new(i as f64 / 6.0, Self::NEUTRAL_Y))
            .collect()
    }

    pub fn is_neutral(pts: &[CurvePoint]) -> bool {
        pts.is_empty() || pts.iter().all(|p| (p.y - Self::NEUTRAL_Y).abs() < 1e-4)
    }

    pub fn is_identity(&self) -> bool {
        Self::is_neutral(&self.hue_vs_hue)
            && Self::is_neutral(&self.hue_vs_sat)
            && Self::is_neutral(&self.hue_vs_lum)
    }

    /// Cyclic piecewise-linear evaluation wrapping across 0/1 seam.
    pub fn eval(pts: &[CurvePoint], x: f64) -> f64 {
        let def = Self::default_points();
        let slice = if pts.is_empty() { &def } else { pts };
        let mut sorted = slice.to_vec();
        sorted.sort_by(|a, b| a.x.partial_cmp(&b.x).unwrap_or(std::cmp::Ordering::Equal));

        let first = match sorted.first() {
            Some(f) => *f,
            None => return Self::NEUTRAL_Y,
        };
        let last = match sorted.last() {
            Some(l) => *l,
            None => return Self::NEUTRAL_Y,
        };

        if x < first.x {
            return Self::lerp(CurvePoint::new(last.x - 1.0, last.y), first, x);
        }
        for i in 1..sorted.len() {
            if x <= sorted[i].x {
                return Self::lerp(sorted[i - 1], sorted[i], x);
            }
        }
        Self::lerp(last, CurvePoint::new(first.x + 1.0, first.y), x)
    }

    fn lerp(a: CurvePoint, b: CurvePoint, x: f64) -> f64 {
        let span = b.x - a.x;
        let t = if span == 0.0 { 0.0 } else { (x - a.x) / span };
        a.y + (b.y - a.y) * t
    }
}
