// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only

use clawvinci::settings::{
    clear_cache, get_cache_root, get_settings_path, get_storage_info, AppSettings,
    RecentProjectDto, TrackColorsDto, WhisperSettingsDto,
};

#[test]
fn test_settings_default_and_roundtrip() {
    let default_settings = AppSettings::default();
    assert_eq!(default_settings.language, "en");
    assert_eq!(default_settings.default_frame_rate, 30.0);
    assert_eq!(default_settings.theme, "dark");
    assert!(!default_settings.onboarding_completed);

    let json = serde_json::to_string(&default_settings).expect("serialize settings");
    let decoded: AppSettings = serde_json::from_str(&json).expect("deserialize settings");
    assert_eq!(default_settings, decoded);
}

#[test]
fn test_track_colors_and_whisper_defaults() {
    let colors = TrackColorsDto::default();
    assert_eq!(colors.video, "#1d5878");
    assert_eq!(colors.audio, "#2e7765");
    assert_eq!(colors.text, "#546e7a");
    assert_eq!(colors.image, "#715486");

    let whisper = WhisperSettingsDto::default();
    assert_eq!(whisper.mode, "auto");
    assert_eq!(whisper.model_size, "base");
    assert_eq!(whisper.device, "cpu");
    assert!(whisper.endpoint.is_none());
}

#[test]
fn test_byok_keys_storage() {
    let mut settings = AppSettings::default();
    settings
        .byok_keys
        .insert("openai".to_string(), "sk-test-key-12345".to_string());
    settings
        .byok_keys
        .insert("fal".to_string(), "fal-key-999".to_string());

    let json = serde_json::to_string_pretty(&settings).expect("serialize");
    let decoded: AppSettings = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(decoded.byok_keys.get("openai").unwrap(), "sk-test-key-12345");
    assert_eq!(decoded.byok_keys.get("fal").unwrap(), "fal-key-999");
}

#[test]
fn test_recent_projects_manipulation() {
    let mut settings = AppSettings::default();
    let proj1 = RecentProjectDto {
        id: "p1".to_string(),
        name: "Project One".to_string(),
        path: "C:/Projects/One.palmier".to_string(),
        last_opened: "2026-09-08T10:00:00Z".to_string(),
        duration_seconds: 120.0,
    };
    let proj2 = RecentProjectDto {
        id: "p2".to_string(),
        name: "Project Two".to_string(),
        path: "C:/Projects/Two.palmier".to_string(),
        last_opened: "2026-09-08T11:00:00Z".to_string(),
        duration_seconds: 45.0,
    };

    settings.recent_projects = vec![proj1];
    settings.recent_projects.insert(0, proj2);
    assert_eq!(settings.recent_projects.len(), 2);
    assert_eq!(settings.recent_projects[0].name, "Project Two");
}

#[test]
fn test_storage_info_and_cache_clear() {
    let settings_path = get_settings_path();
    assert!(!settings_path.to_string_lossy().is_empty());

    let cache_root = get_cache_root();
    assert!(!cache_root.to_string_lossy().is_empty());

    let info = get_storage_info();
    assert!(!info.cache_dir.is_empty());

    let res = clear_cache("thumbnails");
    assert!(res.is_ok());

    let invalid = clear_cache("invalid_cache_type");
    assert!(invalid.is_err());
}
