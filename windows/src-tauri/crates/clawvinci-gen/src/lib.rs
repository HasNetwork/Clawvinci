// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only
//! Clawvinci Generative AI Engine (Phase 11 & 12): BYOK Direct Provider clients, submissions, and catalog.

pub mod backend;
pub mod catalog;
pub mod edit;
pub mod error;
pub mod preprocessing;
pub mod service;
pub mod submission;

pub use backend::{
    BackendGenerationJob, BackendGenerationStatus, ByokGenerationBackend, ByokProviderConfig,
    GenerationBackendClient, HttpGenerationBackend, MockGenerationBackend,
};
pub use catalog::{
    AudioCaps, AudioPricing, CostEstimator, ImageCaps, ModelCapabilities, ModelCatalog,
    ModelCatalogEntry, ModelModality, ModelPreferences, ModelPricing, UpscaleCaps, VideoCaps,
};
pub use edit::{EditActionAvailability, EditActionKind};
pub use error::{GenError, GenResult};
pub use preprocessing::{
    AudioTrackExtractor, ImageConverter, TrimmedSource, VideoTrimExtractor,
};
pub use service::GenerationService;
pub use submission::{
    AudioGenerationParams, AudioGenerationSubmission, BackendGenerationParams,
    ImageGenerationParams, ImageGenerationSubmission, MusicGenerationSubmission, MusicMode,
    UpscaleGenerationParams, VideoGenerationParams, VideoGenerationSubmission,
};
