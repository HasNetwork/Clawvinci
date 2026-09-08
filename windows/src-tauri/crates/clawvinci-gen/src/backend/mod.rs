// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only

pub mod client;
pub mod types;

pub use client::{GenerationBackendClient, HttpGenerationBackend, MockGenerationBackend};
pub use types::{BackendGenerationJob, BackendGenerationStatus};
