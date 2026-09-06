// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only

#[inline(always)]
pub fn srgb_to_linear(c: f32) -> f32 {
    let c = c.clamp(0.0, 1.0);
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

#[inline(always)]
pub fn linear_to_srgb(c: f32) -> f32 {
    let c = c.clamp(0.0, 1.0);
    if c <= 0.0031308 {
        c * 12.92
    } else {
        1.055 * c.powf(1.0 / 2.4) - 0.055
    }
}

pub fn convert_srgb_to_linear(pixels: &mut [u8]) {
    for chunk in pixels.chunks_exact_mut(4) {
        let r = srgb_to_linear(chunk[0] as f32 / 255.0);
        let g = srgb_to_linear(chunk[1] as f32 / 255.0);
        let b = srgb_to_linear(chunk[2] as f32 / 255.0);
        chunk[0] = (r * 255.0).round() as u8;
        chunk[1] = (g * 255.0).round() as u8;
        chunk[2] = (b * 255.0).round() as u8;
    }
}

pub fn convert_linear_to_srgb(pixels: &mut [u8]) {
    for chunk in pixels.chunks_exact_mut(4) {
        let r = linear_to_srgb(chunk[0] as f32 / 255.0);
        let g = linear_to_srgb(chunk[1] as f32 / 255.0);
        let b = linear_to_srgb(chunk[2] as f32 / 255.0);
        chunk[0] = (r * 255.0).round() as u8;
        chunk[1] = (g * 255.0).round() as u8;
        chunk[2] = (b * 255.0).round() as u8;
    }
}
