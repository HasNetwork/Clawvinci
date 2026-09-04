// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only
// Derived from Sources/PalmierPro/Models/ProjectFile.swift (GPLv3).

use crate::error::ModelError;
use crate::multicam::MulticamSource;
use crate::speaker::SpeakerRegistryEntry;
use crate::timeline::{Timeline, TimelineViewState};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectFile {
    pub timelines: Vec<Timeline>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active_timeline_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub open_timeline_ids: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub view_states: Option<HashMap<String, TimelineViewState>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub speakers: Option<Vec<SpeakerRegistryEntry>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub multicam_groups: Option<Vec<MulticamSource>>,
}

impl ProjectFile {
    pub fn new(timelines: Vec<Timeline>) -> Self {
        let active_timeline_id = timelines.first().map(|t| t.id.clone());
        let open_timeline_ids = active_timeline_id.clone().map(|id| vec![id]);
        Self {
            timelines,
            active_timeline_id,
            open_timeline_ids,
            view_states: None,
            speakers: None,
            multicam_groups: None,
        }
    }

    /// Decodes project JSON bytes. Falls back to legacy bare-Timeline format if needed.
    pub fn decode(bytes: &[u8]) -> Result<Self, ModelError> {
        match serde_json::from_slice::<ProjectFile>(bytes) {
            Ok(file) => {
                if file.timelines.is_empty() {
                    Err(ModelError::EmptyTimelines)
                } else {
                    Ok(file)
                }
            }
            Err(orig_err) => {
                // Legacy files are a bare Timeline; anything else returns the original error.
                if let Ok(legacy) = serde_json::from_slice::<Timeline>(bytes) {
                    let id = legacy.id.clone();
                    Ok(ProjectFile {
                        timelines: vec![legacy],
                        active_timeline_id: Some(id.clone()),
                        open_timeline_ids: Some(vec![id]),
                        view_states: None,
                        speakers: None,
                        multicam_groups: None,
                    })
                } else {
                    Err(ModelError::DecodeError(orig_err))
                }
            }
        }
    }

    pub fn encode(&self) -> Result<Vec<u8>, ModelError> {
        Ok(serde_json::to_vec(self)?)
    }

    pub fn encode_pretty(&self) -> Result<String, ModelError> {
        Ok(serde_json::to_string_pretty(self)?)
    }
}
