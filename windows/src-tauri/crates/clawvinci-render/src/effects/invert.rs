// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only
// Derived from Core Image CIColorMatrix invert.

use super::types::ResolvedEffectParams;

/// Invert: 1.0 - RGB, preserves alpha.
pub fn apply_invert(pixels: &mut [u8], width: u32, height: u32, _params: &ResolvedEffectParams) {
    let len = (width * height * 4) as usize;
    let buf = &mut pixels[..len.min(pixels.len())];

    for chunk in buf.chunks_exact_mut(4) {
        chunk[0] = 255 - chunk[0];
        chunk[1] = 255 - chunk[1];
        chunk[2] = 255 - chunk[2];
    }
}
