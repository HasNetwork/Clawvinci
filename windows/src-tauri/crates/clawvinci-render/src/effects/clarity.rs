// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only
// Derived from Metal/Clarity.metal and Sources/PalmierPro/Compositing/Kernels/ClarityKernel.swift (GPLv3).

use super::blur::gaussian_blur;
use super::types::ResolvedEffectParams;

#[inline(always)]
fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let span = edge1 - edge0;
    if span.abs() < 1e-6 {
        return if x >= edge1 { 1.0 } else { 0.0 };
    }
    let t = ((x - edge0) / span).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Clarity (local-contrast unsharp against mid-radius blur) + Dehaze.
pub fn apply_clarity(pixels: &mut [u8], width: u32, height: u32, params: &ResolvedEffectParams) {
    let clarity = params.value("clarity") as f32;
    let dehaze = params.value("dehaze") as f32;

    if (clarity == 0.0 && dehaze == 0.0) || width <= 1 || height <= 1 {
        return;
    }

    let radius = ((width.max(height) as f32) / 40.0).max(1.0);
    let mut blurred = pixels.to_vec();
    gaussian_blur(&mut blurred, width, height, radius);

    let len = (width * height * 4) as usize;
    let buf = &mut pixels[..len.min(pixels.len())];

    for (chunk, b_chunk) in buf.chunks_exact_mut(4).zip(blurred.chunks_exact(4)) {
        let s_r = chunk[0] as f32 / 255.0;
        let s_g = chunk[1] as f32 / 255.0;
        let s_b = chunk[2] as f32 / 255.0;

        let b_r = b_chunk[0] as f32 / 255.0;
        let b_g = b_chunk[1] as f32 / 255.0;
        let b_b = b_chunk[2] as f32 / 255.0;

        let mut r = s_r + (s_r - b_r) * clarity;
        let mut g = s_g + (s_g - b_g) * clarity;
        let mut b = s_b + (s_b - b_b) * clarity;

        if dehaze != 0.0 {
            let dark = s_r.min(s_g).min(s_b);
            let w = dehaze * (0.5 + 0.5 * smoothstep(0.05, 0.5, dark));

            r += (s_r - b_r) * (w * 0.6);
            g += (s_g - b_g) * (w * 0.6);
            b += (s_b - b_b) * (w * 0.6);

            let c_factor = 1.0 + w * 0.45;
            r = 0.45 + (r - 0.45) * c_factor;
            g = 0.45 + (g - 0.45) * c_factor;
            b = 0.45 + (b - 0.45) * c_factor;

            let yy = r * 0.2126 + g * 0.7152 + b * 0.0722;
            let sat_factor = 1.0 + w * 0.5;
            r = yy + (r - yy) * sat_factor;
            g = yy + (g - yy) * sat_factor;
            b = yy + (b - yy) * sat_factor;
        }

        chunk[0] = (r.clamp(0.0, 1.0) * 255.0).round() as u8;
        chunk[1] = (g.clamp(0.0, 1.0) * 255.0).round() as u8;
        chunk[2] = (b.clamp(0.0, 1.0) * 255.0).round() as u8;
    }
}
