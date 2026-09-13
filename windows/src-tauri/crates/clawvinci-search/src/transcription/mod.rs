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
#[cfg(any(test, feature = "test-mocks"))]
pub use local::DeterministicLocalTranscriber;
pub use local::{
    LocalWhisperEngine, LocalWhisperTranscriber, WhisperAudioPreprocessor, WhisperModelFileSpec,
    WhisperModelManifest, WhisperModelSpec, WhisperRsTranscriber, WHISPER_HOP_LENGTH,
    WHISPER_N_FFT, WHISPER_N_MELS, WHISPER_SAMPLE_RATE,
};
pub use result::{
    CutAggressiveness, TranscriptionProvider, TranscriptionResult, TranscriptionSegment,
    TranscriptionWord,
};
pub use search::{TranscriptHit, TranscriptSearch};
pub use service::{TranscriptionEngineMode, TranscriptionService};
pub use word_cut::{CutWord, WordCutPlanner};
