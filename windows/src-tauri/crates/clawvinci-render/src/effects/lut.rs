// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only
// Derived from Metal/LUTTetra.metal (GPLv3).

use super::lut_loader::{CubeLut, LutLoader};
use super::types::ResolvedEffectParams;

#[inline(always)]
fn add_node(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

#[inline(always)]
fn mul_scalar(a: [f32; 3], s: f32) -> [f32; 3] {
    [a[0] * s, a[1] * s, a[2] * s]
}

/// Tetrahedral 3D-LUT interpolation matching Metal/LUTTetra.metal.
pub fn interpolate_tetrahedral(lut: &CubeLut, rgb: [f32; 3]) -> [f32; 3] {
    let n = lut.dimension as f32;
    let r = rgb[0].clamp(0.0, 1.0);
    let g = rgb[1].clamp(0.0, 1.0);
    let b = rgb[2].clamp(0.0, 1.0);

    let p_r = r * (n - 1.0);
    let p_g = g * (n - 1.0);
    let p_b = b * (n - 1.0);

    let max_base = n - 2.0;
    let b0_r = p_r.floor().clamp(0.0, max_base) as usize;
    let b0_g = p_g.floor().clamp(0.0, max_base) as usize;
    let b0_b = p_b.floor().clamp(0.0, max_base) as usize;

    let f_r = p_r - b0_r as f32;
    let f_g = p_g - b0_g as f32;
    let f_b = p_b - b0_b as f32;

    let c000 = lut.sample_node(b0_r, b0_g, b0_b);
    let c111 = lut.sample_node(b0_r + 1, b0_g + 1, b0_b + 1);

    if f_r >= f_g {
        if f_g >= f_b {
            let c100 = lut.sample_node(b0_r + 1, b0_g, b0_b);
            let c110 = lut.sample_node(b0_r + 1, b0_g + 1, b0_b);
            add_node(
                add_node(
                    add_node(mul_scalar(c000, 1.0 - f_r), mul_scalar(c100, f_r - f_g)),
                    mul_scalar(c110, f_g - f_b),
                ),
                mul_scalar(c111, f_b),
            )
        } else if f_r >= f_b {
            let c100 = lut.sample_node(b0_r + 1, b0_g, b0_b);
            let c101 = lut.sample_node(b0_r + 1, b0_g, b0_b + 1);
            add_node(
                add_node(
                    add_node(mul_scalar(c000, 1.0 - f_r), mul_scalar(c100, f_r - f_b)),
                    mul_scalar(c101, f_b - f_g),
                ),
                mul_scalar(c111, f_g),
            )
        } else {
            let c001 = lut.sample_node(b0_r, b0_g, b0_b + 1);
            let c101 = lut.sample_node(b0_r + 1, b0_g, b0_b + 1);
            add_node(
                add_node(
                    add_node(mul_scalar(c000, 1.0 - f_b), mul_scalar(c001, f_b - f_r)),
                    mul_scalar(c101, f_r - f_g),
                ),
                mul_scalar(c111, f_g),
            )
        }
    } else if f_b >= f_g {
        let c001 = lut.sample_node(b0_r, b0_g, b0_b + 1);
        let c011 = lut.sample_node(b0_r, b0_g + 1, b0_b + 1);
        add_node(
            add_node(
                add_node(mul_scalar(c000, 1.0 - f_b), mul_scalar(c001, f_b - f_g)),
                mul_scalar(c011, f_g - f_r),
            ),
            mul_scalar(c111, f_r),
        )
    } else if f_b >= f_r {
        let c010 = lut.sample_node(b0_r, b0_g + 1, b0_b);
        let c011 = lut.sample_node(b0_r, b0_g + 1, b0_b + 1);
        add_node(
            add_node(
                add_node(mul_scalar(c000, 1.0 - f_g), mul_scalar(c010, f_g - f_b)),
                mul_scalar(c011, f_b - f_r),
            ),
            mul_scalar(c111, f_r),
        )
    } else {
        let c010 = lut.sample_node(b0_r, b0_g + 1, b0_b);
        let c110 = lut.sample_node(b0_r + 1, b0_g + 1, b0_b);
        add_node(
            add_node(
                add_node(mul_scalar(c000, 1.0 - f_g), mul_scalar(c010, f_g - f_r)),
                mul_scalar(c110, f_r - f_b),
            ),
            mul_scalar(c111, f_b),
        )
    }
}

pub fn apply_lut(pixels: &mut [u8], width: u32, height: u32, params: &ResolvedEffectParams) {
    let intensity = if params.values.contains_key("intensity") {
        params.value("intensity") as f32
    } else {
        1.0
    };

    if intensity <= 0.0 {
        return;
    }

    let Some(path) = params.string("path") else {
        return;
    };

    let Some(lut) = LutLoader::load(path) else {
        return;
    };

    let len = (width * height * 4) as usize;
    let buf = &mut pixels[..len.min(pixels.len())];

    for chunk in buf.chunks_exact_mut(4) {
        let s_r = chunk[0] as f32 / 255.0;
        let s_g = chunk[1] as f32 / 255.0;
        let s_b = chunk[2] as f32 / 255.0;

        let o = interpolate_tetrahedral(&lut, [s_r, s_g, s_b]);

        let out_r = s_r + (o[0] - s_r) * intensity;
        let out_g = s_g + (o[1] - s_g) * intensity;
        let out_b = s_b + (o[2] - s_b) * intensity;

        chunk[0] = (out_r.clamp(0.0, 1.0) * 255.0).round() as u8;
        chunk[1] = (out_g.clamp(0.0, 1.0) * 255.0).round() as u8;
        chunk[2] = (out_b.clamp(0.0, 1.0) * 255.0).round() as u8;
    }
}
