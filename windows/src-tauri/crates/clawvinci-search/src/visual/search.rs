// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only
// Derived from Sources/PalmierPro/Search/Query/VisualSearch.swift (GPLv3).

use crate::visual::store::AssetIndex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VisualHit {
    pub asset_id: String,
    pub time: f64,
    pub shot_start: f64,
    pub shot_end: f64,
    pub score: f32,
}

pub struct VisualSearch;

impl VisualSearch {
    /// Computes top hits across indexed assets, taking the best scoring frame per shot.
    pub fn search(
        query: &[f32],
        indexes: &[(&str, &AssetIndex)],
        limit: usize,
        relative_cutoff: f32,
        min_score: Option<f32>,
    ) -> Vec<VisualHit> {
        let mut hits: Vec<VisualHit> = Vec::new();

        for &(asset_id, index) in indexes {
            let dim = index.header.dim;
            if dim != query.len() || index.header.count == 0 {
                continue;
            }

            // Map: shot_start (represented as integer millisecond key) -> (row_idx, score)
            let mut best_per_shot: HashMap<i64, (usize, f32)> = HashMap::new();

            for (i, row) in index.rows.iter().enumerate() {
                let offset = i * dim;
                let mut score = 0.0f32;
                for (d, &q) in query.iter().enumerate() {
                    score += index.vectors[offset + d] * q;
                }

                let shot_key = (row.shot_start * 1000.0).round() as i64;
                if let Some(&(_, existing_score)) = best_per_shot.get(&shot_key) {
                    if existing_score >= score {
                        continue;
                    }
                }
                best_per_shot.insert(shot_key, (i, score));
            }

            for (_, (best_row, score)) in best_per_shot {
                let row = &index.rows[best_row];
                hits.push(VisualHit {
                    asset_id: asset_id.to_string(),
                    time: row.time,
                    shot_start: row.shot_start,
                    shot_end: row.shot_end,
                    score,
                });
            }
        }

        // Sort descending by score
        hits.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        if let Some(min_s) = min_score {
            hits.retain(|h| h.score >= min_s);
        }

        if let Some(top) = hits.first() {
            if top.score > 0.0 {
                let floor = top.score * relative_cutoff;
                hits.retain(|h| h.score >= floor);
            } else {
                return Vec::new();
            }
        } else {
            return Vec::new();
        }

        if hits.len() > limit {
            hits.truncate(limit);
        }

        hits
    }
}
