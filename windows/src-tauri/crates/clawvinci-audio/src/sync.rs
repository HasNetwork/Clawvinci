// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only
// Derived from Sources/PalmierPro/Audio/AudioSyncCorrelator.swift (GPLv3).

use serde::{Deserialize, Serialize};
use std::ops::RangeInclusive;

/// Alignment result between target and reference audio envelopes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioSyncResult {
    /// Lag in hops. Positive lag means target occurs after reference.
    pub lag_hops: i64,
    /// Cross-correlation confidence score in range 0.0..=1.0.
    pub confidence: f64,
    /// Fitted clock drift ratio (dimensionless slope, e.g. ±0.0001).
    pub drift_ratio: f64,
    /// Exact fractional lag in hops from linear regression drift fit.
    pub exact_lag_hops: f64,
}

impl AudioSyncResult {
    pub fn new(lag_hops: i64, confidence: f64) -> Self {
        Self {
            lag_hops,
            confidence,
            drift_ratio: 0.0,
            exact_lag_hops: lag_hops as f64,
        }
    }

    pub fn with_drift(lag_hops: i64, confidence: f64, drift_ratio: f64, exact_lag_hops: f64) -> Self {
        Self {
            lag_hops,
            confidence,
            drift_ratio,
            exact_lag_hops,
        }
    }
}

pub struct AudioSyncCorrelator;

impl AudioSyncCorrelator {
    pub const MIN_OVERLAP: usize = 16;
    const PYRAMID_STRIDE: usize = 2;
    const MAX_EXACT_LAG_COUNT: usize = 4_096;
    const MAX_CANDIDATES: usize = 16;

    // Consensus constants assuming 2.5 ms envelope hop (400 hops/s)
    const CONSENSUS_CHUNK_HOPS: usize = 24_000; // 60 seconds
    const CONSENSUS_CHUNK_COUNT: usize = 5;
    const CONSENSUS_TOLERANCE_HOPS: i64 = 400; // 1.0 second
    const REACQUIRE_RADIUS_HOPS: i64 = 800; // ±2.0 seconds
    const DRIFT_FIT_MIN_SPAN_HOPS: f64 = 60_000.0; // 2.5 minutes
    const MAX_DRIFT_RATIO: f64 = 0.0005; // 500 ppm
    const MIN_DRIFT_RATIO: f64 = 0.00002; // 20 ppm

    /// Primary entry point for multi-track audio sync.
    pub fn seeded_correlate(
        reference: &[f32],
        target: &[f32],
        seed_hops: Option<i64>,
        seed_window_hops: i64,
        max_lag_hops: i64,
        min_overlap_hops: usize,
        min_confidence: f64,
    ) -> Option<AudioSyncResult> {
        let mut windows = vec![(0i64, max_lag_hops)];
        if let Some(seed) = seed_hops {
            windows.push((seed, seed_window_hops));
        }

        Self::align_by_consensus(
            reference,
            target,
            &windows,
            min_overlap_hops,
            min_confidence,
        )
    }

    /// Single window correlation.
    pub fn correlate(
        reference: &[f32],
        target: &[f32],
        max_lag_hops: i64,
        center_lag_hops: i64,
        min_overlap_hops: usize,
    ) -> Option<AudioSyncResult> {
        if reference.is_empty() || target.is_empty() || max_lag_hops < 0 {
            return None;
        }

        let shorter_count = reference.len().min(target.len());
        let adaptive_overlap = Self::MIN_OVERLAP.max(shorter_count / 2);
        let required_overlap = Self::MIN_OVERLAP.max(min_overlap_hops).min(adaptive_overlap);
        if shorter_count < required_overlap {
            return None;
        }

        let valid_lower = required_overlap as i64 - target.len() as i64;
        let valid_upper = reference.len() as i64 - required_overlap as i64;

        let lower = valid_lower.max(center_lag_hops.saturating_sub(max_lag_hops));
        let upper = valid_upper.min(center_lag_hops.saturating_add(max_lag_hops));

        if lower > upper {
            return None;
        }

        Self::correlate_candidates(reference, target, &(lower..=upper), required_overlap)
            .into_iter()
            .next()
    }

