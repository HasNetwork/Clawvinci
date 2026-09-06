// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only
// Derived from Core Image CIColorControls saturation.

use super::types::ResolvedEffectParams;

/// Saturation: lerp between luma and RGB.
pub fn apply_saturation(pixels: &mut [u8], width: u32, height: u32, params: &ResolvedEffectParams) {
    let sat = if params.values.contains_key("amount") {
        params.value("amount") as f32
    } else {
        1.0
    };

    if sat == 1.0 {
        return;
    }

    let len = (width * height * 4) as usize;
    let buf = &mut pixels[..len.min(pixels.len())];

    for chunk in buf.chunks_exact_mut(4) {
        let r = chunk[0] as f32 / 255.0;
        let g = chunk[1] as f32 / 255.0;
        let b = chunk[2] as f32 / 255.0;

        let y = r * 0.2126 + g * 0.7152 + b * 0.0722;

        let out_r = (y + sat * (r - y)).clamp(0.0, 1.0);
        let out_g = (y + sat * (g - y)).clamp(0.0, 1.0);
        let out_b = (y + sat * (b - y)).clamp(0.0, 1.0);

        chunk[0] = (out_r * 255.0).round() as u8;
        chunk[1] = (out_g * 255.0).round() as u8;
        chunk[2] = (out_b * 255.0).round() as u8;
    }
}
