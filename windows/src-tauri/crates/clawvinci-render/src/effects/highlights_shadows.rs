// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only
// Derived from Metal/HighlightsShadows.metal (GPLv3).

use super::types::ResolvedEffectParams;

/// Luma-masked highlights & shadows. Adds a tone-region-weighted luminance delta to RGB.
pub fn apply_highlights_shadows(
    pixels: &mut [u8],
    width: u32,
    height: u32,
    params: &ResolvedEffectParams,
) {
    let highlights = params.value("highlights") as f32;
    let shadows = params.value("shadows") as f32;

    if highlights == 0.0 && shadows == 0.0 {
        return;
    }

    let len = (width * height * 4) as usize;
    let buf = &mut pixels[..len.min(pixels.len())];

    for chunk in buf.chunks_exact_mut(4) {
        let r = chunk[0] as f32 / 255.0;
        let g = chunk[1] as f32 / 255.0;
        let b = chunk[2] as f32 / 255.0;

        let y = r * 0.2126 + g * 0.7152 + b * 0.0722;
        let hi = y * y * y;
        let lo = (1.0 - y) * (1.0 - y) * (1.0 - y);
        let dy = (highlights * hi + shadows * lo) * 0.5;

        let out_r = (r + dy).clamp(0.0, 1.0);
        let out_g = (g + dy).clamp(0.0, 1.0);
        let out_b = (b + dy).clamp(0.0, 1.0);

        chunk[0] = (out_r * 255.0).round() as u8;
        chunk[1] = (out_g * 255.0).round() as u8;
        chunk[2] = (out_b * 255.0).round() as u8;
    }
}
