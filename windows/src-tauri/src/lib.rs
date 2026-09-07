// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only

use clawvinci_model::clip_type::ClipType;
use clawvinci_model::effect::{Effect, EffectParam};
use clawvinci_model::text_style::TextStyle;
use clawvinci_model::timeline::{Clip, Timeline, Track};
use clawvinci_render::engine::{PlaybackEngine, PlaybackStateSnapshot, SeekMode};
use clawvinci_render::plan::FramePlan;
use clawvinci_timeline::editor::TimelineEditor;
use clawvinci_timeline::error::TimelineError;
use clawvinci_timeline::ripple::TrimEdge;
use serde::{Deserialize, Serialize};
use clawvinci_export::error::ExportError;
use clawvinci_export::fcpxml::FCPXMLExporter;
use clawvinci_export::options::{
    FCPXMLTarget, FCPXMLVersion, VideoExportOptions,
};
use clawvinci_export::project_bundle::PalmierProjectExporter;
use clawvinci_export::queue::{
    ExportJob, ExportJobSource, ExportJobStatus, ExportQueue,
};
use clawvinci_export::service::ExportService;
use clawvinci_export::xml::XMLExporter;
use clawvinci_mcp::{
    all_tool_definitions, AgentService, McpMediaItem, McpState, SharedMcpState, DEFAULT_MCP_PORT,
};
use clawvinci_model::{MediaManifest, ProjectFile};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaItemDto {
    pub id: String,
    pub name: String,
    pub path: String,
    pub media_type: String,
    pub duration_seconds: f64,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub fps: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryStatusDto {
    pub can_undo: bool,
    pub can_redo: bool,
    pub undo_name: Option<String>,
    pub redo_name: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FrameRenderResult {
    pub frame_index: usize,
    pub width: u32,
    pub height: u32,
    pub rgba_base64: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoExportPayload {
    pub output_path: String,
    pub resolution: Option<String>,
    pub codec: Option<String>,
    pub crf: Option<u32>,
    pub bitrate_kbps: Option<u32>,
    pub is_hdr: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimelineExportPayload {
    pub output_path: String,
    pub format: String,
    pub target: Option<String>,
    pub version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BundleExportPayload {
    pub destination_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CancelJobPayload {
    pub job_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportJobDto {
    pub id: String,
    pub project_id: String,
    pub filename: String,
    pub status: String,
    pub progress: f64,
    pub error: Option<String>,
    pub warnings: Vec<String>,
}

impl From<&ExportJob> for ExportJobDto {
    fn from(job: &ExportJob) -> Self {
        Self {
            id: job.id.to_string(),
            project_id: job.project_id.clone(),
            filename: job.filename.clone(),
            status: format!("{:?}", job.status).to_lowercase(),
            progress: job.progress,
            error: job.error.clone(),
            warnings: job.warnings.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportQueueSubmissionDto {
    pub job_id: String,
    pub started: bool,
    pub queue_position: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectBundleReportDto {
    pub collected_count: usize,
    pub missing_count: usize,
    pub total_bytes: u64,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentChatResponseDto {
    pub reply: String,
    pub tools_called: Vec<String>,
    pub session_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatMessageDto {
    pub id: String,
    pub role: String,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpServerStatusDto {
    pub running: bool,
    pub port: u16,
    pub url: String,
    pub tools_count: usize,
}

pub struct AppState {
    pub editor: TimelineEditor,
    pub engine: PlaybackEngine,
    pub media_items: Vec<MediaItemDto>,
    pub export_queue: ExportQueue,
    pub mcp_state: SharedMcpState,
    pub agent_service: Arc<AgentService>,
    pub mcp_cancel_token: CancellationToken,
}

type SharedState = Arc<Mutex<AppState>>;

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

// MARK: - Playback Commands

#[tauri::command]
async fn playback_get_state(state: tauri::State<'_, SharedState>) -> Result<PlaybackStateSnapshot, String> {
    let app = state.lock().await;
    Ok(app.engine.snapshot())
}

#[tauri::command]
async fn playback_play(state: tauri::State<'_, SharedState>) -> Result<PlaybackStateSnapshot, String> {
    let mut app = state.lock().await;
    app.engine.play();
    Ok(app.engine.snapshot())
}

#[tauri::command]
async fn playback_pause(state: tauri::State<'_, SharedState>) -> Result<PlaybackStateSnapshot, String> {
    let mut app = state.lock().await;
    app.engine.pause();
    Ok(app.engine.snapshot())
}

#[tauri::command]
async fn playback_toggle(state: tauri::State<'_, SharedState>) -> Result<PlaybackStateSnapshot, String> {
    let mut app = state.lock().await;
    app.engine.toggle_play();
    Ok(app.engine.snapshot())
}

#[tauri::command]
async fn playback_seek(
    frame: usize,
    mode: Option<SeekMode>,
    state: tauri::State<'_, SharedState>,
) -> Result<PlaybackStateSnapshot, String> {
    let mut app = state.lock().await;
    let seek_mode = mode.unwrap_or(SeekMode::Exact);
    app.engine
        .seek(frame, seek_mode)
        .map_err(|e| e.to_string())?;
    Ok(app.engine.snapshot())
}

#[tauri::command]
async fn playback_step(
    delta: i64,
    state: tauri::State<'_, SharedState>,
) -> Result<PlaybackStateSnapshot, String> {
    let mut app = state.lock().await;
    app.engine.step(delta).map_err(|e| e.to_string())?;
    Ok(app.engine.snapshot())
}

#[tauri::command]
async fn playback_get_frame_plan(
    frame: usize,
    state: tauri::State<'_, SharedState>,
) -> Result<FramePlan, String> {
    let app = state.lock().await;
    clawvinci_render::plan::CompositionBuilder::build_frame_plan(app.engine.timeline(), frame)
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn playback_render_frame(
    frame: usize,
    state: tauri::State<'_, SharedState>,
) -> Result<FrameRenderResult, String> {
    let mut app = state.lock().await;
    app.engine.seek(frame, SeekMode::Exact).map_err(|e| e.to_string())?;
    let rendered = app.engine.render_current_frame().map_err(|e| e.to_string())?;
    let b64 = bytes_to_base64(&rendered.data);
    Ok(FrameRenderResult {
        frame_index: rendered.frame_index,
        width: rendered.width,
        height: rendered.height,
        rgba_base64: b64,
    })
}

// MARK: - Timeline Editing Commands

#[tauri::command]
async fn timeline_get(state: tauri::State<'_, SharedState>) -> Result<Timeline, String> {
    let app = state.lock().await;
    Ok(app.editor.timeline().clone())
}

#[tauri::command]
async fn timeline_split_clip(
    clip_id: String,
    at_frame: i64,
    state: tauri::State<'_, SharedState>,
) -> Result<Timeline, String> {
    let mut app = state.lock().await;
    app.editor
        .split_clip(&clip_id, at_frame)
        .map_err(|e| e.to_string())?;
    let timeline = app.editor.timeline().clone();
    app.engine.set_timeline(timeline.clone());
    Ok(timeline)
}

#[tauri::command]
async fn timeline_trim_clip(
    clip_id: String,
    edge: String,
    delta: i64,
    state: tauri::State<'_, SharedState>,
) -> Result<Timeline, String> {
    let mut app = state.lock().await;
    let trim_edge = if edge.eq_ignore_ascii_case("left") || edge.eq_ignore_ascii_case("head") {
        TrimEdge::Left
    } else {
        TrimEdge::Right
    };
    app.editor
        .trim_clip(&clip_id, trim_edge, delta)
        .map_err(|e| e.to_string())?;
    let timeline = app.editor.timeline().clone();
    app.engine.set_timeline(timeline.clone());
    Ok(timeline)
}

#[tauri::command]
async fn timeline_move_clips(
    moves: Vec<(String, usize, i64)>,
    state: tauri::State<'_, SharedState>,
) -> Result<Timeline, String> {
    let mut app = state.lock().await;
    app.editor
        .move_clips(&moves)
        .map_err(|e| e.to_string())?;
    let timeline = app.editor.timeline().clone();
    app.engine.set_timeline(timeline.clone());
    Ok(timeline)
}

#[tauri::command]
async fn timeline_remove_clips(
    clip_ids: Vec<String>,
    state: tauri::State<'_, SharedState>,
) -> Result<Timeline, String> {
    let mut app = state.lock().await;
    let set: HashSet<String> = clip_ids.into_iter().collect();
    app.editor
        .remove_clips(&set)
        .map_err(|e| e.to_string())?;
    let timeline = app.editor.timeline().clone();
    app.engine.set_timeline(timeline.clone());
    Ok(timeline)
}

#[tauri::command]
async fn timeline_ripple_delete(
    clip_ids: Vec<String>,
    state: tauri::State<'_, SharedState>,
) -> Result<Timeline, String> {
    let mut app = state.lock().await;
    let set: HashSet<String> = clip_ids.into_iter().collect();
    app.editor
        .ripple_delete(&set)
        .map_err(|e| e.to_string())?;
    let timeline = app.editor.timeline().clone();
    app.engine.set_timeline(timeline.clone());
    Ok(timeline)
}

#[tauri::command]
async fn timeline_insert_track(
    track_type: String,
    state: tauri::State<'_, SharedState>,
) -> Result<Timeline, String> {
    let mut app = state.lock().await;
    let kind = match track_type.to_lowercase().as_str() {
        "audio" => ClipType::Audio,
        "image" => ClipType::Image,
        "text" => ClipType::Text,
        _ => ClipType::Video,
    };
    let index = app.editor.timeline().tracks.len();
    app.editor
        .insert_track(index, kind)
        .map_err(|e| e.to_string())?;
    let timeline = app.editor.timeline().clone();
    app.engine.set_timeline(timeline.clone());
    Ok(timeline)
}

#[tauri::command]
async fn timeline_undo(state: tauri::State<'_, SharedState>) -> Result<Timeline, String> {
    let mut app = state.lock().await;
    app.editor.undo().map_err(|e| e.to_string())?;
    let timeline = app.editor.timeline().clone();
    app.engine.set_timeline(timeline.clone());
    Ok(timeline)
}

#[tauri::command]
async fn timeline_redo(state: tauri::State<'_, SharedState>) -> Result<Timeline, String> {
    let mut app = state.lock().await;
    app.editor.redo().map_err(|e| e.to_string())?;
    let timeline = app.editor.timeline().clone();
    app.engine.set_timeline(timeline.clone());
    Ok(timeline)
}

#[tauri::command]
async fn timeline_history_status(
    state: tauri::State<'_, SharedState>,
) -> Result<HistoryStatusDto, String> {
    let app = state.lock().await;
    Ok(HistoryStatusDto {
        can_undo: app.editor.can_undo(),
        can_redo: app.editor.can_redo(),
        undo_name: app.editor.undo_action_name().map(String::from),
        redo_name: app.editor.redo_action_name().map(String::from),
    })
}

#[allow(clippy::too_many_arguments)]
#[tauri::command]
async fn timeline_update_clip_transform(
    clip_id: String,
    center_x: f64,
    center_y: f64,
    width: f64,
    height: f64,
    rotation: f64,
    opacity: f64,
    state: tauri::State<'_, SharedState>,
) -> Result<Timeline, String> {
    let mut app = state.lock().await;
    app.editor
        .perform("Update Transform", |tl| {
            for track in &mut tl.tracks {
                for clip in &mut track.clips {
                    if clip.id == clip_id {
                        clip.transform.center_x = center_x;
                        clip.transform.center_y = center_y;
                        clip.transform.width = width;
                        clip.transform.height = height;
                        clip.transform.rotation = rotation;
                        clip.opacity = opacity.clamp(0.0, 1.0);
                        return Ok(());
                    }
                }
            }
            Err(TimelineError::ClipNotFound(clip_id.clone()))
        })
        .map_err(|e| e.to_string())?;
    let timeline = app.editor.timeline().clone();
    app.engine.set_timeline(timeline.clone());
    Ok(timeline)
}

#[tauri::command]
async fn timeline_update_clip_effect(
    clip_id: String,
    effect_type: String,
    param_key: String,
    value: f64,
    state: tauri::State<'_, SharedState>,
) -> Result<Timeline, String> {
    let mut app = state.lock().await;
    app.editor
        .perform("Update Effect", |tl| {
            for track in &mut tl.tracks {
                for clip in &mut track.clips {
                    if clip.id == clip_id {
                        let effects = clip.effects.get_or_insert_with(Vec::new);
                        if let Some(eff) = effects.iter_mut().find(|e| e.effect_type == effect_type) {
                            eff.params.insert(param_key, EffectParam::from_value(value));
                        } else {
                            let mut eff = Effect::new(effect_type);
                            eff.params.insert(param_key, EffectParam::from_value(value));
                            effects.push(eff);
                        }
                        return Ok(());
                    }
                }
            }
            Err(TimelineError::ClipNotFound(clip_id.clone()))
        })
        .map_err(|e| e.to_string())?;
    let timeline = app.editor.timeline().clone();
    app.engine.set_timeline(timeline.clone());
    Ok(timeline)
}

#[tauri::command]
async fn timeline_update_clip_text(
    clip_id: String,
    text: String,
    font_size: f64,
    state: tauri::State<'_, SharedState>,
) -> Result<Timeline, String> {
    let mut app = state.lock().await;
    app.editor
        .perform("Update Text", |tl| {
            for track in &mut tl.tracks {
                for clip in &mut track.clips {
                    if clip.id == clip_id {
                        clip.text_content = Some(text);
                        if let Some(style) = &mut clip.text_style {
                            style.font_size = font_size;
                        } else {
                            clip.text_style = Some(TextStyle {
                                font_size,
                                ..Default::default()
                            });
                        }
                        return Ok(());
                    }
                }
            }
            Err(TimelineError::ClipNotFound(clip_id.clone()))
        })
        .map_err(|e| e.to_string())?;
    let timeline = app.editor.timeline().clone();
    app.engine.set_timeline(timeline.clone());
    Ok(timeline)
}

// MARK: - Media Pool Commands

#[tauri::command]
async fn media_list(state: tauri::State<'_, SharedState>) -> Result<Vec<MediaItemDto>, String> {
    let app = state.lock().await;
    Ok(app.media_items.clone())
}

#[tauri::command]
async fn media_import(path: String, state: tauri::State<'_, SharedState>) -> Result<MediaItemDto, String> {
    let file_path = Path::new(&path);
    let probe_opt = if let Ok(ctx) = clawvinci_media::FfmpegContext::discover().await {
        clawvinci_media::probe_media(&ctx, file_path).await.ok()
    } else {
        None
    };

    let mut app = state.lock().await;
    let item = if let Some(probe) = probe_opt {
        let name = file_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Untitled Media")
            .to_string();
        let is_video = !probe.video_streams.is_empty();
        let (width, height, fps) = if let Some(v) = probe.primary_video() {
            (Some(v.width), Some(v.height), Some(v.fps))
        } else {
            (None, None, None)
        };
        MediaItemDto {
            id: format!("media-{}", app.media_items.len() + 1),
            name,
            path,
            media_type: if is_video { "video".to_string() } else { "audio".to_string() },
            duration_seconds: probe.duration_seconds,
            width,
            height,
            fps,
        }
    } else {
        let name = file_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Imported File")
            .to_string();
        MediaItemDto {
            id: format!("media-{}", app.media_items.len() + 1),
            name,
            path,
            media_type: "video".to_string(),
            duration_seconds: 5.0,
            width: Some(1920),
            height: Some(1080),
            fps: Some(30.0),
        }
    };
    app.media_items.push(item.clone());
    Ok(item)
}

// MARK: - Export Commands

async fn process_export_queue(state: SharedState) {
    loop {
        let (job_id, cancel_token, output_path) = {
            let mut app = state.lock().await;
            if let Some((id, token)) = app.export_queue.next_waiting_job() {
                if let Some(job) = app.export_queue.get_job(id) {
                    (id, token, job.output_path.clone())
                } else {
                    break;
                }
            } else {
                break;
            }
        };

        let (timeline, sources) = {
            let app = state.lock().await;
            (app.editor.timeline().clone(), HashMap::new())
        };

        let state_cb = Arc::clone(&state);
        let progress_cb = move |progress: f64| {
            let state_inner = Arc::clone(&state_cb);
            tokio::spawn(async move {
                let mut app = state_inner.lock().await;
                app.export_queue.update_progress(job_id, progress);
            });
        };

        let opts = VideoExportOptions::default();
        let res = ExportService::render_timeline_to_file(
            &timeline,
            &opts,
            &output_path,
            &sources,
            cancel_token,
            progress_cb,
        )
        .await;

        let mut app = state.lock().await;
        match res {
            Ok(_) => app.export_queue.finish_job(job_id, ExportJobStatus::Completed, None, vec![]),
            Err(ExportError::Cancelled) => app.export_queue.finish_job(
                job_id,
                ExportJobStatus::Canceled,
                Some("Cancelled by user".to_string()),
                vec![],
            ),
            Err(e) => app.export_queue.finish_job(
                job_id,
                ExportJobStatus::Failed,
                Some(e.to_string()),
                vec![],
            ),
        }
    }
}

#[tauri::command]
async fn export_enqueue_video(
    payload: VideoExportPayload,
    state: tauri::State<'_, SharedState>,
) -> Result<ExportQueueSubmissionDto, String> {
    let output_path = PathBuf::from(&payload.output_path);
    let mut app = state.lock().await;

    let sub = app
        .export_queue
        .enqueue(
            "default-project".to_string(),
            output_path,
            ExportJobSource::Manual,
            vec![],
        )
        .map_err(|e| e.to_string())?;

    let started = sub.started;
    let dto = ExportQueueSubmissionDto {
        job_id: sub.job_id.to_string(),
        started: sub.started,
        queue_position: sub.queue_position,
    };

    drop(app);

    if started {
        let state_clone = Arc::clone(state.inner());
        tokio::spawn(async move {
            process_export_queue(state_clone).await;
        });
    }

    Ok(dto)
}

#[tauri::command]
async fn export_enqueue_timeline(
    payload: TimelineExportPayload,
    state: tauri::State<'_, SharedState>,
) -> Result<ExportQueueSubmissionDto, String> {
    let output_path = PathBuf::from(&payload.output_path);
    let mut app = state.lock().await;

    let sub = app
        .export_queue
        .enqueue(
            "default-project".to_string(),
            output_path.clone(),
            ExportJobSource::Manual,
            vec![],
        )
        .map_err(|e| e.to_string())?;

    let job_id = sub.job_id;
    let timeline = app.editor.timeline().clone();
    let media_paths: HashMap<String, String> = app
        .media_items
        .iter()
        .map(|m| (m.id.clone(), m.path.clone()))
        .collect();

    let format_lower = payload.format.to_lowercase();
    let res = match format_lower.as_str() {
        "xmeml" | "xml" => XMLExporter::export_to_file(&timeline, &media_paths, &output_path),
        "fcpxml" => {
            let target = match payload.target.as_deref() {
                Some("fcp") => FCPXMLTarget::FinalCutPro,
                _ => FCPXMLTarget::Resolve,
            };
            let version = match payload.version.as_deref() {
                Some("1.11") => FCPXMLVersion::V1_11,
                Some("1.12") => FCPXMLVersion::V1_12,
                Some("1.13") => FCPXMLVersion::V1_13,
                Some("1.14") => FCPXMLVersion::V1_14,
                _ => FCPXMLVersion::V1_10,
            };
            FCPXMLExporter::export_to_file(&timeline, &media_paths, version, target, &output_path)
        }
        other => Err(ExportError::InvalidFormat(other.to_string())),
    };

    match res {
        Ok(_) => app.export_queue.finish_job(job_id, ExportJobStatus::Completed, None, vec![]),
        Err(e) => app.export_queue.finish_job(job_id, ExportJobStatus::Failed, Some(e.to_string()), vec![]),
    }

    Ok(ExportQueueSubmissionDto {
        job_id: sub.job_id.to_string(),
        started: true,
        queue_position: 1,
    })
}

#[tauri::command]
async fn export_enqueue_project_bundle(
    payload: BundleExportPayload,
    state: tauri::State<'_, SharedState>,
) -> Result<ProjectBundleReportDto, String> {
    let dest_path = PathBuf::from(&payload.destination_path);
    let app = state.lock().await;
    let timeline = app.editor.timeline().clone();
    let project_file = ProjectFile::new(vec![timeline]);
    let manifest = MediaManifest::default();
    drop(app);

    let cancel_token = tokio_util::sync::CancellationToken::new();
    let report = PalmierProjectExporter::export(
        &project_file,
        &manifest,
        None,
        &dest_path,
        cancel_token,
        None,
    )
    .await
    .map_err(|e| e.to_string())?;

    Ok(ProjectBundleReportDto {
        collected_count: report.collected.len(),
        missing_count: report.missing.len(),
        total_bytes: report.total_bytes,
        warnings: report.warnings(),
    })
}

#[tauri::command]
async fn export_queue_list(
    state: tauri::State<'_, SharedState>,
) -> Result<Vec<ExportJobDto>, String> {
    let app = state.lock().await;
    Ok(app.export_queue.jobs().iter().map(ExportJobDto::from).collect())
}

#[tauri::command]
async fn export_queue_cancel(
    payload: CancelJobPayload,
    state: tauri::State<'_, SharedState>,
) -> Result<bool, String> {
    let uuid = Uuid::parse_str(&payload.job_id).map_err(|e| e.to_string())?;
    let mut app = state.lock().await;
    Ok(app.export_queue.cancel(uuid))
}

#[tauri::command]
async fn export_queue_clear_finished(
    state: tauri::State<'_, SharedState>,
) -> Result<(), String> {
    let mut app = state.lock().await;
    app.export_queue.clear_finished(None);
    Ok(())
}

// MARK: - Agent Chat & MCP Server Commands

#[tauri::command]
async fn agent_chat_send(
    message: String,
    session_id: Option<String>,
    state: tauri::State<'_, SharedState>,
) -> Result<AgentChatResponseDto, String> {
    let service = {
        let app = state.lock().await;
        app.agent_service.clone()
    };

    let resp = service.send_message(&message, session_id, None).await?;

    let (cloned_editor, cloned_timeline) = {
        let app = state.lock().await;
        let mcp = app.mcp_state.lock().await;
        (mcp.editor.clone(), mcp.editor.timeline().clone())
    };
    {
        let mut app = state.lock().await;
        app.editor = cloned_editor;
        app.engine.set_timeline(cloned_timeline);
    }

    Ok(AgentChatResponseDto {
        reply: resp.reply,
        tools_called: resp.tools_called,
        session_id: resp.session_id,
    })
}

#[tauri::command]
async fn agent_chat_history(
    session_id: Option<String>,
    state: tauri::State<'_, SharedState>,
) -> Result<Vec<ChatMessageDto>, String> {
    let service = {
        let app = state.lock().await;
        app.agent_service.clone()
    };

    let messages = service.get_history(session_id).await;
    Ok(messages
        .into_iter()
        .map(|m| {
            let text = m.text_content();
            let role = format!("{:?}", m.role).to_lowercase();
            ChatMessageDto {
                id: m.id,
                role,
                text,
            }
        })
        .collect())
}

#[tauri::command]
async fn agent_chat_clear(
    session_id: Option<String>,
    state: tauri::State<'_, SharedState>,
) -> Result<(), String> {
    let service = {
        let app = state.lock().await;
        app.agent_service.clone()
    };

    if let Some(id) = session_id {
        service.clear_history(&id).await;
    }
    Ok(())
}

#[tauri::command]
async fn mcp_server_status() -> Result<McpServerStatusDto, String> {
    let tools = all_tool_definitions();
    Ok(McpServerStatusDto {
        running: true,
        port: DEFAULT_MCP_PORT,
        url: format!("http://127.0.0.1:{DEFAULT_MCP_PORT}/mcp"),
        tools_count: tools.len(),
    })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mut timeline = Timeline::new(30, 1920, 1080);

    // Default Track 0: Video Track
    let mut video_track = Track::new(ClipType::Video);
    video_track.name = Some("Video 1".to_string());
    let mut clip1 = Clip::new("sample-media-1", 0, 90);
    clip1.id = "clip-v1".to_string();
    let mut clip2 = Clip::new("sample-media-2", 90, 150);
    clip2.id = "clip-v2".to_string();
    video_track.clips.push(clip1);
    video_track.clips.push(clip2);
    timeline.tracks.push(video_track);

    // Default Track 1: Audio Track
    let mut audio_track = Track::new(ClipType::Audio);
    audio_track.name = Some("Audio 1".to_string());
    let mut clip_a1 = Clip::new("sample-audio-1", 0, 240);
    clip_a1.id = "clip-a1".to_string();
    clip_a1.media_type = ClipType::Audio;
    clip_a1.source_clip_type = ClipType::Audio;
    audio_track.clips.push(clip_a1);
    timeline.tracks.push(audio_track);

    // Default Track 2: Title Text Track
    let mut text_track = Track::new(ClipType::Text);
    text_track.name = Some("Titles".to_string());
    let mut clip_t1 = Clip::new("sample-text-1", 15, 120);
    clip_t1.id = "clip-t1".to_string();
    clip_t1.media_type = ClipType::Text;
    clip_t1.source_clip_type = ClipType::Text;
    clip_t1.text_content = Some("Clawvinci Studio".to_string());
    let style = TextStyle {
        font_size: 72.0,
        ..Default::default()
    };
    clip_t1.text_style = Some(style);
    text_track.clips.push(clip_t1);
    timeline.tracks.push(text_track);

    let media_items = vec![
        MediaItemDto {
            id: "sample-media-1".to_string(),
            name: "Mountain_Landscape_4K.mp4".to_string(),
            path: "sample-media-1".to_string(),
            media_type: "video".to_string(),
            duration_seconds: 3.0,
            width: Some(1920),
            height: Some(1080),
            fps: Some(30.0),
        },
        MediaItemDto {
            id: "sample-media-2".to_string(),
            name: "City_Streets_Night.mp4".to_string(),
            path: "sample-media-2".to_string(),
            media_type: "video".to_string(),
            duration_seconds: 5.0,
            width: Some(1920),
            height: Some(1080),
            fps: Some(30.0),
        },
        MediaItemDto {
            id: "sample-audio-1".to_string(),
            name: "Atmospheric_Synth_Theme.wav".to_string(),
            path: "sample-audio-1".to_string(),
            media_type: "audio".to_string(),
            duration_seconds: 8.0,
            width: None,
            height: None,
            fps: None,
        },
    ];

    let mcp_items: Vec<McpMediaItem> = media_items
        .iter()
        .map(|m| McpMediaItem {
            id: m.id.clone(),
            name: m.name.clone(),
            path: m.path.clone(),
            media_type: m.media_type.clone(),
            duration_seconds: m.duration_seconds,
            width: m.width,
            height: m.height,
            fps: m.fps,
            has_audio: m.media_type == "audio" || m.media_type == "video",
            folder: None,
            generation_prompt: None,
            generation_status: None,
        })
        .collect();

    let mcp_state: SharedMcpState = Arc::new(tokio::sync::Mutex::new(McpState::new(
        timeline.clone(),
        mcp_items,
        ExportQueue::new(),
    )));

    let mcp_cancel_token = CancellationToken::new();
    clawvinci_mcp::start_mcp_server(
        DEFAULT_MCP_PORT,
        mcp_state.clone(),
        mcp_cancel_token.clone(),
    );

    let agent_service = Arc::new(AgentService::new(mcp_state.clone()));

    let editor = TimelineEditor::new(timeline.clone());
    let engine = PlaybackEngine::new(timeline);
    let export_queue = ExportQueue::new();
    let state: SharedState = Arc::new(Mutex::new(AppState {
        editor,
        engine,
        media_items,
        export_queue,
        mcp_state,
        agent_service,
        mcp_cancel_token,
    }));

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            playback_get_state,
            playback_play,
            playback_pause,
            playback_toggle,
            playback_seek,
            playback_step,
            playback_get_frame_plan,
            playback_render_frame,
            timeline_get,
            timeline_split_clip,
            timeline_trim_clip,
            timeline_move_clips,
            timeline_remove_clips,
            timeline_ripple_delete,
            timeline_insert_track,
            timeline_undo,
            timeline_redo,
            timeline_history_status,
            timeline_update_clip_transform,
            timeline_update_clip_effect,
            timeline_update_clip_text,
            media_list,
            media_import,
            export_enqueue_video,
            export_enqueue_timeline,
            export_enqueue_project_bundle,
            export_queue_list,
            export_queue_cancel,
            export_queue_clear_finished,
            agent_chat_send,
            agent_chat_history,
            agent_chat_clear,
            mcp_server_status,
        ])
        .setup(|_app| Ok(()))
        .run(tauri::generate_context!())
        .expect("error while running Clawvinci application");
}
