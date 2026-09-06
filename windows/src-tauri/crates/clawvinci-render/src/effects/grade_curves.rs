// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only
// Derived from Metal/GradeCurves.metal and Sources/PalmierPro/Compositing/Kernels/GradeCurveKernel.swift (GPLv3).

use super::types::ResolvedEffectParams;
use clawvinci_model::grade::GradeCurve;

const LUT_WIDTH: usize = 256;

pub struct GradeLuts {
    pub red: [f32; LUT_WIDTH],
    pub green: [f32; LUT_WIDTH],
    pub blue: [f32; LUT_WIDTH],
    pub master: [f32; LUT_WIDTH],
}

impl GradeLuts {
    pub fn build(curve: &GradeCurve) -> Self {
        let mut red = [0.0f32; LUT_WIDTH];
        let mut green = [0.0f32; LUT_WIDTH];
        let mut blue = [0.0f32; LUT_WIDTH];
        let mut master = [0.0f32; LUT_WIDTH];

        for i in 0..LUT_WIDTH {
            let t = i as f64 / (LUT_WIDTH - 1) as f64;
            red[i] = GradeCurve::eval(&curve.red, t).clamp(0.0, 1.0) as f32;
            green[i] = GradeCurve::eval(&curve.green, t).clamp(0.0, 1.0) as f32;
            blue[i] = GradeCurve::eval(&curve.blue, t).clamp(0.0, 1.0) as f32;
            master[i] = GradeCurve::eval(&curve.master, t).clamp(0.0, 1.0) as f32;
        }

        Self {
            red,
            green,
            blue,
            master,
        }
    }

    #[inline(always)]
    fn sample_lut(lut: &[f32; LUT_WIDTH], v: f32) -> f32 {
        let v_clamped = v.clamp(0.0, 1.0);
        let pos = v_clamped * (LUT_WIDTH - 1) as f32;
        let idx = pos as usize;
        let frac = pos - idx as f32;
        let next_idx = (idx + 1).min(LUT_WIDTH - 1);
        lut[idx] + (lut[next_idx] - lut[idx]) * frac
    }

    pub fn apply_pixel(&self, rgb: [f32; 3]) -> [f32; 3] {
        let r = rgb[0].clamp(0.0, 1.0);
        let g = rgb[1].clamp(0.0, 1.0);
        let b = rgb[2].clamp(0.0, 1.0);

        let y = r * 0.2126 + g * 0.7152 + b * 0.0722;
        let yp = Self::sample_lut(&self.master, y);

        // Luma-preserving rescale capped at 8.0 gain
        let (scaled_r, scaled_g, scaled_b) = if y > 1e-4 {
            let scale = (yp / y).min(8.0);
            (r * scale, g * scale, b * scale)
        } else {
            (yp, yp, yp)
        };

        [
            Self::sample_lut(&self.red, scaled_r),
            Self::sample_lut(&self.green, scaled_g),
            Self::sample_lut(&self.blue, scaled_b),
        ]
    }
}

pub fn apply_grade_curves(
    pixels: &mut [u8],
    width: u32,
    height: u32,
    params: &ResolvedEffectParams,
) {
    let curve_opt = params
        .string("curve")
        .and_then(|json| serde_json::from_str::<GradeCurve>(json).ok());

    let Some(curve) = curve_opt else {
        return;
    };

    if curve.is_identity() {
        return;
    }

    let luts = GradeLuts::build(&curve);
    let limit = ((width * height * 4) as usize).min(pixels.len());
    let buf = &mut pixels[..limit];

    for chunk in buf.chunks_exact_mut(4) {
        let r = chunk[0] as f32 / 255.0;
        let g = chunk[1] as f32 / 255.0;
        let b = chunk[2] as f32 / 255.0;

        let out = luts.apply_pixel([r, g, b]);

        chunk[0] = (out[0].clamp(0.0, 1.0) * 255.0).round() as u8;
        chunk[1] = (out[1].clamp(0.0, 1.0) * 255.0).round() as u8;
        chunk[2] = (out[2].clamp(0.0, 1.0) * 255.0).round() as u8;
    }
}
