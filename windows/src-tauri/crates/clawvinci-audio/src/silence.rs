// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only
// Derived from Sources/PalmierPro/Audio/Analysis/SilenceRemovalSettings.swift (GPLv3).

use serde::{Deserialize, Serialize};
use std::ops::Range;

/// Configuration parameters for silence and dead-air detection.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SilenceRemovalSettings {
    pub minimum_pause_seconds: f64,
    pub speech_padding_seconds: f64,
}

impl Default for SilenceRemovalSettings {
    fn default() -> Self {
        Self {
            minimum_pause_seconds: 0.5,
            speech_padding_seconds: 0.15,
        }
    }
}

impl SilenceRemovalSettings {
    pub const MIN_PAUSE_MIN: f64 = 0.25;
    pub const MIN_PAUSE_MAX: f64 = 3.0;
    pub const PADDING_MIN: f64 = 0.0;
    pub const PADDING_MAX: f64 = 0.5;

    pub fn new(minimum_pause_seconds: f64, speech_padding_seconds: f64) -> Option<Self> {
        if !minimum_pause_seconds.is_finite()
            || !speech_padding_seconds.is_finite()
            || minimum_pause_seconds < Self::MIN_PAUSE_MIN
            || minimum_pause_seconds > Self::MIN_PAUSE_MAX
            || speech_padding_seconds < Self::PADDING_MIN
            || speech_padding_seconds > Self::PADDING_MAX
        {
            return None;
        }

        Some(Self {
            minimum_pause_seconds,
            speech_padding_seconds,
        })
    }
}

/// Planner for converting quiet non-speech masks into cuttable timeline intervals.
pub struct SilenceRemovalPlanner;

impl SilenceRemovalPlanner {
    pub const DEFAULT_CELL_DURATION: f64 = 0.032; // 32 ms (512 / 16,000)

    /// Computes a boolean mask of removable dead-air cells from a raw quiet non-speech mask.
    pub fn removable_mask(
        quiet_non_speech_mask: &[bool],
        settings: &SilenceRemovalSettings,
        cell_duration: f64,
    ) -> Vec<bool> {
        if quiet_non_speech_mask.is_empty() || !cell_duration.is_finite() || cell_duration <= 0.0 {
            return Vec::new();
        }

        let minimum_cells = cell_count(
            settings.minimum_pause_seconds,
            cell_duration,
            quiet_non_speech_mask.len() + 1,
        );
        let padding_cells = cell_count(
            settings.speech_padding_seconds,
            cell_duration,
            quiet_non_speech_mask.len(),
        );

        let mut removable = vec![false; quiet_non_speech_mask.len()];
        let mut i = 0;

        while i < quiet_non_speech_mask.len() {
            if !quiet_non_speech_mask[i] {
                i += 1;
                continue;
            }

            let mut j = i + 1;
            while j < quiet_non_speech_mask.len() && quiet_non_speech_mask[j] {
                j += 1;
            }

            if j - i >= minimum_cells {
                let start = i + if i > 0 { padding_cells } else { 0 };
                let end = j - if j < quiet_non_speech_mask.len() { padding_cells } else { 0 };
                if start < end {
                    for cell in start..end {
                        removable[cell] = true;
                    }
                }
            }

            i = j;
        }

        removable
    }

    /// Clip-visible removable ranges in source-frame coordinates.
    pub fn visible_removable_ranges(
        removable_mask: &[bool],
        visible_source_range: Range<f64>,
        frames_per_second: u32,
        settings: &SilenceRemovalSettings,
        cell_duration: f64,
    ) -> Vec<Range<f64>> {
        let cell_frames = cell_duration * frames_per_second.max(1) as f64;
        if !cell_frames.is_finite()
            || cell_frames <= 0.0
            || !visible_source_range.start.is_finite()
            || !visible_source_range.end.is_finite()
        {
            return Vec::new();
        }

        let visible_cells = (visible_source_range.start / cell_frames)
            ..(visible_source_range.end / cell_frames);

        let cell_ranges = Self::visible_removable_cell_ranges(
            removable_mask,
            visible_cells,
            settings,
            cell_duration,
        );

        cell_ranges
            .into_iter()
            .map(|r| (r.start * cell_frames)..(r.end * cell_frames))
            .collect()
    }

    fn visible_removable_cell_ranges(
        removable_mask: &[bool],
        visible_range: Range<f64>,
        settings: &SilenceRemovalSettings,
        cell_duration: f64,
    ) -> Vec<Range<f64>> {
        if removable_mask.is_empty()
            || visible_range.is_empty()
            || !visible_range.start.is_finite()
            || !visible_range.end.is_finite()
            || !cell_duration.is_finite()
            || cell_duration <= 0.0
        {
            return Vec::new();
        }

        let edge_padding = cell_count(
            settings.speech_padding_seconds,
            cell_duration,
            removable_mask.len(),
        ) as f64;

        let scan_start = ((visible_range.start - edge_padding).floor() as usize).min(removable_mask.len());
        let scan_end = (((visible_range.end + edge_padding).ceil() as usize).max(scan_start)).min(removable_mask.len());

        let mut ranges = Vec::new();
        let mut i = scan_start;

        while i < scan_end {
            if !removable_mask[i] {
                i += 1;
                continue;
            }

            let mut j = i + 1;
            while j < scan_end && removable_mask[j] {
                j += 1;
            }

            let source_range = (i as f64)..(j as f64);
            let expanded = expanding_to_visible_edges(source_range, &visible_range, edge_padding);

            if expanded.start < visible_range.end && expanded.end > visible_range.start {
                let start = expanded.start.max(visible_range.start);
                let end = expanded.end.min(visible_range.end);
                if start < end {
                    ranges.push(start..end);
                }
            }

            i = j;
        }

        ranges
    }
}

fn expanding_to_visible_edges(
    removable_range: Range<f64>,
    visible_range: &Range<f64>,
    edge_padding: f64,
) -> Range<f64> {
    if edge_padding < 0.0 {
        return removable_range;
    }

    let start = if removable_range.end > visible_range.start
        && removable_range.start <= visible_range.start + edge_padding
    {
        visible_range.start
    } else {
        removable_range.start
    };

    let end = if removable_range.start < visible_range.end
        && removable_range.end >= visible_range.end - edge_padding
    {
        visible_range.end
    } else {
        removable_range.end
    };

    start..end
}

fn cell_count(seconds: f64, cell_duration: f64, maximum: usize) -> usize {
    let count = (seconds / cell_duration).ceil();
    if !count.is_finite() || count >= maximum as f64 {
        return maximum;
    }
    (count.max(0.0) as usize).min(maximum)
}
