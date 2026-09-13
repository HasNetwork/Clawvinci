// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only
//! Clawvinci Search & ML Engine (Phase 10 & 12): Transcript indexing, BYOK & Local Whisper transcription, SigLIP2 embeddings, semantic search.

pub mod coordinator;
pub mod error;
pub mod model_download;
pub mod transcription;
pub mod visual;

pub use coordinator::{
    PreflightResult, SearchAssetInfo, SearchIndexCoordinator, SearchMediaType,
};
pub use error::{SearchError, SearchResult};
#[cfg(any(test, feature = "test-mocks"))]
pub use transcription::DeterministicLocalTranscriber;
pub use transcription::{
    BackendTranscriptionJob, BackendTranscriptionStatus, BackendTranscriptionSubmit,
    CutAggressiveness, CutWord, LocalWhisperEngine, LocalWhisperTranscriber, TranscriptCache,
    TranscriptHit, TranscriptSearch, TranscriptionBackend, TranscriptionBackendConfig,
    TranscriptionEngineMode, TranscriptionProvider, TranscriptionResult, TranscriptionSegment,
    TranscriptionService, TranscriptionWord, WhisperAudioPreprocessor, WhisperModelFileSpec,
    WhisperModelManifest, WhisperModelSpec, WhisperRsTranscriber, WordCutPlanner,
    WHISPER_HOP_LENGTH, WHISPER_N_FFT, WHISPER_N_MELS, WHISPER_SAMPLE_RATE,
};
pub use visual::{
    AssetIndex, EmbeddingHeader, EmbeddingRow, EmbeddingStore, FrameSamplerOptions,
    FrameSamplerState, LoaderState, LumaGrid, ModelFileSpec, ModelManifest,
    ModelSpec, OnnxVisualEmbedder, SampleDecision, VisualEmbedder, VisualHit, VisualIndexer,
    VisualModelLoader, VisualSearch, EMBEDDING_STORE_MAGIC, LUMA_GRID_CELLS, LUMA_GRID_SIZE,
    SAMPLER_VERSION,
};
