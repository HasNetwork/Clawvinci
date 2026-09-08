// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TrackColorsDto {
    pub video: String,
    pub audio: String,
    pub text: String,
    pub image: String,
}

impl Default for TrackColorsDto {
    fn default() -> Self {
        Self {
            video: "#1d5878".to_string(),
            audio: "#2e7765".to_string(),
            text: "#546e7a".to_string(),
            image: "#715486".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct WhisperSettingsDto {
    pub mode: String,
    pub endpoint: Option<String>,
    pub model_size: String,
    pub device: String,
}

impl Default for WhisperSettingsDto {
    fn default() -> Self {
        Self {
            mode: "auto".to_string(),
            endpoint: None,
            model_size: "base".to_string(),
            device: "cpu".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RecentProjectDto {
    pub id: String,
    pub name: String,
    pub path: String,
    pub last_opened: String,
    pub duration_seconds: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub language: String,
    pub default_frame_rate: f64,
    pub theme: String,
    pub track_colors: TrackColorsDto,
    pub byok_keys: HashMap<String, String>,
    pub whisper_config: WhisperSettingsDto,
    pub recent_projects: Vec<RecentProjectDto>,
    pub onboarding_completed: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            language: "en".to_string(),
            default_frame_rate: 30.0,
            theme: "dark".to_string(),
            track_colors: TrackColorsDto::default(),
            byok_keys: HashMap::new(),
            whisper_config: WhisperSettingsDto::default(),
            recent_projects: vec![RecentProjectDto {
                id: "sample-demo".to_string(),
                name: "Clawvinci Welcome Demo".to_string(),
                path: "sample://welcome-demo".to_string(),
                last_opened: "2026-09-08T12:00:00Z".to_string(),
                duration_seconds: 8.0,
            }],
            onboarding_completed: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StorageInfoDto {
    pub cache_dir: String,
    pub thumbnail_cache_bytes: u64,
    pub waveform_cache_bytes: u64,
    pub transcript_cache_bytes: u64,
    pub total_cache_bytes: u64,
}

pub fn get_settings_path() -> PathBuf {
    if let Ok(appdata) = std::env::var("APPDATA") {
        let mut p = PathBuf::from(appdata);
        p.push("Clawvinci");
        p.push("settings.json");
        p
    } else if let Ok(home) = std::env::var("USERPROFILE") {
        let mut p = PathBuf::from(home);
        p.push(".clawvinci");
        p.push("settings.json");
        p
    } else {
        PathBuf::from("settings.json")
    }
}

pub fn get_cache_root() -> PathBuf {
    if let Ok(local_appdata) = std::env::var("LOCALAPPDATA") {
        let mut p = PathBuf::from(local_appdata);
        p.push("Clawvinci");
        p.push("Cache");
        p
    } else if let Ok(home) = std::env::var("USERPROFILE") {
        let mut p = PathBuf::from(home);
        p.push(".clawvinci");
        p.push("cache");
        p
    } else {
        PathBuf::from(".clawvinci_cache")
    }
}

pub fn load_settings() -> AppSettings {
    let path = get_settings_path();
    if path.exists() {
        if let Ok(content) = fs::read_to_string(&path) {
            if let Ok(settings) = serde_json::from_str::<AppSettings>(&content) {
                return settings;
            }
        }
    }
    AppSettings::default()
}

pub fn save_settings(settings: &AppSettings) -> Result<(), String> {
    let path = get_settings_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let json = serde_json::to_string_pretty(settings).map_err(|e| e.to_string())?;
    fs::write(&path, json).map_err(|e| e.to_string())?;
    Ok(())
}

fn dir_size(path: &Path) -> u64 {
    let mut total = 0;
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir() {
                total += dir_size(&p);
            } else if let Ok(meta) = entry.metadata() {
                total += meta.len();
            }
        }
    }
    total
}

pub fn get_storage_info() -> StorageInfoDto {
    let cache_root = get_cache_root();
    let thumb_dir = cache_root.join("thumbnails");
    let wave_dir = cache_root.join("waveforms");
    let trans_dir = cache_root.join("transcripts");

    let thumbnail_cache_bytes = dir_size(&thumb_dir);
    let waveform_cache_bytes = dir_size(&wave_dir);
    let transcript_cache_bytes = dir_size(&trans_dir);
    let total_cache_bytes = thumbnail_cache_bytes + waveform_cache_bytes + transcript_cache_bytes;

    StorageInfoDto {
        cache_dir: cache_root.to_string_lossy().to_string(),
        thumbnail_cache_bytes,
        waveform_cache_bytes,
        transcript_cache_bytes,
        total_cache_bytes,
    }
}

pub fn clear_cache(cache_type: &str) -> Result<(), String> {
    let cache_root = get_cache_root();
    match cache_type {
        "thumbnails" => {
            let p = cache_root.join("thumbnails");
            if p.exists() {
                fs::remove_dir_all(&p).map_err(|e| e.to_string())?;
            }
        }
        "waveforms" => {
            let p = cache_root.join("waveforms");
            if p.exists() {
                fs::remove_dir_all(&p).map_err(|e| e.to_string())?;
            }
        }
        "transcripts" => {
            let p = cache_root.join("transcripts");
            if p.exists() {
                fs::remove_dir_all(&p).map_err(|e| e.to_string())?;
            }
        }
        "all" => {
            if cache_root.exists() {
                fs::remove_dir_all(&cache_root).map_err(|e| e.to_string())?;
            }
        }
        _ => return Err(format!("Unknown cache type: {cache_type}")),
    }
    Ok(())
}

pub fn record_recent_project(project: RecentProjectDto) -> Result<AppSettings, String> {
    let mut settings = load_settings();
    settings.recent_projects.retain(|p| p.id != project.id && p.path != project.path);
    settings.recent_projects.insert(0, project);
    if settings.recent_projects.len() > 20 {
        settings.recent_projects.truncate(20);
    }
    save_settings(&settings)?;
    Ok(settings)
}
