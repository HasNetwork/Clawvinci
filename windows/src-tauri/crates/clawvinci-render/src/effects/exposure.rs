// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only
// Derived from Core Image CIExposureAdjust.

use super::types::ResolvedEffectParams;

/// Exposure adjust: scales RGB by 2^ev (runs in linear light per linearizes flag).
pub fn apply_exposure(pixels: &mut [u8], width: u32, height: u32, params: &ResolvedEffectParams) {
    let ev = params.value("ev") as f32;
    if ev == 0.0 {
        return;
    }

    let mult = 2.0f32.powf(ev);
    let len = (width * height * 4) as usize;
    let buf = &mut pixels[..len.min(pixels.len())];

    for chunk in buf.chunks_exact_mut(4) {
        let r = (chunk[0] as f32 * mult).clamp(0.0, 255.0);
        let g = (chunk[1] as f32 * mult).clamp(0.0, 255.0);
        let b = (chunk[2] as f32 * mult).clamp(0.0, 255.0);

        chunk[0] = r.round() as u8;
        chunk[1] = g.round() as u8;
        chunk[2] = b.round() as u8;
    }
}
