// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only
// Derived from Core Image CIGaussianBlur, CISharpenLuminance, CINoiseReduction, CIMotionBlur.

use super::types::ResolvedEffectParams;
use std::f32::consts::PI;

/// Fast separable Gaussian blur on RGBA8 pixel buffer.
pub fn gaussian_blur(pixels: &mut [u8], width: u32, height: u32, radius: f32) {
    if radius <= 0.0 || width <= 1 || height <= 1 {
        return;
    }

    let sigma = radius;
    let k_radius = (2.0 * sigma).ceil().max(1.0) as i32;
    let k_size = (2 * k_radius + 1) as usize;

    let mut weights = Vec::with_capacity(k_size);
    let mut sum = 0.0f32;
    let two_sigma_sq = 2.0 * sigma * sigma;

    for i in -k_radius..=k_radius {
        let w = (-(i as f32 * i as f32) / two_sigma_sq).exp();
        weights.push(w);
        sum += w;
    }
    for w in &mut weights {
        *w /= sum;
    }

    let w_u = width as usize;
    let h_u = height as usize;
    let mut temp = vec![0u8; pixels.len()];

    // Horizontal pass: pixels -> temp
    for y in 0..h_u {
        let row_start = y * w_u * 4;
        for x in 0..w_u {
            let mut acc_r = 0.0f32;
            let mut acc_g = 0.0f32;
            let mut acc_b = 0.0f32;
            let mut acc_a = 0.0f32;

            for (k_idx, ki) in (-k_radius..=k_radius).enumerate() {
                let sx = (x as i32 + ki).clamp(0, width as i32 - 1) as usize;
                let s_idx = row_start + sx * 4;
                let weight = weights[k_idx];

                acc_r += pixels[s_idx] as f32 * weight;
                acc_g += pixels[s_idx + 1] as f32 * weight;
                acc_b += pixels[s_idx + 2] as f32 * weight;
                acc_a += pixels[s_idx + 3] as f32 * weight;
            }

            let d_idx = row_start + x * 4;
            temp[d_idx] = acc_r.round() as u8;
            temp[d_idx + 1] = acc_g.round() as u8;
            temp[d_idx + 2] = acc_b.round() as u8;
            temp[d_idx + 3] = acc_a.round() as u8;
        }
    }

    // Vertical pass: temp -> pixels
    for y in 0..h_u {
        let row_start = y * w_u * 4;
        for x in 0..w_u {
            let mut acc_r = 0.0f32;
            let mut acc_g = 0.0f32;
            let mut acc_b = 0.0f32;
            let mut acc_a = 0.0f32;

            for (k_idx, ki) in (-k_radius..=k_radius).enumerate() {
                let sy = (y as i32 + ki).clamp(0, height as i32 - 1) as usize;
                let s_idx = sy * w_u * 4 + x * 4;
                let weight = weights[k_idx];

                acc_r += temp[s_idx] as f32 * weight;
                acc_g += temp[s_idx + 1] as f32 * weight;
                acc_b += temp[s_idx + 2] as f32 * weight;
                acc_a += temp[s_idx + 3] as f32 * weight;
            }

            let d_idx = row_start + x * 4;
            pixels[d_idx] = acc_r.round() as u8;
            pixels[d_idx + 1] = acc_g.round() as u8;
            pixels[d_idx + 2] = acc_b.round() as u8;
            pixels[d_idx + 3] = acc_a.round() as u8;
        }
    }
}

pub fn apply_gaussian_blur(
    pixels: &mut [u8],
    width: u32,
    height: u32,
    params: &ResolvedEffectParams,
) {
    let radius = if params.values.contains_key("radius") {
        (params.value("radius") * params.spatial_scale) as f32
    } else {
        8.0 * params.spatial_scale as f32
    };
    gaussian_blur(pixels, width, height, radius);
}

pub fn apply_sharpen(pixels: &mut [u8], width: u32, height: u32, params: &ResolvedEffectParams) {
    let amount = if params.values.contains_key("amount") {
        params.value("amount") as f32
    } else {
        0.4
    };
    if amount <= 0.0 || width <= 1 || height <= 1 {
        return;
    }

    let mut blurred = pixels.to_vec();
    gaussian_blur(&mut blurred, width, height, 1.5);

    let len = (width * height * 4) as usize;
    let buf = &mut pixels[..len.min(pixels.len())];

    for (chunk, b_chunk) in buf.chunks_exact_mut(4).zip(blurred.chunks_exact(4)) {
        let r = chunk[0] as f32;
        let g = chunk[1] as f32;
        let b = chunk[2] as f32;

        let y = r * 0.2126 + g * 0.7152 + b * 0.0722;
        let by = b_chunk[0] as f32 * 0.2126 + b_chunk[1] as f32 * 0.7152 + b_chunk[2] as f32 * 0.0722;
        let dy = y - by;

        let out_r = (r + amount * dy).clamp(0.0, 255.0);
        let out_g = (g + amount * dy).clamp(0.0, 255.0);
        let out_b = (b + amount * dy).clamp(0.0, 255.0);

        chunk[0] = out_r.round() as u8;
        chunk[1] = out_g.round() as u8;
        chunk[2] = out_b.round() as u8;
    }
}

