// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only

use clawvinci_model::timeline::Timeline;
use clawvinci_render::engine::{PlaybackEngine, PlaybackStateSnapshot, SeekMode};
use clawvinci_render::plan::FramePlan;
use serde::Serialize;
use std::sync::Arc;
use tokio::sync::Mutex;

type EngineState = Arc<Mutex<PlaybackEngine>>;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FrameRenderResult {
    pub frame_index: usize,
    pub width: u32,
    pub height: u32,
    pub rgba_base64: String,
}

fn bytes_to_base64(bytes: &[u8]) -> String {
    const CHARSET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let b0 = chunk[0];
        let b1 = chunk.get(1).copied().unwrap_or(0);
        let b2 = chunk.get(2).copied().unwrap_or(0);
        out.push(CHARSET[(b0 >> 2) as usize] as char);
        out.push(CHARSET[(((b0 & 3) << 4) | (b1 >> 4)) as usize] as char);
        if chunk.len() > 1 {
            out.push(CHARSET[(((b1 & 15) << 2) | (b2 >> 6)) as usize] as char);
        } else {
            out.push('=');
        }
        if chunk.len() > 2 {
            out.push(CHARSET[(b2 & 63) as usize] as char);
        } else {
            out.push('=');
        }
    }
    out
}

#[tauri::command]
async fn playback_get_state(state: tauri::State<'_, EngineState>) -> Result<PlaybackStateSnapshot, String> {
    let engine = state.lock().await;
    Ok(engine.snapshot())
}

#[tauri::command]
async fn playback_play(state: tauri::State<'_, EngineState>) -> Result<PlaybackStateSnapshot, String> {
    let mut engine = state.lock().await;
    engine.play();
    Ok(engine.snapshot())
}

#[tauri::command]
async fn playback_pause(state: tauri::State<'_, EngineState>) -> Result<PlaybackStateSnapshot, String> {
    let mut engine = state.lock().await;
    engine.pause();
    Ok(engine.snapshot())
}

#[tauri::command]
async fn playback_toggle(state: tauri::State<'_, EngineState>) -> Result<PlaybackStateSnapshot, String> {
    let mut engine = state.lock().await;
    engine.toggle_play();
    Ok(engine.snapshot())
}

#[tauri::command]
async fn playback_seek(
    frame: usize,
    mode: Option<SeekMode>,
    state: tauri::State<'_, EngineState>,
) -> Result<PlaybackStateSnapshot, String> {
    let mut engine = state.lock().await;
    let seek_mode = mode.unwrap_or(SeekMode::Exact);
    engine
        .seek(frame, seek_mode)
        .map_err(|e| e.to_string())?;
    Ok(engine.snapshot())
}

#[tauri::command]
async fn playback_step(
    delta: i64,
    state: tauri::State<'_, EngineState>,
) -> Result<PlaybackStateSnapshot, String> {
    let mut engine = state.lock().await;
    engine.step(delta).map_err(|e| e.to_string())?;
    Ok(engine.snapshot())
}

#[tauri::command]
async fn playback_get_frame_plan(
    frame: usize,
    state: tauri::State<'_, EngineState>,
) -> Result<FramePlan, String> {
    let engine = state.lock().await;
    clawvinci_render::plan::CompositionBuilder::build_frame_plan(engine.timeline(), frame)
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn playback_render_frame(
    frame: usize,
    state: tauri::State<'_, EngineState>,
) -> Result<FrameRenderResult, String> {
    let mut engine = state.lock().await;
    engine.seek(frame, SeekMode::Exact).map_err(|e| e.to_string())?;
    let rendered = engine.render_current_frame().map_err(|e| e.to_string())?;
    let b64 = bytes_to_base64(&rendered.data);
    Ok(FrameRenderResult {
        frame_index: rendered.frame_index,
        width: rendered.width,
        height: rendered.height,
        rgba_base64: b64,
    })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let default_timeline = Timeline::new(30, 1920, 1080);
    let engine: EngineState = Arc::new(Mutex::new(PlaybackEngine::new(default_timeline)));

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(engine)
        .invoke_handler(tauri::generate_handler![
            playback_get_state,
            playback_play,
            playback_pause,
            playback_toggle,
            playback_seek,
            playback_step,
            playback_get_frame_plan,
            playback_render_frame,
        ])
        .setup(|_app| Ok(()))
        .run(tauri::generate_context!())
        .expect("error while running Clawvinci application");
}
