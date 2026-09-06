// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only
// Derived from Core Image CITemperatureAndTint.

use super::types::ResolvedEffectParams;

/// Temperature & Tint: chromatic adaptation from source neutral (temp, tint) to 6500K target.
pub fn apply_temperature(
    pixels: &mut [u8],
    width: u32,
    height: u32,
    params: &ResolvedEffectParams,
) {
    let temp = if params.values.contains_key("temperature") {
        params.value("temperature") as f32
    } else {
        6500.0
    };
    let tint = params.value("tint") as f32;

    if (temp - 6500.0).abs() < 1e-3 && tint.abs() < 1e-3 {
        return;
    }

    // Normalized shifts: temperature relative to 6500K D65, tint relative to [-100, 100]
    let t_norm = (temp - 6500.0) / 4500.0;
    let tint_norm = tint / 100.0;

    // Red/Blue temperature axis and Green/Magenta tint axis
    let r_mult = (1.0 + t_norm * 0.4 - tint_norm * 0.08).max(0.0);
    let g_mult = (1.0 + tint_norm * 0.2).max(0.0);
    let b_mult = (1.0 - t_norm * 0.4 - tint_norm * 0.08).max(0.0);

    let len = (width * height * 4) as usize;
    let buf = &mut pixels[..len.min(pixels.len())];

    for chunk in buf.chunks_exact_mut(4) {
        let r = chunk[0] as f32 / 255.0;
        let g = chunk[1] as f32 / 255.0;
        let b = chunk[2] as f32 / 255.0;

        let out_r = (r * r_mult).clamp(0.0, 1.0);
        let out_g = (g * g_mult).clamp(0.0, 1.0);
        let out_b = (b * b_mult).clamp(0.0, 1.0);

        chunk[0] = (out_r * 255.0).round() as u8;
        chunk[1] = (out_g * 255.0).round() as u8;
        chunk[2] = (out_b * 255.0).round() as u8;
    }
}
