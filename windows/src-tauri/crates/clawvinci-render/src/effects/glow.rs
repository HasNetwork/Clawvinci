// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only
// Derived from Metal/Glow.metal and Sources/PalmierPro/Compositing/Kernels/GlowKernel.swift (GPLv3).

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

/// Multi-pass glow / halation: isolates bright highlights, blurs, and screen-blends back.
pub fn apply_glow(pixels: &mut [u8], width: u32, height: u32, params: &ResolvedEffectParams) {
    let intensity = params.value("intensity") as f32;
    if intensity <= 0.0 || width <= 1 || height <= 1 {
        return;
    }

    let radius = if params.values.contains_key("radius") {
        (params.value("radius") * params.spatial_scale) as f32
    } else {
        20.0 * params.spatial_scale as f32
    };
    let threshold = if params.values.contains_key("threshold") {
        params.value("threshold") as f32
    } else {
        0.6
    };
    let warmth = params.value("warmth") as f32;

    let len = (width * height * 4) as usize;
    let mut bright_layer = vec![0u8; len];

    // Pass 1: glowBright
    for (src_chunk, dst_chunk) in pixels.chunks_exact(4).zip(bright_layer.chunks_exact_mut(4)) {
        let r = src_chunk[0] as f32 / 255.0;
        let g = src_chunk[1] as f32 / 255.0;
        let b = src_chunk[2] as f32 / 255.0;

        let y = r * 0.2126 + g * 0.7152 + b * 0.0722;
        let factor = smoothstep(threshold, 1.0, y);

        let hi_r = r * factor;
        let hi_g = g * factor;
        let hi_b = b * factor;

        let warm_r = hi_r * 1.0;
        let warm_g = hi_g * 0.7;
        let warm_b = hi_b * 0.45;

        let out_r = hi_r + (warm_r - hi_r) * warmth;
        let out_g = hi_g + (warm_g - hi_g) * warmth;
        let out_b = hi_b + (warm_b - hi_b) * warmth;

        dst_chunk[0] = (out_r.clamp(0.0, 1.0) * 255.0).round() as u8;
        dst_chunk[1] = (out_g.clamp(0.0, 1.0) * 255.0).round() as u8;
        dst_chunk[2] = (out_b.clamp(0.0, 1.0) * 255.0).round() as u8;
        dst_chunk[3] = src_chunk[3];
    }

    // Pass 2: blur bright layer
    gaussian_blur(&mut bright_layer, width, height, radius);

    // Pass 3: glowComposite (screen blend)
    let buf = &mut pixels[..len.min(pixels.len())];
    for (chunk, glow_chunk) in buf.chunks_exact_mut(4).zip(bright_layer.chunks_exact(4)) {
        let s_r = chunk[0] as f32 / 255.0;
        let s_g = chunk[1] as f32 / 255.0;
        let s_b = chunk[2] as f32 / 255.0;

        let g_r = (glow_chunk[0] as f32 / 255.0 * intensity).clamp(0.0, 1.0);
        let g_g = (glow_chunk[1] as f32 / 255.0 * intensity).clamp(0.0, 1.0);
        let g_b = (glow_chunk[2] as f32 / 255.0 * intensity).clamp(0.0, 1.0);

        let out_r = 1.0 - (1.0 - s_r) * (1.0 - g_r);
        let out_g = 1.0 - (1.0 - s_g) * (1.0 - g_g);
        let out_b = 1.0 - (1.0 - s_b) * (1.0 - g_b);

        chunk[0] = (out_r.clamp(0.0, 1.0) * 255.0).round() as u8;
        chunk[1] = (out_g.clamp(0.0, 1.0) * 255.0).round() as u8;
        chunk[2] = (out_b.clamp(0.0, 1.0) * 255.0).round() as u8;
    }
}
