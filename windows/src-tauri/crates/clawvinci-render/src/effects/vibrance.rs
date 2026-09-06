// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only
// Derived from Core Image CIVibrance.

use super::types::ResolvedEffectParams;

/// Vibrance: selective saturation boost where less-saturated colors are boosted more.
pub fn apply_vibrance(pixels: &mut [u8], width: u32, height: u32, params: &ResolvedEffectParams) {
    let amount = params.value("amount") as f32;
    if amount == 0.0 {
        return;
    }

    let limit = ((width * height * 4) as usize).min(pixels.len());
    let buf = &mut pixels[..limit];

    for chunk in buf.chunks_exact_mut(4) {
        let r = chunk[0] as f32 / 255.0;
        let g = chunk[1] as f32 / 255.0;
        let b = chunk[2] as f32 / 255.0;

        let mx = r.max(g).max(b);
        let mn = r.min(g).min(b);
        let sat = mx - mn;
        let mask = 1.0 - sat;
        let boost = 1.0 + amount * mask;

        let y = r * 0.2126 + g * 0.7152 + b * 0.0722;

        let out_r = (y + (r - y) * boost).clamp(0.0, 1.0);
        let out_g = (y + (g - y) * boost).clamp(0.0, 1.0);
        let out_b = (y + (b - y) * boost).clamp(0.0, 1.0);

        chunk[0] = (out_r * 255.0).round() as u8;
        chunk[1] = (out_g * 255.0).round() as u8;
        chunk[2] = (out_b * 255.0).round() as u8;
    }
}
