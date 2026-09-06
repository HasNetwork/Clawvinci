// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only
// Derived from Metal/Grain.metal (GPLv3).

use super::types::ResolvedEffectParams;

#[inline(always)]
fn fract(x: f32) -> f32 {
    x - x.floor()
}

#[inline(always)]
fn hash13(p3: [f32; 3]) -> f32 {
    let mut p = [
        fract(p3[0] * 0.1031),
        fract(p3[1] * 0.1031),
        fract(p3[2] * 0.1031),
    ];
    let d = p[0] * (p[2] + 31.32) + p[1] * (p[1] + 31.32) + p[2] * (p[0] + 31.32);
    p[0] += d;
    p[1] += d;
    p[2] += d;
    fract((p[0] + p[1]) * p[2])
}

/// Film grain: monochromatic, position+frame-seeded noise, strongest in mid-tones.
pub fn apply_grain(pixels: &mut [u8], width: u32, height: u32, params: &ResolvedEffectParams) {
    let amount = params.value("amount") as f32;
    if amount <= 0.0 || width == 0 || height == 0 || pixels.len() < (width * height * 4) as usize {
        return;
    }

    let size = if params.values.contains_key("size") {
        (params.value("size") as f32).max(0.5)
    } else {
        1.5
    };
    let frame = params.frame as f32;

    for y in 0..height {
        let co_y = (y as f32 + 0.5) / size;
        let row_offset = (y * width * 4) as usize;

        for x in 0..width {
            let co_x = (x as f32 + 0.5) / size;
            let n = hash13([co_x, co_y, frame]) - 0.5;

            let idx = row_offset + (x * 4) as usize;
            let r = pixels[idx] as f32 / 255.0;
            let g = pixels[idx + 1] as f32 / 255.0;
            let b = pixels[idx + 2] as f32 / 255.0;

            let luma = r * 0.2126 + g * 0.7152 + b * 0.0722;
            let luma_mask = 4.0 * luma * (1.0 - luma);
            let delta = n * amount * 0.35 * luma_mask;

            pixels[idx] = ((r + delta).clamp(0.0, 1.0) * 255.0).round() as u8;
            pixels[idx + 1] = ((g + delta).clamp(0.0, 1.0) * 255.0).round() as u8;
            pixels[idx + 2] = ((b + delta).clamp(0.0, 1.0) * 255.0).round() as u8;
        }
    }
}
