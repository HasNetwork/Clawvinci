// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only
// Derived from Metal/Levels.metal (GPLv3).

use super::types::ResolvedEffectParams;

/// Levels: independent black/white-point remap (per-channel linear stretch).
/// blacks: <0 crush / >0 lift the floor. whites: >0 brighten/clip / <0 recover.
pub fn apply_levels(pixels: &mut [u8], width: u32, height: u32, params: &ResolvedEffectParams) {
    let blacks = params.value("blacks") as f32;
    let whites = params.value("whites") as f32;

    if blacks == 0.0 && whites == 0.0 {
        return;
    }

    let bp = -blacks * 0.4;
    let wp = 1.0 - whites * 0.4;
    let denom = (wp - bp).max(0.05);

    let limit = ((width * height * 4) as usize).min(pixels.len());
    let buf = &mut pixels[..limit];

    for chunk in buf.chunks_exact_mut(4) {
        let r = chunk[0] as f32 / 255.0;
        let g = chunk[1] as f32 / 255.0;
        let b = chunk[2] as f32 / 255.0;

        let out_r = ((r - bp) / denom).clamp(0.0, 1.0);
        let out_g = ((g - bp) / denom).clamp(0.0, 1.0);
        let out_b = ((b - bp) / denom).clamp(0.0, 1.0);

        chunk[0] = (out_r * 255.0).round() as u8;
        chunk[1] = (out_g * 255.0).round() as u8;
        chunk[2] = (out_b * 255.0).round() as u8;
    }
}
