// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only
// Derived from Sources/PalmierPro/Models/MulticamSource.swift (GPLv3).

use serde::{Deserialize, Serialize};
use uuid::Uuid;

fn default_uuid() -> String {
    Uuid::new_v4().to_string()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MemberKind {
    Angle,
    Mic,
    Both,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SyncMap {
    #[serde(default)]
    pub offset_seconds: f64,
    #[serde(default)]
    pub confidence: f64,
    #[serde(default)]
    pub locked: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MulticamMember {
    #[serde(default = "default_uuid")]
    pub id: String,
    pub media_ref: String,
    pub kind: MemberKind,
    pub angle_label: String,
    #[serde(default)]
    pub sync: SyncMap,
}

impl MulticamMember {
    pub fn new(
        media_ref: impl Into<String>,
        kind: MemberKind,
        angle_label: impl Into<String>,
    ) -> Self {
        Self {
            id: default_uuid(),
            media_ref: media_ref.into(),
            kind,
            angle_label: angle_label.into(),
            sync: SyncMap::default(),
        }
    }

    pub fn provides_video(&self) -> bool {
        self.kind != MemberKind::Mic
    }

    pub fn provides_audio(&self) -> bool {
        self.kind != MemberKind::Angle
    }

    pub fn is_usable(&self) -> bool {
        self.sync.confidence > 0.0 || self.sync.locked
    }

    pub fn offset_frames(&self, fps: i32) -> i64 {
        (self.sync.offset_seconds * fps as f64).round() as i64
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MulticamSource {
    #[serde(default = "default_uuid")]
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub members: Vec<MulticamMember>,
    #[serde(default)]
    pub master_member_id: String,
}

impl MulticamSource {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: default_uuid(),
            name: name.into(),
            members: Vec::new(),
            master_member_id: String::new(),
        }
    }

    pub fn master(&self) -> Option<&MulticamMember> {
        self.members.iter().find(|m| m.id == self.master_member_id)
    }

    pub fn angles(&self) -> Vec<&MulticamMember> {
        self.members
            .iter()
            .filter(|m| m.provides_video() && m.is_usable())
            .collect()
    }

    pub fn mics(&self) -> Vec<&MulticamMember> {
        self.members
            .iter()
            .filter(|m| m.provides_audio() && m.is_usable())
            .collect()
    }
}
