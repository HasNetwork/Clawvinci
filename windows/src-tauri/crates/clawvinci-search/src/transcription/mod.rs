// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only
// Derived from Sources/PalmierPro/Transcription/ (GPLv3).

pub mod backend;
pub mod cache;
pub mod result;
pub mod search;
pub mod word_cut;

pub use backend::{
    BackendTranscriptionJob, BackendTranscriptionStatus, BackendTranscriptionSubmit,
    TranscriptionBackend, TranscriptionBackendConfig,
};
pub use cache::TranscriptCache;
pub use result::{
    CutAggressiveness, TranscriptionProvider, TranscriptionResult, TranscriptionSegment,
    TranscriptionWord,
};
pub use search::{TranscriptHit, TranscriptSearch};
pub use word_cut::{CutWord, WordCutPlanner};
