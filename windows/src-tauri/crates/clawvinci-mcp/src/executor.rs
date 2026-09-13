// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only
// Derived from Sources/PalmierPro/Agent/Tools/ToolExecutor.swift (GPLv3).

use crate::result::ToolResult;
use crate::state::McpState;
use crate::tools::generate::GenerationContext;
use crate::tools::*;
use serde_json::Value;
use std::sync::Arc;

#[derive(Clone)]
pub struct ToolExecutor {
    gen_ctx: Option<Arc<GenerationContext>>,
}

impl Default for ToolExecutor {
    fn default() -> Self {
        Self { gen_ctx: None }
    }
}

impl ToolExecutor {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_generation_context(gen_ctx: Arc<GenerationContext>) -> Self {
        Self {
            gen_ctx: Some(gen_ctx),
        }
    }

    /// Cleans up empty or null strings that some AI models pass as autofilled defaults.
    pub fn clean_args(args: &Value) -> Value {
        match args {
            Value::Object(map) => {
                let mut cleaned = serde_json::Map::new();
                for (k, v) in map {
                    match v {
                        Value::Null => {}
                        Value::String(s) if s.is_empty() => {}
                        other => {
                            cleaned.insert(k.clone(), Self::clean_args(other));
                        }
                    }
                }
                Value::Object(cleaned)
            }
            Value::Array(arr) => Value::Array(arr.iter().map(Self::clean_args).collect()),
            other => other.clone(),
        }
    }

    /// Dispatches a tool execution request by name to the appropriate handler.
    pub fn execute(&self, name: &str, args: &Value, state: &mut McpState) -> ToolResult {
        let cleaned = Self::clean_args(args);
        match name {
            // Projects & Timelines
            "manage_project" => project::manage_project(&cleaned, state),
            "get_timeline" => timeline::get_timeline(&cleaned, state),
            "inspect_timeline" => timeline::inspect_timeline(&cleaned, state),
            "create_timeline" => project::create_timeline(&cleaned, state),
            "set_active_timeline" => project::set_active_timeline(&cleaned, state),
            "manage_markers" => markers::manage_markers(&cleaned, state),
            "set_project_settings" => timeline::set_project_settings(&cleaned, state),
            "export_project" => export::export_project(&cleaned, state),
            "manage_exports" => export::manage_exports(&cleaned, state),

            // Media Library
            "get_media" => media::get_media(&cleaned, state),
            "inspect_media" => media::inspect_media(&cleaned, state),
            "search_media" => media::search_media(&cleaned, state),
            "import_media" => media::import_media(&cleaned, state),
            "capture_frame" => capture::capture_frame(&cleaned, state),
            "organize_media" => media::organize_media(&cleaned, state),

            // Clips & Timeline Editing
            "manage_tracks" => clips::manage_tracks(&cleaned, state),
            "manage_clip_links" => clips::manage_clip_links(&cleaned, state),
            "add_clips" => clips::add_clips(&cleaned, state),
            "insert_clips" => clips::insert_clips(&cleaned, state),
            "move_clips" => clips::move_clips(&cleaned, state),
            "remove_clips" => clips::remove_clips(&cleaned, state),
            "split_clips" => clips::split_clips(&cleaned, state),
            "ripple_delete_ranges" => clips::ripple_delete_ranges(&cleaned, state),
            "swap_clip_media" => clips::swap_clip_media(&cleaned, state),
            "set_clip_properties" => clips::set_clip_properties(&cleaned, state),
            "copy_clip_settings" => clips::copy_clip_settings(&cleaned, state),
            "set_keyframes" => clips::set_keyframes(&cleaned, state),
            "apply_layout" => layout::apply_layout(&cleaned, state),
            "sync_clips" => sync::sync_clips(&cleaned, state),
            "undo" => clips::undo(&cleaned, state),

            // Multicam
            "manage_multicam" => multicam::manage_multicam(&cleaned, state),
            "change_cam" => multicam::change_cam(&cleaned, state),
            "get_multicam" => multicam::get_multicam(&cleaned, state),

            // Transcript & Speech
            "get_transcript" => transcript::get_transcript(&cleaned, state),
            "remove_words" => transcript::remove_words(&cleaned, state),
            "remove_silence" => transcript::remove_silence(&cleaned, state),
            "detect_beats" => transcript::detect_beats(&cleaned, state),

            // Text & Captions
            "add_texts" => text::add_texts(&cleaned, state),
            "update_text" => text::update_text(&cleaned, state),
            "add_captions" => text::add_captions(&cleaned, state),

            // Color & Effects
            "apply_color" => color::apply_color(&cleaned, state),
            "apply_effect" => effects::apply_effect(&cleaned, state),
            "inspect_color" => color::inspect_color(&cleaned, state),
            "denoise_audio" => effects::denoise_audio(&cleaned, state),

            // Generation & AI
            "list_models" => generate::list_models(&cleaned, state),
            "generate_video" => {
                generate::generate_video(&cleaned, state, self.gen_ctx.as_deref())
            }
            "generate_image" => {
                generate::generate_image(&cleaned, state, self.gen_ctx.as_deref())
            }
            "generate_audio" => {
                generate::generate_audio(&cleaned, state, self.gen_ctx.as_deref())
            }
            "upscale_media" => {
                generate::upscale_media(&cleaned, state, self.gen_ctx.as_deref())
            }

            // Skills & Feedback
            "send_feedback" => meta::send_feedback(&cleaned, state),
            "read_skill" => meta::read_skill(&cleaned, state),
            "manage_skills" => meta::manage_skills(&cleaned, state),

            unknown => ToolResult::error(format!("Unknown tool: '{unknown}'")),
        }
    }
}
