// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only
// Derived from Sources/PalmierPro/Audio/Analysis/SpeakerIdentity.swift (GPLv3).

use serde::{Deserialize, Serialize};

/// A continuous speech turn by a single speaker.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpeakerTurn {
    pub speaker: String,
    pub start: f64,
    pub end: f64,
}

impl SpeakerTurn {
    pub fn new(speaker: impl Into<String>, start: f64, end: f64) -> Self {
        Self {
            speaker: speaker.into(),
            start,
            end,
        }
    }

    pub fn duration(&self) -> f64 {
        (self.end - self.start).max(0.0)
    }
}

/// Word-level timing metadata from a transcript.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TranscriptWordInfo {
    pub text: String,
    pub speaker: Option<String>,
    pub start: Option<f64>,
    pub end: Option<f64>,
}

/// Merges consecutive same-speaker words (with gaps under 1.0s) into continuous conversational turns.
pub fn turns_from_words(words: &[TranscriptWordInfo]) -> Vec<SpeakerTurn> {
    let mut turns: Vec<SpeakerTurn> = Vec::new();

    for word in words {
        let (speaker, start, end) = match (&word.speaker, word.start, word.end) {
            (Some(s), Some(st), Some(en)) if en > st => (s, st, en),
            _ => continue,
        };

        if let Some(last) = turns.last_mut() {
            if &last.speaker == speaker && (start - last.end) < 1.0 {
                last.end = end;
                continue;
            }
        }

        turns.push(SpeakerTurn::new(speaker, start, end));
    }

    turns
}

/// Computes the cosine similarity between two voice embedding vectors.
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.is_empty() || a.len() != b.len() {
        return 0.0;
    }

    let mut dot = 0.0f32;
    let mut na = 0.0f32;
    let mut nb = 0.0f32;

    for (&x, &y) in a.iter().zip(b.iter()) {
        dot += x * y;
        na += x * x;
        nb += y * y;
    }

    let denom = (na * nb).sqrt();
    if denom > 0.0 {
        (dot / denom).max(0.0)
    } else {
        0.0
    }
}

/// Normalizes a vector to unit length (L2 norm).
pub fn normalize_vector(v: &[f32]) -> Vec<f32> {
    let norm = v.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm > 0.0 {
        v.iter().map(|x| x / norm).collect()
    } else {
        v.to_vec()
    }
}

/// Computes the element-wise mean of multiple embedding vectors.
pub fn mean_vector(vectors: &[Vec<f32>]) -> Vec<f32> {
    if vectors.is_empty() {
        return Vec::new();
    }
    let dim = vectors[0].len();
    let mut out = vec![0.0f32; dim];

    for vec in vectors {
        if vec.len() == dim {
            for (out_val, &val) in out.iter_mut().zip(vec.iter()) {
                *out_val += val;
            }
        }
    }

    let count = vectors.len() as f32;
    out.iter_mut().for_each(|x| *x /= count);
    out
}

/// Registry mapping speaker voice embeddings to global speaker IDs across files.
#[derive(Debug, Clone, Default)]
pub struct SpeakerRegistry {
    pub similarity_floor: f32,
    clusters: Vec<(usize, Vec<f32>)>,
    next_id: usize,
}

impl SpeakerRegistry {
    pub const DEFAULT_SIMILARITY_FLOOR: f32 = 0.45;

    pub fn new() -> Self {
        Self {
            similarity_floor: Self::DEFAULT_SIMILARITY_FLOOR,
            clusters: Vec::new(),
            next_id: 1,
        }
    }

    /// Assigns or matches an embedding vector to a global speaker ID.
    pub fn assign_or_create(&mut self, embedding: &[f32]) -> usize {
        let norm_vec = normalize_vector(embedding);
        let mut best_id = None;
        let mut best_sim = self.similarity_floor;

        for (id, centroid) in &self.clusters {
            let sim = cosine_similarity(&norm_vec, centroid);
            if sim >= best_sim {
                best_sim = sim;
                best_id = Some(*id);
            }
        }

        if let Some(id) = best_id {
            id
        } else {
            let new_id = self.next_id;
            self.next_id += 1;
            self.clusters.push((new_id, norm_vec));
            new_id
        }
    }

    pub fn len(&self) -> usize {
        self.clusters.len()
    }

    pub fn is_empty(&self) -> bool {
        self.clusters.is_empty()
    }
}