pub fn apply_noise_reduction(
    pixels: &mut [u8],
    width: u32,
    height: u32,
    params: &ResolvedEffectParams,
) {
    let amount = params.value("amount") as f32;
    if amount <= 0.0 || width <= 1 || height <= 1 {
        return;
    }

    // Bilateral filter on 5x5 window
    let radius = 2i32;
    let spatial_sigma = 2.0f32;
    let color_sigma = (amount * 40.0).max(5.0);
    let two_spatial_sq = 2.0 * spatial_sigma * spatial_sigma;
    let two_color_sq = 2.0 * color_sigma * color_sigma;

    let orig = pixels.to_vec();
    let w_u = width as usize;
    let h_u = height as usize;

    for y in 0..h_u {
        let row_start = y * w_u * 4;
        for x in 0..w_u {
            let center_idx = row_start + x * 4;
            let c_r = orig[center_idx] as f32;
            let c_g = orig[center_idx + 1] as f32;
            let c_b = orig[center_idx + 2] as f32;

            let mut w_sum = 0.0f32;
            let mut acc_r = 0.0f32;
            let mut acc_g = 0.0f32;
            let mut acc_b = 0.0f32;

            for dy in -radius..=radius {
                let ny = (y as i32 + dy).clamp(0, height as i32 - 1) as usize;
                let n_row = ny * w_u * 4;
                for dx in -radius..=radius {
                    let nx = (x as i32 + dx).clamp(0, width as i32 - 1) as usize;
                    let n_idx = n_row + nx * 4;

                    let nr = orig[n_idx] as f32;
                    let ng = orig[n_idx + 1] as f32;
                    let nb = orig[n_idx + 2] as f32;

                    let d_spatial_sq = (dx * dx + dy * dy) as f32;
                    let d_color_sq = (nr - c_r).powi(2) + (ng - c_g).powi(2) + (nb - c_b).powi(2);

                    let w = (-d_spatial_sq / two_spatial_sq - d_color_sq / two_color_sq).exp();
                    w_sum += w;
                    acc_r += nr * w;
                    acc_g += ng * w;
                    acc_b += nb * w;
                }
            }

            if w_sum > 1e-5 {
                pixels[center_idx] = (acc_r / w_sum).round() as u8;
                pixels[center_idx + 1] = (acc_g / w_sum).round() as u8;
                pixels[center_idx + 2] = (acc_b / w_sum).round() as u8;
            }
        }
    }
}

pub fn apply_motion_blur(
    pixels: &mut [u8],
    width: u32,
    height: u32,
    params: &ResolvedEffectParams,
) {
    let radius = params.value("radius") as f32;
    if radius <= 0.0 || width <= 1 || height <= 1 {
        return;
    }

    let angle_deg = params.value("angle") as f32;
    let angle_rad = angle_deg * PI / 180.0;
    let dx = angle_rad.cos();
    let dy = angle_rad.sin();

    let steps = radius.ceil().max(1.0) as i32;
    let orig = pixels.to_vec();
    let w_u = width as usize;
    let h_u = height as usize;

    for y in 0..h_u {
        let row_start = y * w_u * 4;
        for x in 0..w_u {
            let mut acc_r = 0.0f32;
            let mut acc_g = 0.0f32;
            let mut acc_b = 0.0f32;
            let mut count = 0.0f32;

            for step in -steps..=steps {
                let sx = (x as f32 + step as f32 * dx).round() as i32;
                let sy = (y as f32 + step as f32 * dy).round() as i32;

                if sx >= 0 && sx < width as i32 && sy >= 0 && sy < height as i32 {
                    let s_idx = sy as usize * w_u * 4 + sx as usize * 4;
                    acc_r += orig[s_idx] as f32;
                    acc_g += orig[s_idx + 1] as f32;
                    acc_b += orig[s_idx + 2] as f32;
                    count += 1.0;
                }
            }

            if count > 0.0 {
                let d_idx = row_start + x * 4;
                pixels[d_idx] = (acc_r / count).round() as u8;
                pixels[d_idx + 1] = (acc_g / count).round() as u8;
                pixels[d_idx + 2] = (acc_b / count).round() as u8;
            }
        }
    }
}
