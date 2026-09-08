// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only
// Derived from Sources/PalmierPro/Search/Indexing/FrameSampler.swift (GPLv3).

use serde::{Deserialize, Serialize};

pub const SAMPLER_VERSION: i32 = 1;
pub const LUMA_GRID_CELLS: usize = 8;
pub const LUMA_GRID_SIZE: usize = LUMA_GRID_CELLS * LUMA_GRID_CELLS; // 64

/// Mean luma per cell of an 8×8 downsample — cheap visual-change fingerprint.
pub struct LumaGrid;

impl LumaGrid {
    /// Computes an 8x8 luma grid from an RGB or RGBA pixel buffer (width × height).
    /// Uses ITU-R BT.601 coefficients: L = 0.299 * R + 0.587 * G + 0.114 * B.
    pub fn compute_from_rgba(pixels: &[u8], width: u32, height: u32) -> [f32; LUMA_GRID_SIZE] {
        let mut grid = [0.0f32; LUMA_GRID_SIZE];
        let mut counts = [0u32; LUMA_GRID_SIZE];

        if width == 0 || height == 0 || pixels.is_empty() {
            return grid;
        }

        let channels = (pixels.len() / (width as usize * height as usize)).max(3);

        for y in 0..height {
            let cell_y = (y as usize * LUMA_GRID_CELLS) / height as usize;
            for x in 0..width {
                let cell_x = (x as usize * LUMA_GRID_CELLS) / width as usize;
                let cell_idx = cell_y * LUMA_GRID_CELLS + cell_x;

                let px_idx = (y as usize * width as usize + x as usize) * channels;
                if px_idx + 2 < pixels.len() {
                    let r = pixels[px_idx] as f32;
                    let g = pixels[px_idx + 1] as f32;
                    let b = pixels[px_idx + 2] as f32;
                    let luma = 0.299 * r + 0.587 * g + 0.114 * b;
                    grid[cell_idx] += luma;
                    counts[cell_idx] += 1;
                }
            }
        }

        for (item, count) in grid.iter_mut().zip(counts.iter()) {
            if *count > 0 {
                *item /= *count as f32;
            }
        }

        grid
    }

    /// Computes the mean absolute difference across cells between two 8x8 luma grids.
    pub fn mean_diff(a: &[f32; LUMA_GRID_SIZE], b: &[f32; LUMA_GRID_SIZE]) -> f32 {
        let sum: f32 = a.iter().zip(b.iter()).map(|(x, y)| (x - y).abs()).sum();
        sum / LUMA_GRID_SIZE as f32
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FrameSamplerOptions {
    pub candidate_interval: f64,
    pub coverage_floor: f64,
    pub promote_diff: f32,
    pub max_size: (u32, u32),
    pub high_res_edge: u32,
}

impl Default for FrameSamplerOptions {
    fn default() -> Self {
        Self {
            candidate_interval: 2.0,
            coverage_floor: 8.0,
            promote_diff: 12.0,
            max_size: (512, 512),
            high_res_edge: 3000,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SampleDecision {
    pub time: f64,
    pub is_new_shot: bool,
}

/// State machine tracking scene changes and coverage floor across a stream of sampled frames.
pub struct FrameSamplerState {
    options: FrameSamplerOptions,
    last_grid: Option<[f32; LUMA_GRID_SIZE]>,
    last_kept_time: f64,
}

impl FrameSamplerState {
    pub fn new(options: FrameSamplerOptions) -> Self {
        Self {
            options,
            last_grid: None,
            last_kept_time: f64::NEG_INFINITY,
        }
    }

    /// Evaluates a candidate frame. Returns `Some(decision)` if frame should be indexed.
    pub fn evaluate(&mut self, time: f64, grid: [f32; LUMA_GRID_SIZE]) -> Option<SampleDecision> {
        let is_new_shot = if let Some(last) = self.last_grid {
            LumaGrid::mean_diff(&grid, &last) > self.options.promote_diff
        } else {
            true
        };
        self.last_grid = Some(grid);

        if is_new_shot || (time - self.last_kept_time >= self.options.coverage_floor) {
            self.last_kept_time = time;
            Some(SampleDecision { time, is_new_shot })
        } else {
            None
        }
    }

    /// Generates the list of candidate sample timestamps for a video of given duration.
    pub fn candidate_times(duration: f64, mut interval: f64, is_high_res: bool) -> Vec<f64> {
        if is_high_res {
            interval *= 2.0;
        }
        if duration <= 0.0 {
            return Vec::new();
        }

        let mut times = Vec::new();
        let mut t = interval / 2.0;
        while t < duration {
            times.push(t);
            t += interval;
        }

        if times.is_empty() {
            times.push(duration / 2.0);
        }

        times
    }
}
