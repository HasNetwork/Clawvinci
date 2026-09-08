// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only
//! Clawvinci Audio Analysis Engine (Phase 9): Beat detection, silence detection, metering,
//! multi-track audio sync correlation, and voice activity detection.

pub mod beats;
pub mod envelope;
pub mod error;
pub mod meter;
pub mod reader;
pub mod silence;
pub mod speaker;
pub mod sync;
pub mod vad;

pub use beats::{estimate_bpm, pick_peaks, BeatAnalysis, BeatDetector, BeatStore};
pub use envelope::{compute_envelope, AudioEnvelope, DEFAULT_HOP_SECONDS, DEFAULT_SAMPLE_RATE};
pub use error::{AudioError, AudioResult};
pub use meter::{
    AudioLevelAnalyzer, AudioMeterAnalysis, AudioMeterChannelDisplay, AudioMeterChannelState,
    AudioMeterHub, StereoAudioMeterDisplay,
};
pub use reader::AudioTrackReader;
pub use silence::{SilenceRemovalPlanner, SilenceRemovalSettings};
pub use speaker::{
    cosine_similarity, mean_vector, normalize_vector, turns_from_words, SpeakerRegistry,
    SpeakerTurn, TranscriptWordInfo,
};
pub use sync::{AudioSyncCorrelator, AudioSyncResult};
pub use vad::{
    SpeechMaskStore, VadAnalysis, VadSpan, VoiceActivityDetector, VAD_CHUNK_DURATION,
    VAD_CHUNK_SIZE, VAD_SAMPLE_RATE,
};
