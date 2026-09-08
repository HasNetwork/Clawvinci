// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only
// Derived from Sources/PalmierPro/Search/ (GPLv3).

pub mod indexer;
pub mod loader;
pub mod model;
pub mod sampler;
pub mod search;
pub mod store;

pub use indexer::VisualIndexer;
pub use loader::{LoaderState, ModelFileSpec, ModelManifest, VisualModelLoader};
pub use model::{MockVisualEmbedder, ModelSpec, VisualEmbedder};
pub use sampler::{
    FrameSamplerOptions, FrameSamplerState, LumaGrid, SampleDecision, LUMA_GRID_CELLS,
    LUMA_GRID_SIZE, SAMPLER_VERSION,
};
pub use search::{VisualHit, VisualSearch};
pub use store::{
    AssetIndex, EmbeddingHeader, EmbeddingRow, EmbeddingStore, EMBEDDING_STORE_MAGIC,
};
