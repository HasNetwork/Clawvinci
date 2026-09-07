// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only
// Derived from Sources/PalmierPro/Agent/Tools/ToolExecutor+Export.swift (GPLv3).

use crate::result::ToolResult;
use crate::state::McpState;
use clawvinci_export::queue::ExportJobSource;
use serde_json::{json, Value};
use std::path::PathBuf;
use uuid::Uuid;

pub fn export_project(args: &Value, state: &mut McpState) -> ToolResult {
    let mode = args.get("mode").and_then(|v| v.as_str()).unwrap_or("video");
    let ext = match mode {
        "xml" => "xml",
        "fcpxml" => "fcpxml",
        "palmier" => "palmier",
        _ => "mp4",
    };

    let default_output = dirs::download_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(format!("{}.{}", state.active_project_name, ext));

    let output_path = args
        .get("outputPath")
        .and_then(|v| v.as_str())
        .map(PathBuf::from)
        .unwrap_or(default_output);

    let project_id = state.editor.timeline().id.clone();
    match state.export_queue.enqueue(
        project_id,
        output_path.clone(),
        ExportJobSource::Agent,
        Vec::new(),
    ) {
        Ok(submission) => {
            state.bump_version();
            ToolResult::json(&json!({
                "jobId": submission.job_id.to_string(),
                "status": if submission.started { "started" } else { "queued" },
                "queuePosition": submission.queue_position,
                "outputPath": output_path.to_string_lossy(),
                "mode": mode
            }))
        }
        Err(e) => ToolResult::error(format!("Failed to enqueue export: {e}")),
    }
}

pub fn manage_exports(args: &Value, state: &mut McpState) -> ToolResult {
    let action = args.get("action").and_then(|v| v.as_str()).unwrap_or("");
    match action {
        "list" => {
            let jobs_json = state
                .export_queue
                .jobs()
                .iter()
                .map(|j| {
                    json!({
                        "jobId": j.id.to_string(),
                        "filename": j.filename,
                        "status": format!("{:?}", j.status).to_lowercase(),
                        "progress": j.progress,
                        "outputPath": j.output_path.to_string_lossy(),
                        "error": j.error,
                        "warnings": j.warnings
                    })
                })
                .collect::<Vec<_>>();
            ToolResult::json(&json!({ "jobs": jobs_json }))
        }
        "cancel" => {
            let job_id_str = match args.get("jobId").and_then(|v| v.as_str()) {
                Some(id) => id,
                None => return ToolResult::error("Missing required parameter 'jobId'"),
            };

            let job_id = match Uuid::parse_str(job_id_str) {
                Ok(id) => id,
                Err(_) => return ToolResult::error("Invalid jobId UUID"),
            };

            if state.export_queue.cancel(job_id) {
                state.bump_version();
                ToolResult::ok(format!("Export job '{job_id_str}' canceled"))
            } else {
                ToolResult::error(format!("Export job '{job_id_str}' not found or already finished"))
            }
        }
        "clear_finished" => {
            state.export_queue.clear_finished(None);
            state.bump_version();
            ToolResult::ok("Cleared finished export jobs")
        }
        other => ToolResult::error(format!("Unsupported export action: '{other}'")),
    }
}

// Fallback dirs provider if dirs crate isn't pulled in directly
mod dirs {
    use std::path::PathBuf;
    pub fn download_dir() -> Option<PathBuf> {
        std::env::var_os("USERPROFILE")
            .map(|p| PathBuf::from(p).join("Downloads"))
            .or_else(|| std::env::var_os("HOME").map(|p| PathBuf::from(p).join("Downloads")))
    }
}