    fn align_by_consensus(
        reference: &[f32],
        target: &[f32],
        windows: &[(i64, i64)],
        min_overlap_hops: usize,
        min_confidence: f64,
    ) -> Option<AudioSyncResult> {
        let best_across_windows = |chunk: &[f32], chunk_start: usize, overlap: usize| -> Option<AudioSyncResult> {
            let mut best: Option<AudioSyncResult> = None;
            for &(center, radius) in windows {
                if let Some(cand) = Self::correlate(
                    reference,
                    chunk,
                    radius,
                    center + chunk_start as i64,
                    overlap,
                ) {
                    match &best {
                        None => best = Some(cand),
                        Some(b) if cand.confidence > b.confidence => best = Some(cand),
                        _ => {}
                    }
                }
            }
            best
        };

        let chunk_hops = target.len().min(Self::CONSENSUS_CHUNK_HOPS);
        let mut starts = vec![0usize];
        if target.len() > chunk_hops && Self::CONSENSUS_CHUNK_COUNT > 1 {
            let max_start = target.len() - chunk_hops;
            let mut s_set = std::collections::BTreeSet::new();
            for i in 0..Self::CONSENSUS_CHUNK_COUNT {
                s_set.insert(max_start * i / (Self::CONSENSUS_CHUNK_COUNT - 1));
            }
            starts = s_set.into_iter().collect();
        }

        struct Hit {
            start: usize,
            center: f64,
            clip_lag: i64,
            confidence: f64,
        }

        let mut hits: Vec<Hit> = Vec::new();
        for start in starts {
            let chunk = &target[start..start + chunk_hops];
            let (min_val, max_val) = chunk.iter().fold((f32::INFINITY, f32::NEG_INFINITY), |(mn, mx), &v| {
                (mn.min(v), mx.max(v))
            });
            if min_val == max_val {
                continue;
            }

            let overlap = min_overlap_hops.max(chunk.len() / 2).min(chunk.len());
            let mut best: Option<AudioSyncResult> = None;

            if let Some(prev) = hits.last() {
                if let Some(found) = Self::correlate(
                    reference,
                    chunk,
                    Self::REACQUIRE_RADIUS_HOPS,
                    prev.clip_lag + start as i64,
                    overlap,
                ) {
                    if found.confidence >= min_confidence {
                        best = Some(found);
                    }
                }
            }

            if best.is_none() {
                best = best_across_windows(chunk, start, overlap);
            }

            if let Some(found) = best {
                if found.confidence >= min_confidence {
                    hits.push(Hit {
                        start,
                        center: start as f64 + chunk.len() as f64 / 2.0,
                        clip_lag: found.lag_hops - start as i64,
                        confidence: found.confidence,
                    });
                }
            }
        }

        if hits.is_empty() {
            let fallback = best_across_windows(target, 0, min_overlap_hops)?;
            if fallback.confidence >= min_confidence {
                return Some(fallback);
            }
            return None;
        }

        hits.sort_by_key(|h| h.clip_lag);

        // Find densest cluster within consensus tolerance
        let mut best_cluster: &[Hit] = &hits[..];
        let mut best_cluster_score = f64::NEG_INFINITY;

        for (i, hit) in hits.iter().enumerate() {
            let mut j = i;
            while j + 1 < hits.len()
                && (hits[j + 1].clip_lag - hit.clip_lag) <= Self::CONSENSUS_TOLERANCE_HOPS
            {
                j += 1;
            }
            let candidate = &hits[i..=j];
            let score: f64 = candidate.iter().map(|h| h.confidence).sum();
            if score > best_cluster_score {
                best_cluster_score = score;
                best_cluster = candidate;
            }
        }

        let cluster_conf: f64 = best_cluster.iter().map(|h| h.confidence).sum::<f64>()
            / best_cluster.len() as f64;

        let (min_center, max_center) = best_cluster.iter().fold((f64::INFINITY, f64::NEG_INFINITY), |(mn, mx), h| {
            (mn.min(h.center), mx.max(h.center))
        });
        let span = max_center - min_center;

        if best_cluster.len() >= 3 && span >= Self::DRIFT_FIT_MIN_SPAN_HOPS {
            let n = best_cluster.len() as f64;
            let mean_x = best_cluster.iter().map(|h| h.center).sum::<f64>() / n;
            let mean_y = best_cluster.iter().map(|h| h.clip_lag as f64).sum::<f64>() / n;

            let mut var_x = 0.0;
            let mut cov = 0.0;
            for h in best_cluster {
                let dx = h.center - mean_x;
                let dy = h.clip_lag as f64 - mean_y;
                var_x += dx * dx;
                cov += dx * dy;
            }

            let slope = if var_x > 0.0 { cov / var_x } else { 0.0 };
            if slope.abs() >= Self::MIN_DRIFT_RATIO && slope.abs() <= Self::MAX_DRIFT_RATIO {
                let max_residual = best_cluster
                    .iter()
                    .map(|h| ((h.clip_lag as f64) - (mean_y + slope * (h.center - mean_x))).abs())
                    .fold(0.0f64, f64::max);

                if max_residual <= Self::CONSENSUS_TOLERANCE_HOPS as f64 {
                    let lag_at_head = mean_y - slope * mean_x;
                    return Some(AudioSyncResult::with_drift(
                        lag_at_head.round() as i64,
                        cluster_conf,
                        slope,
                        lag_at_head,
                    ));
                }
            }
        }

        let anchor = best_cluster.iter().min_by_key(|h| h.start)?;
        Some(AudioSyncResult::new(anchor.clip_lag, cluster_conf))
    }

