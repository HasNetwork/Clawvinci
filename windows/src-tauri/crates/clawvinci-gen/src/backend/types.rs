// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only
// Derived from Sources/PalmierPro/Generation/GenerationBackend.swift (GPLv3).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BackendGenerationStatus {
    Queued,
    Running,
    Succeeded,
    Failed,
}

impl BackendGenerationStatus {
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Succeeded | Self::Failed)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackendGenerationJob {
    pub id: String,
    pub status: BackendGenerationStatus,
    #[serde(default)]
    pub progress: f64,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub result_urls: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cost_credits: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refunded_credits: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<f64>,
}
