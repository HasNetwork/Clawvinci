// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only
// Derived from Sources/PalmierPro/Audio/AudioMeter.swift (GPLv3).

use serde::{Deserialize, Serialize};
use std::ops::Range;

/// Peak audio levels across stereo channels.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioMeterAnalysis {
    pub left_peak: f32,
    pub right_peak: f32,
}

impl AudioMeterAnalysis {
    pub const SILENCE: Self = Self {
        left_peak: 0.0,
        right_peak: 0.0,
    };

    pub fn new(left_peak: f32, right_peak: f32) -> Self {
        Self {
            left_peak,
            right_peak,
        }
    }
}

/// Instantaneous channel meter display values for UI rendering.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioMeterChannelDisplay {
    pub level_db: f32,
    pub peak_db: f32,
    pub clipped: bool,
}

/// Combined stereo meter display values.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StereoAudioMeterDisplay {
    pub left: AudioMeterChannelDisplay,
    pub right: AudioMeterChannelDisplay,
}

/// Stateful decay model for an audio meter channel.
#[derive(Debug, Clone)]
pub struct AudioMeterChannelState {
    pub floor_db: f32,
    pub ceiling_db: f32,
    pub level_decay_db_per_second: f32,
    pub peak_decay_db_per_second: f32,
    pub peak_hold_seconds: f64,

    level_db: f32,
    level_time: f64,
    peak_db: f32,
    peak_hold_until: f64,
    clipped: bool,
}

impl Default for AudioMeterChannelState {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioMeterChannelState {
    pub const DEFAULT_FLOOR_DB: f32 = -60.0;
    pub const DEFAULT_CEILING_DB: f32 = 0.0;
    pub const DEFAULT_LEVEL_DECAY_DB_PER_SEC: f32 = 24.0;
    pub const DEFAULT_PEAK_DECAY_DB_PER_SEC: f32 = 18.0;
    pub const DEFAULT_PEAK_HOLD_SECONDS: f64 = 1.5;

    pub fn new() -> Self {
        Self {
            floor_db: Self::DEFAULT_FLOOR_DB,
            ceiling_db: Self::DEFAULT_CEILING_DB,
            level_decay_db_per_second: Self::DEFAULT_LEVEL_DECAY_DB_PER_SEC,
            peak_decay_db_per_second: Self::DEFAULT_PEAK_DECAY_DB_PER_SEC,
            peak_hold_seconds: Self::DEFAULT_PEAK_HOLD_SECONDS,
            level_db: Self::DEFAULT_FLOOR_DB,
            level_time: 0.0,
            peak_db: Self::DEFAULT_FLOOR_DB,
            peak_hold_until: 0.0,
            clipped: false,
        }
    }

    pub fn decibels(&self, amplitude: f32) -> f32 {
        if amplitude > 0.0 {
            (20.0 * amplitude.log10()).max(self.floor_db)
        } else {
            self.floor_db
        }
    }

    pub fn ingest(&mut self, peak: f32, at_time: f64) {
        let current = self.display(at_time);
        let incoming_peak = self.decibels(peak);

        self.level_db = incoming_peak.max(current.level_db);
        self.level_time = at_time;

        if incoming_peak >= current.peak_db {
            self.peak_db = incoming_peak;
            self.peak_hold_until = at_time + self.peak_hold_seconds;
        } else if at_time > self.peak_hold_until {
            self.peak_db = current.peak_db;
            self.peak_hold_until = at_time;
        }

        self.clipped = self.clipped || peak >= 1.0;
    }

    pub fn display(&self, at_time: f64) -> AudioMeterChannelDisplay {
        let level_elapsed = (at_time - self.level_time).max(0.0) as f32;
        let peak_elapsed = (at_time - self.peak_hold_until).max(0.0) as f32;

        AudioMeterChannelDisplay {
            level_db: (self.level_db - level_elapsed * self.level_decay_db_per_second)
                .max(self.floor_db),
            peak_db: (self.peak_db - peak_elapsed * self.peak_decay_db_per_second)
                .max(self.floor_db),
            clipped: self.clipped,
        }
    }

