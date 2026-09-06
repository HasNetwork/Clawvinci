// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only
// Derived from Metal/ChromaKey.metal (GPLv3).

use super::types::ResolvedEffectParams;

#[inline(always)]
fn fract(x: f32) -> f32 {
    x - x.floor()
}

#[inline(always)]
fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let span = edge1 - edge0;
    if span.abs() < 1e-6 {
        return if x >= edge1 { 1.0 } else { 0.0 };
    }
    let t = ((x - edge0) / span).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Chroma key: pixels near `keyHue` become transparent with a soft edge.
/// `spill` desaturates leftover key tint on edges. Unpremultiplied I/O.
pub fn apply_chroma_key(
    pixels: &mut [u8],
    width: u32,
    height: u32,
    params: &ResolvedEffectParams,
) {
    let key_hue = if params.values.contains_key("keyHue") {
        params.value("keyHue") as f32
    } else {
        0.333
    };
    let tolerance = params.value("tolerance") as f32;
    let softness = if params.values.contains_key("softness") {
        params.value("softness") as f32
    } else {
        0.1
    };
    let spill = if params.values.contains_key("spill") {
        params.value("spill") as f32
    } else {
        0.5
    };

    let inner = tolerance * 0.25;
    let outer = inner + softness * 0.3 + 0.02;

    let len = (width * height * 4) as usize;
    let buf = &mut pixels[..len.min(pixels.len())];

    for chunk in buf.chunks_exact_mut(4) {
        let mut r = chunk[0] as f32 / 255.0;
        let mut g = chunk[1] as f32 / 255.0;
        let mut b = chunk[2] as f32 / 255.0;
        let a = chunk[3] as f32 / 255.0;

        let mx = r.max(g).max(b);
        let mn = r.min(g).min(b);
        let dd = mx - mn;
        let sat = if mx <= 1e-5 { 0.0 } else { dd / mx };

        let mut hue = 0.0f32;
        if dd > 1e-5 {
            if mx == r {
                hue = (g - b) / dd;
            } else if mx == g {
                hue = (b - r) / dd + 2.0;
            } else {
                hue = (r - g) / dd + 4.0;
            }
            hue = fract(hue / 6.0);
        }

        let mut hd = (hue - key_hue).abs();
        hd = hd.min(1.0 - hd); // Circular hue distance [0, 0.5]

        let key = (1.0 - smoothstep(inner, outer, hd))
            * smoothstep(0.12, 0.32, sat)
            * smoothstep(0.04, 0.12, dd);

        let y = r * 0.2126 + g * 0.7152 + b * 0.0722;
        let spill_factor = spill * key;

        r = r * (1.0 - spill_factor) + y * spill_factor;
        g = g * (1.0 - spill_factor) + y * spill_factor;
        b = b * (1.0 - spill_factor) + y * spill_factor;

        let out_a = a * (1.0 - key);

        chunk[0] = (r.clamp(0.0, 1.0) * 255.0).round() as u8;
        chunk[1] = (g.clamp(0.0, 1.0) * 255.0).round() as u8;
        chunk[2] = (b.clamp(0.0, 1.0) * 255.0).round() as u8;
        chunk[3] = (out_a.clamp(0.0, 1.0) * 255.0).round() as u8;
    }
}
