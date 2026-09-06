// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only
// Derived from Sources/PalmierPro/Compositing/LUTLoader.swift (GPLv3).

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// A parsed 3D Cube LUT with RGBA float data.
#[derive(Debug, Clone, PartialEq)]
pub struct CubeLut {
    pub dimension: usize,
    pub data: Vec<f32>, // dimension^3 * 4 floats: [R, G, B, A]
}

impl CubeLut {
    /// Look up RGB node at integer lattice coordinate (r, g, b) where r, g, b in 0..dimension.
    #[inline(always)]
    pub fn sample_node(&self, r: usize, g: usize, b: usize) -> [f32; 3] {
        let n = self.dimension;
        let idx = ((b * n + g) * n + r) * 4;
        [self.data[idx], self.data[idx + 1], self.data[idx + 2]]
    }
}

pub struct LutLoader;

static LUT_CACHE: Mutex<Option<HashMap<String, Arc<CubeLut>>>> = Mutex::new(None);

impl LutLoader {
    /// Parses a .cube format string into a CubeLut.
    pub fn parse(text: &str) -> Option<CubeLut> {
        let mut dimension = 0usize;
        let mut domain_min = [0.0f32, 0.0, 0.0];
        let mut domain_max = [1.0f32, 1.0, 1.0];
        let mut values = Vec::new();

        for raw_line in text.lines() {
            let line = raw_line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.is_empty() {
                continue;
            }

            match parts[0].to_ascii_uppercase().as_str() {
                "TITLE" => continue,
                "LUT_1D_SIZE" => return None, // 1D LUTs not supported
                "LUT_3D_SIZE" => {
                    if let Some(dim_str) = parts.get(1) {
                        dimension = dim_str.parse().unwrap_or(0);
                    }
                }
                "DOMAIN_MIN" => {
                    if parts.len() >= 4 {
                        domain_min[0] = parts[1].parse().unwrap_or(0.0);
                        domain_min[1] = parts[2].parse().unwrap_or(0.0);
                        domain_min[2] = parts[3].parse().unwrap_or(0.0);
                    }
                }
                "DOMAIN_MAX" => {
                    if parts.len() >= 4 {
                        domain_max[0] = parts[1].parse().unwrap_or(1.0);
                        domain_max[1] = parts[2].parse().unwrap_or(1.0);
                        domain_max[2] = parts[3].parse().unwrap_or(1.0);
                    }
                }
                _ => {
                    if parts.len() >= 3 {
                        if let (Ok(r), Ok(g), Ok(b)) = (
                            parts[0].parse::<f32>(),
                            parts[1].parse::<f32>(),
                            parts[2].parse::<f32>(),
                        ) {
                            values.push(r);
                            values.push(g);
                            values.push(b);
                        } else {
                            return None;
                        }
                    }
                }
            }
        }

        if !(2..=128).contains(&dimension) {
            return None;
        }

        let expected_values = dimension * dimension * dimension * 3;
        if values.len() != expected_values {
            return None;
        }

        let mut rgba = Vec::with_capacity(dimension * dimension * dimension * 4);
        let span_r = (domain_max[0] - domain_min[0]).max(0.0001);
        let span_g = (domain_max[1] - domain_min[1]).max(0.0001);
        let span_b = (domain_max[2] - domain_min[2]).max(0.0001);

        for chunk in values.chunks_exact(3) {
            let r = ((chunk[0] - domain_min[0]) / span_r).clamp(0.0, 1.0);
            let g = ((chunk[1] - domain_min[1]) / span_g).clamp(0.0, 1.0);
            let b = ((chunk[2] - domain_min[2]) / span_b).clamp(0.0, 1.0);

            rgba.push(r);
            rgba.push(g);
            rgba.push(b);
            rgba.push(1.0);
        }

        Some(CubeLut {
            dimension,
            data: rgba,
        })
    }

    /// Loads a .cube file from path, utilizing an in-memory cache.
    pub fn load(path: &str) -> Option<Arc<CubeLut>> {
        {
            let mut guard = LUT_CACHE.lock().unwrap();
            let cache = guard.get_or_insert_with(HashMap::new);
            if let Some(lut) = cache.get(path) {
                return Some(Arc::clone(lut));
            }
        }

        let content = std::fs::read_to_string(path).ok()?;
        let lut = Arc::new(Self::parse(&content)?);

        let mut guard = LUT_CACHE.lock().unwrap();
        let cache = guard.get_or_insert_with(HashMap::new);
        cache.insert(path.to_string(), Arc::clone(&lut));
        Some(lut)
    }

    /// Caches a parsed LUT directly in memory under a specified key.
    pub fn cache_lut(key: &str, lut: CubeLut) {
        let mut guard = LUT_CACHE.lock().unwrap();
        let cache = guard.get_or_insert_with(HashMap::new);
        cache.insert(key.to_string(), Arc::new(lut));
    }
}
