// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only
// Derived from Sources/PalmierPro/Transcription/ (GPLv3).

pub mod backend;
pub mod cache;
pub mod local;
pub mod result;
pub mod search;
pub mod service;
pub mod word_cut;

pub use backend::{
    BackendTranscriptionJob, BackendTranscriptionStatus, BackendTranscriptionSubmit,
    TranscriptionBackend, TranscriptionBackendConfig,
};
pub use cache::TranscriptCache;
pub use local::{
    DeterministicLocalTranscriber, LocalWhisperEngine, LocalWhisperTranscriber,
    WhisperAudioPreprocessor, WhisperModelFileSpec, WhisperModelManifest, WhisperModelSpec,
    WHISPER_HOP_LENGTH, WHISPER_N_FFT, WHISPER_N_MELS, WHISPER_SAMPLE_RATE,
};
pub use result::{
    CutAggressiveness, TranscriptionProvider, TranscriptionResult, TranscriptionSegment,
    TranscriptionWord,
};
pub use search::{TranscriptHit, TranscriptSearch};
pub use service::{TranscriptionEngineMode, TranscriptionService};
pub use word_cut::{CutWord, WordCutPlanner};
