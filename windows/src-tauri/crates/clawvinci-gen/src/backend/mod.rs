// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only

pub mod client;
pub mod types;

#[cfg(any(test, feature = "test-mocks"))]
pub use client::MockGenerationBackend;
pub use client::{
    ByokGenerationBackend, ByokProviderConfig, GenerationBackendClient, HttpGenerationBackend,
};
pub use types::{BackendGenerationJob, BackendGenerationStatus};