    pub fn reset_clipping(&mut self) {
        self.clipped = false;
    }

    pub fn reset(&mut self) {
        self.level_db = self.floor_db;
        self.level_time = 0.0;
        self.peak_db = self.floor_db;
        self.peak_hold_until = 0.0;
        self.clipped = false;
    }
}

/// Stereo audio meter hub coordinating left and right channels.
#[derive(Debug, Clone, Default)]
pub struct AudioMeterHub {
    pub left: AudioMeterChannelState,
    pub right: AudioMeterChannelState,
}

impl AudioMeterHub {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn ingest(&mut self, analysis: &AudioMeterAnalysis, at_time: f64) {
        self.left.ingest(analysis.left_peak, at_time);
        self.right.ingest(analysis.right_peak, at_time);
    }

    pub fn display(&self, at_time: f64) -> StereoAudioMeterDisplay {
        StereoAudioMeterDisplay {
            left: self.left.display(at_time),
            right: self.right.display(at_time),
        }
    }

    pub fn reset_clipping(&mut self) {
        self.left.reset_clipping();
        self.right.reset_clipping();
    }

    pub fn reset(&mut self) {
        self.left.reset();
        self.right.reset();
    }
}

/// Analyzers for measuring peak levels from audio buffers.
pub struct AudioLevelAnalyzer;

impl AudioLevelAnalyzer {
    pub fn analyze_floats(
        left: &[f32],
        right: &[f32],
        range: Range<usize>,
    ) -> AudioMeterAnalysis {
        let upper = range.end.min(left.len().min(right.len()));
        let lower = range.start.min(upper);
        if lower >= upper {
            return AudioMeterAnalysis::SILENCE;
        }

        let left_slice = &left[lower..upper];
        let right_slice = &right[lower..upper];

        AudioMeterAnalysis {
            left_peak: peak_of_floats(left_slice),
            right_peak: peak_of_floats(right_slice),
        }
    }

    pub fn analyze_i16(
        left: &[i16],
        right: &[i16],
        range: Range<usize>,
    ) -> AudioMeterAnalysis {
        let upper = range.end.min(left.len().min(right.len()));
        let lower = range.start.min(upper);
        if lower >= upper {
            return AudioMeterAnalysis::SILENCE;
        }

        AudioMeterAnalysis {
            left_peak: peak_of_i16(&left[lower..upper]),
            right_peak: peak_of_i16(&right[lower..upper]),
        }
    }

    pub fn analyze_interleaved(samples: &[f32], channels: usize) -> AudioMeterAnalysis {
        if samples.is_empty() || channels == 0 {
            return AudioMeterAnalysis::SILENCE;
        }

        let mut left_peak = 0.0f32;
        let mut right_peak = 0.0f32;

        if channels == 1 {
            left_peak = peak_of_floats(samples);
            right_peak = left_peak;
        } else {
            for (idx, &s) in samples.iter().enumerate() {
                let abs = s.abs();
                if idx % channels == 0 {
                    if abs > left_peak {
                        left_peak = abs;
                    }
                } else if idx % channels == 1 && abs > right_peak {
                    right_peak = abs;
                }
            }
        }

        AudioMeterAnalysis {
            left_peak,
            right_peak,
        }
    }
}

#[inline]
fn peak_of_floats(samples: &[f32]) -> f32 {
    let mut max_val = 0.0f32;
    for &s in samples {
        let abs = s.abs();
        if abs > max_val {
            max_val = abs;
        }
    }
    max_val
}

#[inline]
fn peak_of_i16(samples: &[i16]) -> f32 {
    let mut max_val = 0i16;
    for &s in samples {
        let abs = if s == i16::MIN { i16::MAX } else { s.abs() };
        if abs > max_val {
            max_val = abs;
        }
    }
    max_val as f32 / 32767.0
}
