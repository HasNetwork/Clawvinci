// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only
// Derived from Sources/PalmierPro/Transcription/TranscriptSearch.swift (GPLv3).

use crate::transcription::result::TranscriptionResult;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TranscriptHit {
    pub asset_id: String,
    pub start: f64,
    pub end: f64,
    pub text: String,
}

pub struct TranscriptSearch;

impl TranscriptSearch {
    /// Tokenizes query into words, stripping edge punctuation (e.g. "budget," -> "budget").
    pub fn terms(query: &str) -> Vec<String> {
        query
            .split_whitespace()
            .map(|w| {
                w.trim_matches(|c: char| c.is_ascii_punctuation() || c.is_whitespace())
                    .to_lowercase()
            })
            .filter(|w| !w.is_empty())
            .collect()
    }

    /// Checks if `text` contains all search `terms` case-insensitively.
    pub fn matches(text: &str, terms: &[String]) -> bool {
        if terms.is_empty() {
            return false;
        }
        let lower = text.to_lowercase();
        terms.iter().all(|term| lower.contains(term))
    }

    /// Searches across a list of asset transcription results, returning up to `limit` hits.
    pub fn search(
        query: &str,
        assets: &[(String, TranscriptionResult)],
        limit: usize,
    ) -> Vec<TranscriptHit> {
        let query_terms = Self::terms(query);
        if query_terms.is_empty() {
            return Vec::new();
        }

        let mut hits = Vec::new();
        for (asset_id, transcript) in assets {
            for segment in &transcript.segments {
                if Self::matches(&segment.text, &query_terms) {
                    hits.push(TranscriptHit {
                        asset_id: asset_id.clone(),
                        start: segment.start,
                        end: segment.end,
                        text: segment.text.clone(),
                    });
                    if hits.len() >= limit {
                        return hits;
                    }
                }
            }
        }
        hits
    }
}