    fn correlate_candidates(
        reference: &[f32],
        target: &[f32],
        lag_range: &RangeInclusive<i64>,
        min_overlap_hops: usize,
    ) -> Vec<AudioSyncResult> {
        let lag_count = (lag_range.end() - lag_range.start() + 1).max(0) as usize;
        if lag_count <= Self::MAX_EXACT_LAG_COUNT
            || reference.len() < Self::PYRAMID_STRIDE * Self::MIN_OVERLAP
            || target.len() < Self::PYRAMID_STRIDE * Self::MIN_OVERLAP
        {
            return Self::exact_candidates(reference, target, std::slice::from_ref(lag_range), min_overlap_hops);
        }

        let coarse_target = Self::downsample(target, Self::PYRAMID_STRIDE, 0);
        let mut mapped_candidates = Vec::new();

        for phase in 0..Self::PYRAMID_STRIDE {
            let coarse_lower = ((*lag_range.start() - phase as i64) as f64
                / Self::PYRAMID_STRIDE as f64)
                .ceil() as i64;
            let coarse_upper = ((*lag_range.end() - phase as i64) as f64
                / Self::PYRAMID_STRIDE as f64)
                .floor() as i64;
            if coarse_lower > coarse_upper {
                continue;
            }

            let coarse_ref = Self::downsample(reference, Self::PYRAMID_STRIDE, phase);
            let coarse_overlap = 4.max(min_overlap_hops.div_ceil(Self::PYRAMID_STRIDE));
            let cands = Self::correlate_candidates(
                &coarse_ref,
                &coarse_target,
                &(coarse_lower..=coarse_upper),
                coarse_overlap,
            );

            for c in cands {
                mapped_candidates.push(AudioSyncResult::new(
                    c.lag_hops * Self::PYRAMID_STRIDE as i64 + phase as i64,
                    c.confidence,
                ));
            }
        }

        let coarse_candidates = Self::ranked_candidates(&mapped_candidates);
        if coarse_candidates.is_empty() {
            return Vec::new();
        }

        let radius = (Self::PYRAMID_STRIDE * 2) as i64;
        let mut ranges = Vec::new();
        for c in coarse_candidates {
            let r_start = (*lag_range.start()).max(c.lag_hops - radius);
            let r_end = (*lag_range.end()).min(c.lag_hops + radius);
            if r_start <= r_end {
                ranges.push(r_start..=r_end);
            }
        }

        let merged = Self::merge_ranges(&ranges);
        Self::exact_candidates(reference, target, &merged, min_overlap_hops)
    }

