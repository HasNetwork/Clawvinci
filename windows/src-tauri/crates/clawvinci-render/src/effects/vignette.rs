// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only
// Derived from Metal/Vignette.metal (GPLv3).

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

/// Centered, shaped, feathered vignette from Vignette.metal.
/// roundness morphs a superellipse from rectangular to round.
/// midpoint sets where falloff starts; feather sets falloff width.
pub fn apply_vignette(pixels: &mut [u8], width: u32, height: u32, params: &ResolvedEffectParams) {
    let amount = params.value("amount") as f32;
    if amount == 0.0 || width == 0 || height == 0 || pixels.len() < (width * height * 4) as usize {
        return;
    }

    let midpoint = if params.values.contains_key("midpoint") {
        params.value("midpoint") as f32
    } else {
        0.5
    };
    let roundness = params.value("roundness") as f32;
    let feather = if params.values.contains_key("feather") {
        params.value("feather") as f32
    } else {
        0.5
    };

    let half_w = (width as f32 * 0.5).max(1.0);
    let half_h = (height as f32 * 0.5).max(1.0);
    let center_x = width as f32 * 0.5;
    let center_y = height as f32 * 0.5;

    // Morph exponent: -1 -> 6.0 (rect), +1 -> 2.0 (round)
    let p = 6.0 - 4.0 * ((roundness + 1.0) * 0.5);
    let inv_p = 1.0 / p;
    let edge0 = midpoint;
    let edge1 = midpoint + feather * 1.5 + 0.05;

    for y in 0..height {
        let dy = ((y as f32 + 0.5) - center_y) / half_h;
        let dy_p = dy.abs().powf(p);
        let row_offset = (y * width * 4) as usize;

        for x in 0..width {
            let dx = ((x as f32 + 0.5) - center_x) / half_w;
            let dx_p = dx.abs().powf(p);

            let dist = (dx_p + dy_p).powf(inv_p);
            let v = smoothstep(edge0, edge1, dist);

            let mult = 1.0 + amount * v;
            let idx = row_offset + (x * 4) as usize;

            let r = (pixels[idx] as f32 / 255.0) * mult;
            let g = (pixels[idx + 1] as f32 / 255.0) * mult;
            let b = (pixels[idx + 2] as f32 / 255.0) * mult;

            pixels[idx] = (r.clamp(0.0, 1.0) * 255.0).round() as u8;
            pixels[idx + 1] = (g.clamp(0.0, 1.0) * 255.0).round() as u8;
            pixels[idx + 2] = (b.clamp(0.0, 1.0) * 255.0).round() as u8;
        }
    }
}
