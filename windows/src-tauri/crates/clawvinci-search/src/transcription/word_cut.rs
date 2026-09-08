// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only
// Derived from Sources/PalmierPro/Transcription/WordCutPlanner.swift (GPLv3).

use clawvinci_timeline::ripple::FrameRange;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CutWord {
    pub start_frame: i64,
    pub end_frame: i64,
    pub selected: bool,
}

impl CutWord {
    pub fn new(start_frame: i64, end_frame: i64, selected: bool) -> Self {
        Self {
            start_frame,
            end_frame,
            selected,
        }
    }
}

pub struct WordCutPlanner;

impl WordCutPlanner {
    /// Computes cut frame ranges to delete selected words while preserving `keep_gap_frames` padding.
    pub fn cut_ranges(
        words: &[CutWord],
        clip_start: i64,
        clip_end: i64,
        keep_gap_frames: i64,
    ) -> Vec<FrameRange> {
        let valid_words: Vec<&CutWord> = words
            .iter()
            .filter(|w| w.end_frame > w.start_frame)
            .collect();

        if clip_end <= clip_start || valid_words.is_empty() {
            return Vec::new();
        }

        let half = (keep_gap_frames / 2).max(0);
        let mut ranges: Vec<FrameRange> = Vec::new();
        let mut k = 0;

        while k < valid_words.len() {
            if !valid_words[k].selected {
                k += 1;
                continue;
            }

            let mut l = k;
            while l + 1 < valid_words.len() && valid_words[l + 1].selected {
                l += 1;
            }

            let left = if k > 0 {
                valid_words[k - 1].end_frame
            } else {
                clip_start
            };

            let right = if l + 1 < valid_words.len() {
                valid_words[l + 1].start_frame
            } else {
                clip_end
            };

            let run_start = valid_words[k].start_frame;
            let run_end = valid_words[l].end_frame;

            let keep_before = (run_start - left).max(0).min(half);
            let keep_after = (right - run_end).max(0).min(half);

            let start = clip_start.max((left + keep_before).min(run_start));
            let end = clip_end.min(run_end.max(right - keep_after));

            if end > start {
                ranges.push(FrameRange::new(start, end));
            }

            k = l + 1;
        }

        Self::merge_ranges(ranges)
    }

    /// Merges overlapping and adjacent frame intervals.
    pub fn merge_ranges(mut ranges: Vec<FrameRange>) -> Vec<FrameRange> {
        if ranges.is_empty() {
            return ranges;
        }

        ranges.sort_by_key(|r| r.start);
        let mut merged: Vec<FrameRange> = Vec::with_capacity(ranges.len());

        for r in ranges {
            if let Some(last) = merged.last_mut() {
                if r.start <= last.end {
                    last.end = last.end.max(r.end);
                } else {
                    merged.push(r);
                }
            } else {
                merged.push(r);
            }
        }

        merged
    }
}
