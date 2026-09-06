// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only
// Derived from Sources/PalmierPro/Compositing/ColorWheels.swift and Metal/Wheels.metal (GPLv3).

use super::types::ResolvedEffectParams;
use std::f64::consts::PI;

const CHROMA_LIFT: f64 = 0.2;
const CHROMA_GAIN: f64 = 0.35;
const CHROMA_GAMMA: f64 = 0.35;

pub fn hue_rgb(h: f64) -> (f64, f64, f64) {
    let x = (h - h.floor()) * 6.0;
    let f = x - x.floor();
    let idx = (x as usize) % 6;
    match idx {
        0 => (1.0, f, 0.0),
        1 => (1.0 - f, 1.0, 0.0),
        2 => (0.0, 1.0, f),
        3 => (0.0, 1.0 - f, 1.0),
        4 => (f, 0.0, 1.0),
        _ => (1.0, 0.0, 1.0 - f),
    }
}

pub fn chroma_offset(x: f64, y: f64) -> (f64, f64, f64) {
    let r = (x * x + y * y).sqrt().min(1.0);
    if r <= 1e-6 {
        return (0.0, 0.0, 0.0);
    }
    let angle = y.atan2(x) / (2.0 * PI);
    let (cr, cg, cb) = hue_rgb(angle);
    let mean = (cr + cg + cb) / 3.0;
    ((cr - mean) * r, (cg - mean) * r, (cb - mean) * r)
}

pub fn is_neutral(p: &ResolvedEffectParams) -> bool {
    p.value("lift_x") == 0.0
        && p.value("lift_y") == 0.0
        && p.value("lift_m") == 0.0
        && p.value("gamma_x") == 0.0
        && p.value("gamma_y") == 0.0
        && p.value("gamma_m") == 1.0
        && p.value("gain_x") == 0.0
        && p.value("gain_y") == 0.0
        && p.value("gain_m") == 1.0
}

pub fn coefficients(p: &ResolvedEffectParams) -> ([f32; 3], [f32; 3], [f32; 3]) {
    let lift = chroma_offset(p.value("lift_x"), p.value("lift_y"));
    let gamma = chroma_offset(p.value("gamma_x"), p.value("gamma_y"));
    let gain = chroma_offset(p.value("gain_x"), p.value("gain_y"));

    let lift_m = p.value("lift_m");
    let gamma_m = if p.values.contains_key("gamma_m") {
        p.value("gamma_m")
    } else {
        1.0
    };
    let gain_m = if p.values.contains_key("gain_m") {
        p.value("gain_m")
    } else {
        1.0
    };

    let l = |c: f64| (lift_m + c * CHROMA_LIFT) as f32;
    let g = |c: f64| (gain_m * (1.0 + c * CHROMA_GAIN)) as f32;
    let ig = |c: f64| (1.0 / (0.01f64.max(gamma_m * (1.0 + c * CHROMA_GAMMA)))) as f32;

    (
        [l(lift.0), l(lift.1), l(lift.2)],
        [g(gain.0), g(gain.1), g(gain.2)],
        [ig(gamma.0), ig(gamma.1), ig(gamma.2)],
    )
}

pub fn apply_color_wheels(
    pixels: &mut [u8],
    width: u32,
    height: u32,
    params: &ResolvedEffectParams,
) {
    if is_neutral(params) {
        return;
    }

    let (lift, gain, inv_gamma) = coefficients(params);

    let limit = ((width * height * 4) as usize).min(pixels.len());
    let buf = &mut pixels[..limit];

    for chunk in buf.chunks_exact_mut(4) {
        let r = chunk[0] as f32 / 255.0;
        let g = chunk[1] as f32 / 255.0;
        let b = chunk[2] as f32 / 255.0;

        let lit_r = (r * (1.0 - lift[0]) + lift[0]).max(0.0) * gain[0];
        let lit_g = (g * (1.0 - lift[1]) + lift[1]).max(0.0) * gain[1];
        let lit_b = (b * (1.0 - lift[2]) + lift[2]).max(0.0) * gain[2];

        let out_r = lit_r.powf(inv_gamma[0]).clamp(0.0, 1.0);
        let out_g = lit_g.powf(inv_gamma[1]).clamp(0.0, 1.0);
        let out_b = lit_b.powf(inv_gamma[2]).clamp(0.0, 1.0);

        chunk[0] = (out_r * 255.0).round() as u8;
        chunk[1] = (out_g * 255.0).round() as u8;
        chunk[2] = (out_b * 255.0).round() as u8;
    }
}