    pub fn exact_candidates(
        reference: &[f32],
        target: &[f32],
        lag_ranges: &[RangeInclusive<i64>],
        min_overlap_hops: usize,
    ) -> Vec<AudioSyncResult> {
        let mut results = Vec::new();

        for range in lag_ranges {
            for lag in *range.start()..=*range.end() {
                let i_start = 0i64.max(-lag) as usize;
                let i_end = (target.len() as i64).min(reference.len() as i64 - lag) as usize;
                if i_end <= i_start {
                    continue;
                }
                let n = i_end - i_start;
                if n < min_overlap_hops {
                    continue;
                }

                let mut sx = 0.0f64;
                let mut sy = 0.0f64;
                let mut sxx = 0.0f64;
                let mut syy = 0.0f64;
                let mut sxy = 0.0f64;

                let x_slice = &target[i_start..i_end];
                let ref_offset = (i_start as i64 + lag) as usize;
                let y_slice = &reference[ref_offset..ref_offset + n];

                for idx in 0..n {
                    let x = x_slice[idx] as f64;
                    let y = y_slice[idx] as f64;
                    sx += x;
                    sy += y;
                    sxx += x * x;
                    syy += y * y;
                    sxy += x * y;
                }

                let n_d = n as f64;
                let cov = sxy - (sx * sy / n_d);
                let vx = sxx - (sx * sx / n_d);
                let vy = syy - (sy * sy / n_d);
                let denom = (vx * vy).max(0.0).sqrt();

                if denom > 0.0 {
                    let conf = (cov / denom).clamp(0.0, 1.0);
                    results.push(AudioSyncResult::new(lag, conf));
                }
            }
        }

        // Peak selection
        let mut peaks = Vec::new();
        for (i, res) in results.iter().enumerate() {
            let left_lower = i == 0
                || results[i - 1].lag_hops != res.lag_hops - 1
                || res.confidence >= results[i - 1].confidence;
            let right_lower = i == results.len() - 1
                || results[i + 1].lag_hops != res.lag_hops + 1
                || res.confidence >= results[i + 1].confidence;
            if left_lower && right_lower {
                peaks.push(res.clone());
            }
        }

        let candidates = if peaks.is_empty() { results } else { peaks };
        Self::ranked_candidates(&candidates)
    }

    fn ranked_candidates(candidates: &[AudioSyncResult]) -> Vec<AudioSyncResult> {
        let mut sorted = candidates.to_vec();
        sorted.sort_by(|a, b| {
            b.confidence
                .partial_cmp(&a.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.lag_hops.abs().cmp(&b.lag_hops.abs()))
                .then_with(|| a.lag_hops.cmp(&b.lag_hops))
        });

        let mut selected = Vec::new();
        for cand in sorted {
            if selected.iter().all(|s: &AudioSyncResult| (s.lag_hops - cand.lag_hops).abs() > 2) {
                selected.push(cand);
                if selected.len() >= Self::MAX_CANDIDATES {
                    break;
                }
            }
        }
        selected
    }

    fn downsample(samples: &[f32], stride: usize, offset: usize) -> Vec<f32> {
        if offset >= samples.len() {
            return Vec::new();
        }
        if stride <= 1 {
            return samples[offset..].to_vec();
        }

        let mut out = Vec::with_capacity((samples.len() - offset).div_ceil(stride));
        let mut start = offset;
        while start < samples.len() {
            let end = (start + stride).min(samples.len());
            let sum: f32 = samples[start..end].iter().sum();
            out.push(sum / (end - start) as f32);
            start += stride;
        }
        out
    }

    fn merge_ranges(ranges: &[RangeInclusive<i64>]) -> Vec<RangeInclusive<i64>> {
        if ranges.is_empty() {
            return Vec::new();
        }
        let mut sorted = ranges.to_vec();
        sorted.sort_by_key(|r| *r.start());

        let mut merged = Vec::new();
        let mut current = sorted[0].clone();

        for r in &sorted[1..] {
            if *r.start() <= *current.end() + 1 {
                current = *current.start()..=(*current.end()).max(*r.end());
            } else {
                merged.push(current);
                current = r.clone();
            }
        }
        merged.push(current);
        merged
    }
}
