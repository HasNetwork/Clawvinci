// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only
//! Clawvinci Search & ML Engine (Phase 10): Transcript indexing, SigLIP2 embeddings, semantic search.

pub mod coordinator;
pub mod error;
pub mod transcription;
pub mod visual;

pub use coordinator::{
    PreflightResult, SearchAssetInfo, SearchIndexCoordinator, SearchMediaType,
};
pub use error::{SearchError, SearchResult};
pub use transcription::{
    BackendTranscriptionJob, BackendTranscriptionStatus, BackendTranscriptionSubmit,
    CutAggressiveness, CutWord, TranscriptCache, TranscriptHit, TranscriptSearch,
    TranscriptionBackend, TranscriptionBackendConfig, TranscriptionProvider, TranscriptionResult,
    TranscriptionSegment, TranscriptionWord, WordCutPlanner,
};
pub use visual::{
    AssetIndex, EmbeddingHeader, EmbeddingRow, EmbeddingStore, FrameSamplerOptions,
    FrameSamplerState, LoaderState, LumaGrid, MockVisualEmbedder, ModelFileSpec, ModelManifest,
    ModelSpec, SampleDecision, VisualEmbedder, VisualHit, VisualIndexer, VisualModelLoader,
    VisualSearch, EMBEDDING_STORE_MAGIC, LUMA_GRID_CELLS, LUMA_GRID_SIZE, SAMPLER_VERSION,
};
