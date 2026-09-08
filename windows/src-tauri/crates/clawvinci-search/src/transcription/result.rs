// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only
// Derived from Sources/PalmierPro/Transcription/Transcription.swift and WordCutPlanner.swift (GPLv3).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TranscriptionProvider {
    Local,
    Cloud,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TranscriptionWord {
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub speaker: Option<String>,
}

impl TranscriptionWord {
    pub fn new(text: impl Into<String>, start: Option<f64>, end: Option<f64>) -> Self {
        Self {
            text: text.into(),
            start,
            end,
            speaker: None,
        }
    }

    pub fn with_speaker(mut self, speaker: impl Into<String>) -> Self {
        self.speaker = Some(speaker.into());
        self
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TranscriptionSegment {
    pub text: String,
    pub start: f64,
    pub end: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub speaker: Option<String>,
}

impl TranscriptionSegment {
    pub fn new(text: impl Into<String>, start: f64, end: f64) -> Self {
        Self {
            text: text.into(),
            start,
            end,
            speaker: None,
        }
    }

    pub fn with_speaker(mut self, speaker: impl Into<String>) -> Self {
        self.speaker = Some(speaker.into());
        self
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TranscriptionResult {
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    pub words: Vec<TranscriptionWord>,
    pub segments: Vec<TranscriptionSegment>,
}

impl TranscriptionResult {
    pub fn new(
        text: impl Into<String>,
        language: Option<String>,
        words: Vec<TranscriptionWord>,
        segments: Vec<TranscriptionSegment>,
    ) -> Self {
        Self {
            text: text.into(),
            language,
            words,
            segments,
        }
    }

    /// Shifts all timestamps back into source time after transcribing an extracted range.
    pub fn offsetting(&self, offset: f64) -> Self {
        if offset == 0.0 {
            return self.clone();
        }
        Self {
            text: self.text.clone(),
            language: self.language.clone(),
            words: self
                .words
                .iter()
                .map(|w| TranscriptionWord {
                    text: w.text.clone(),
                    start: w.start.map(|s| s + offset),
                    end: w.end.map(|e| e + offset),
                    speaker: w.speaker.clone(),
                })
                .collect(),
            segments: self
                .segments
                .iter()
                .map(|s| TranscriptionSegment {
                    text: s.text.clone(),
                    start: s.start + offset,
                    end: s.end + offset,
                    speaker: s.speaker.clone(),
                })
                .collect(),
        }
    }

    /// Filters segments and words to only those overlapping the requested range `[start, end]`.
    pub fn filter_range(&self, start: f64, end: f64) -> Self {
        let segments: Vec<TranscriptionSegment> = self
            .segments
            .iter()
            .filter(|s| s.end > start && s.start < end)
            .cloned()
            .collect();

        let words: Vec<TranscriptionWord> = self
            .words
            .iter()
            .filter(|w| {
                if let (Some(s), Some(e)) = (w.start, w.end) {
                    e > start && s < end
                } else {
                    false
                }
            })
            .cloned()
            .collect();

        let text = segments
            .iter()
            .map(|s| s.text.as_str())
            .collect::<Vec<_>>()
            .join(" ");

        Self {
            text,
            language: self.language.clone(),
            words,
            segments,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CutAggressiveness {
    Tight,
    Balanced,
    Loose,
}

impl CutAggressiveness {
    pub fn kept_gap_ms(&self) -> f64 {
        match self {
            Self::Tight => 60.0,
            Self::Balanced => 150.0,
            Self::Loose => 320.0,
        }
    }

    pub fn kept_gap_seconds(&self) -> f64 {
        self.kept_gap_ms() / 1000.0
    }

    pub fn kept_gap_frames(&self, fps: f64) -> i64 {
        (self.kept_gap_seconds() * fps).round() as i64
    }
}
