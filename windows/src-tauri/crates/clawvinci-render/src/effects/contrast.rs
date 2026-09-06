// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only
// Derived from Core Image CIColorControls contrast.

use super::types::ResolvedEffectParams;

/// Contrast: scales RGB around mid-gray (0.5).
pub fn apply_contrast(pixels: &mut [u8], width: u32, height: u32, params: &ResolvedEffectParams) {
    let contrast = if params.values.contains_key("amount") {
        params.value("amount") as f32
    } else {
        1.0
    };

    if contrast == 1.0 {
        return;
    }

    let limit = ((width * height * 4) as usize).min(pixels.len());
    let buf = &mut pixels[..limit];

    for chunk in buf.chunks_exact_mut(4) {
        let r = chunk[0] as f32 / 255.0;
        let g = chunk[1] as f32 / 255.0;
        let b = chunk[2] as f32 / 255.0;

        let out_r = ((r - 0.5) * contrast + 0.5).clamp(0.0, 1.0);
        let out_g = ((g - 0.5) * contrast + 0.5).clamp(0.0, 1.0);
        let out_b = ((b - 0.5) * contrast + 0.5).clamp(0.0, 1.0);

        chunk[0] = (out_r * 255.0).round() as u8;
        chunk[1] = (out_g * 255.0).round() as u8;
        chunk[2] = (out_b * 255.0).round() as u8;
    }
}
