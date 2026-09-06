// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only
// Derived from Metal/EdgeRounding.metal (GPLv3).

#[inline(always)]
fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let span = edge1 - edge0;
    if span.abs() < 1e-6 {
        return if x >= edge1 { 1.0 } else { 0.0 };
    }
    let t = ((x - edge0) / span).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Applies corner rounding and softness to an RGBA pixel buffer.
/// Uses exact signed distance field (SDF) of a rounded box from EdgeRounding.metal.
pub fn apply_edge_rounding(
    pixels: &mut [u8],
    width: u32,
    height: u32,
    edge_rounding: f32,
    edge_softness: f32,
) {
    if edge_rounding <= 0.0 && edge_softness <= 0.0 {
        return;
    }
    if width == 0 || height == 0 || pixels.len() < (width * height * 4) as usize {
        return;
    }

    let min_dim = (width as f32).min(height as f32);
    let radius = edge_rounding.clamp(0.0, 1.0) * min_dim * 0.5;
    let feather = edge_softness.clamp(0.0, 1.0) * min_dim * 0.5;

    let center_x = width as f32 * 0.5;
    let center_y = height as f32 * 0.5;
    let inset_half_w = width as f32 * 0.5 - radius;
    let inset_half_h = height as f32 * 0.5 - radius;

    let edge0 = -0.5 - feather;
    let edge1 = 0.5;

    for y in 0..height {
        let py = (y as f32 + 0.5) - center_y;
        let q_y = py.abs() - inset_half_h;
        let row_offset = (y * width * 4) as usize;

        for x in 0..width {
            let px = (x as f32 + 0.5) - center_x;
            let q_x = px.abs() - inset_half_w;

            let max_qx = q_x.max(0.0);
            let max_qy = q_y.max(0.0);
            let len = (max_qx * max_qx + max_qy * max_qy).sqrt();
            let distance = len + q_x.max(q_y).min(0.0) - radius;

            let coverage = 1.0 - smoothstep(edge0, edge1, distance);
            let idx = row_offset + (x * 4) as usize;

            let old_a = pixels[idx + 3] as f32 / 255.0;
            let new_a = (old_a * coverage).clamp(0.0, 1.0);
            pixels[idx + 3] = (new_a * 255.0).round() as u8;
        }
    }
}
