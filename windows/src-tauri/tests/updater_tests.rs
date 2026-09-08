// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PlatformUpdateInfo {
    pub signature: String,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TauriUpdateManifest {
    pub version: String,
    pub notes: Option<String>,
    pub pub_date: Option<String>,
    pub platforms: HashMap<String, PlatformUpdateInfo>,
}

impl TauriUpdateManifest {
    pub fn is_newer_than(&self, current_version: &str) -> bool {
        let parse_version = |v: &str| -> Vec<u64> {
            v.trim_start_matches('v')
                .split('.')
                .filter_map(|part| part.parse::<u64>().ok())
                .collect()
        };

        let remote = parse_version(&self.version);
        let current = parse_version(current_version);

        for (r, c) in remote.iter().zip(current.iter()) {
            if r > c {
                return true;
            } else if r < c {
                return false;
            }
        }
        remote.len() > current.len()
    }
}

#[test]
fn test_tauri_update_manifest_parsing_and_version_check() {
    let manifest_json = r#"{
        "version": "v0.2.0",
        "notes": "Clawvinci Phase 12 BYOK updates and engine optimizations",
        "pub_date": "2026-09-08T12:00:00Z",
        "platforms": {
            "windows-x86_64": {
                "signature": "dW50cnVzdGVkIGNvbW1lbnQ6IHNpZ25hdHVyZQpS...",
                "url": "https://github.com/HasNetwork/Clawvinci/releases/download/v0.2.0/Clawvinci_0.2.0_x64-setup.exe"
            }
        }
    }"#;

    let manifest: TauriUpdateManifest = serde_json::from_str(manifest_json).unwrap();
    assert_eq!(manifest.version, "v0.2.0");
    assert!(manifest.is_newer_than("0.1.0"));
    assert!(!manifest.is_newer_than("0.2.0"));
    assert!(!manifest.is_newer_than("0.3.0"));

    let win_platform = manifest.platforms.get("windows-x86_64").expect("windows-x86_64 platform exists");
    assert!(win_platform.url.ends_with(".exe"));
    assert!(!win_platform.signature.is_empty());
}
