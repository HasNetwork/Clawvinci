// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only
// Derived from Metal/HueCurves.metal and Sources/PalmierPro/Compositing/Kernels/HueCurveKernel.swift (GPLv3).

use super::types::ResolvedEffectParams;
use clawvinci_model::grade::HueCurves;

const LUT_WIDTH: usize = 256;
const MAX_HUE_SHIFT: f32 = 1.0 / 12.0;
const MAX_LUM_SHIFT: f32 = 0.5;

#[inline(always)]
fn fract(x: f32) -> f32 {
    x - x.floor()
}

#[inline(always)]
fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let span = edge1 - edge0;
    if span.abs() < 1e-6 {
        return if x >= edge1 { 1.0 } else { 0.0 };
    }
    let t = ((x - edge0) / span).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

pub fn rgb2hsv(c: [f32; 3]) -> [f32; 3] {
    let r = c[0].clamp(0.0, 1.0);
    let g = c[1].clamp(0.0, 1.0);
    let b = c[2].clamp(0.0, 1.0);

    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let delta = max - min;

    let v = max;
    let s = if max > 1e-6 { delta / max } else { 0.0 };

    let mut h = 0.0f32;
    if delta > 1e-6 {
        if max == r {
            h = (g - b) / delta;
        } else if max == g {
            h = (b - r) / delta + 2.0;
        } else {
            h = (r - g) / delta + 4.0;
        }
        h = fract(h / 6.0);
    }

    [h, s, v]
}

pub fn hsv2rgb(c: [f32; 3]) -> [f32; 3] {
    let h = c[0];
    let s = c[1].clamp(0.0, 1.0);
    let v = c[2].clamp(0.0, 1.0);

    let i = (h * 6.0).floor() as i32;
    let f = h * 6.0 - i as f32;
    let p = v * (1.0 - s);
    let q = v * (1.0 - f * s);
    let t = v * (1.0 - (1.0 - f) * s);

    let (r, g, b) = match ((i % 6) + 6) % 6 {
        0 => (v, t, p),
        1 => (q, v, p),
        2 => (p, v, t),
        3 => (p, q, v),
        4 => (t, p, v),
        _ => (v, p, q),
    };

    [r.clamp(0.0, 1.0), g.clamp(0.0, 1.0), b.clamp(0.0, 1.0)]
}

pub struct HueCurveLut {
    pub delta_hue: [f32; LUT_WIDTH],
    pub sat_scale: [f32; LUT_WIDTH],
    pub delta_lum: [f32; LUT_WIDTH],
}

impl HueCurveLut {
    pub fn build(curves: &HueCurves) -> Self {
        let mut delta_hue = [0.0f32; LUT_WIDTH];
        let mut sat_scale = [0.0f32; LUT_WIDTH];
        let mut delta_lum = [0.0f32; LUT_WIDTH];

        for i in 0..LUT_WIDTH {
            let hue = (i as f64 + 0.5) / LUT_WIDTH as f64;
            let d_h = (HueCurves::eval(&curves.hue_vs_hue, hue) - 0.5) * 2.0 * MAX_HUE_SHIFT as f64;
            let s_s = (HueCurves::eval(&curves.hue_vs_sat, hue) - 0.5) * 2.0;
            let d_l = (HueCurves::eval(&curves.hue_vs_lum, hue) - 0.5) * 2.0 * MAX_LUM_SHIFT as f64;

            delta_hue[i] = d_h as f32;
            sat_scale[i] = s_s as f32;
            delta_lum[i] = d_l as f32;
        }

        Self {
            delta_hue,
            sat_scale,
            delta_lum,
        }
    }

    #[inline(always)]
    pub fn sample(&self, hue: f32) -> (f32, f32, f32) {
        let pos = fract(hue) * LUT_WIDTH as f32;
        let idx = (pos as usize) % LUT_WIDTH;
        let next_idx = (idx + 1) % LUT_WIDTH;
        let f = pos - pos.floor();

        let dh = self.delta_hue[idx] + (self.delta_hue[next_idx] - self.delta_hue[idx]) * f;
        let ss = self.sat_scale[idx] + (self.sat_scale[next_idx] - self.sat_scale[idx]) * f;
        let dl = self.delta_lum[idx] + (self.delta_lum[next_idx] - self.delta_lum[idx]) * f;
        (dh, ss, dl)
    }
}

pub fn apply_hue_curves(
    pixels: &mut [u8],
    width: u32,
    height: u32,
    params: &ResolvedEffectParams,
) {
    let curves_opt = params
        .string("curves")
        .and_then(|json| serde_json::from_str::<HueCurves>(json).ok());

    let Some(curves) = curves_opt else {
        return;
    };

    if curves.is_identity() {
        return;
    }

    let lut = HueCurveLut::build(&curves);
    let len = (width * height * 4) as usize;
    let buf = &mut pixels[..len.min(pixels.len())];

    for chunk in buf.chunks_exact_mut(4) {
        let r = chunk[0] as f32 / 255.0;
        let g = chunk[1] as f32 / 255.0;
        let b = chunk[2] as f32 / 255.0;

        let hsv = rgb2hsv([r, g, b]);
        let (dh, ss, dl) = lut.sample(hsv[0]);
        let gate = smoothstep(0.04, 0.18, hsv[1]);

        let h2 = fract(hsv[0] + dh * gate);
        let s2 = (hsv[1] * (1.0 + ss * gate)).clamp(0.0, 1.0);
        let v2 = (hsv[2] + dl * gate).clamp(0.0, 1.0);

        let out = hsv2rgb([h2, s2, v2]);

        chunk[0] = (out[0] * 255.0).round() as u8;
        chunk[1] = (out[1] * 255.0).round() as u8;
        chunk[2] = (out[2] * 255.0).round() as u8;
    }
}
