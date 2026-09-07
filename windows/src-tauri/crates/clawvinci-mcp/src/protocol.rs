// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only
// Derived from Sources/PalmierPro/Agent/Tools/ToolDefinitions.swift (GPLv3).

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    #[serde(default = "default_jsonrpc")]
    pub jsonrpc: String,
    pub id: Option<Value>,
    pub method: String,
    #[serde(default)]
    pub params: Option<Value>,
}

fn default_jsonrpc() -> String {
    "2.0".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

impl JsonRpcResponse {
    pub fn success(id: Option<Value>, result: Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(result),
            error: None,
        }
    }

    pub fn error(id: Option<Value>, code: i32, message: impl Into<String>) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(JsonRpcError {
                code,
                message: message.into(),
                data: None,
            }),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

pub mod error_codes {
    pub const PARSE_ERROR: i32 = -32700;
    pub const INVALID_REQUEST: i32 = -32600;
    pub const METHOD_NOT_FOUND: i32 = -32601;
    pub const INVALID_PARAMS: i32 = -32602;
    pub const INTERNAL_ERROR: i32 = -32603;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpToolDefinition {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpClientInfo {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpInitializeResult {
    pub protocol_version: String,
    pub capabilities: Value,
    pub server_info: McpServerInfo,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpServerInfo {
    pub name: String,
    pub version: String,
}

/// Returns the complete catalog of all 52 Clawvinci MCP tool definitions.
pub fn all_tool_definitions() -> Vec<McpToolDefinition> {
    vec![
        // 1. Projects & Timelines
        McpToolDefinition {
            name: "manage_project".to_string(),
            description: "List, open, create, or close Clawvinci projects for this MCP session. Set action to: list, open, create, or close.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "action": { "type": "string", "enum": ["list", "open", "create", "close"] },
                    "name": { "type": "string", "description": "Project display name." },
                    "id": { "type": "string", "description": "Project id from list." },
                    "path": { "type": "string", "description": "Path to a .palmier or project file." },
                    "fps": { "type": "integer", "description": "Timeline fps (1-120)." },
                    "aspectRatio": { "type": "string", "description": "Aspect ratio, e.g. '16:9', '9:16'." },
                    "quality": { "type": "string", "enum": ["720p", "1080p", "2K", "4K"] }
                },
                "required": ["action"]
            }),
        },
        McpToolDefinition {
            name: "get_timeline".to_string(),
            description: "Returns project settings, tracks with stable trackId, their current index, type, clips, and gaps.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "startFrame": { "type": "integer", "description": "Optional window start frame." },
                    "endFrame": { "type": "integer", "description": "Optional window end frame." },
                    "captionDetail": { "type": "boolean", "description": "Expand caption groups to individual clips." }
                }
            }),
        },
        McpToolDefinition {
            name: "inspect_timeline".to_string(),
            description: "Inspect timeline diagnostics (gaps, overlaps, offline media) or render composited frame preview.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "startFrame": { "type": "integer", "description": "Frame to sample or diagnose." },
                    "endFrame": { "type": "integer", "description": "Optional end frame for range sampling." },
                    "maxFrames": { "type": "integer", "description": "Max sampled frames (default 6)." }
                }
            }),
        },
        McpToolDefinition {
            name: "create_timeline".to_string(),
            description: "Creates a timeline and switches to it. Without 'from', creates empty; with 'from', duplicates source timeline.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "name": { "type": "string", "description": "Display name." },
                    "from": { "type": "string", "description": "Optional timelineId to duplicate." }
                }
            }),
        },
        McpToolDefinition {
            name: "set_active_timeline".to_string(),
            description: "Switches the active timeline targeted by edit tools.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "timelineId": { "type": "string", "description": "Timeline ID to switch to." }
                },
                "required": ["timelineId"]
            }),
        },
        McpToolDefinition {
            name: "manage_markers".to_string(),
            description: "Creates, updates, or deletes persistent timeline review markers.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "action": { "type": "string", "enum": ["create", "update", "delete", "list"] },
                    "markerId": { "type": "string" },
                    "name": { "type": "string" },
                    "startFrame": { "type": "integer" },
                    "durationFrames": { "type": "integer" },
                    "color": { "type": "string", "description": "#RGB or #RRGGBB hex color." },
                    "comment": { "type": "string" },
                    "status": { "type": "string", "enum": ["open", "review", "resolved"] }
                },
                "required": ["action"]
            }),
        },
        McpToolDefinition {
            name: "set_project_settings".to_string(),
            description: "Change project frame rate, resolution, or aspect ratio. Auto-refits existing timeline clips.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "fps": { "type": "integer" },
                    "width": { "type": "integer" },
                    "height": { "type": "integer" },
                    "aspectRatio": { "type": "string" },
                    "quality": { "type": "string", "enum": ["720p", "1080p", "2K", "4K"] }
                }
            }),
        },
        McpToolDefinition {
            name: "export_project".to_string(),
            description: "Queues an export from the current project (video render, FCPXML, Premiere XML, or .palmier bundle).".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "mode": { "type": "string", "enum": ["video", "xml", "fcpxml", "palmier"] },
                    "codec": { "type": "string", "enum": ["H.264", "H.265", "ProRes"] },
                    "resolution": { "type": "string", "enum": ["720p", "1080p", "2K", "4K", "Match Timeline"] },
                    "outputPath": { "type": "string" },
                    "overwrite": { "type": "boolean" },
                    "fcpxmlTarget": { "type": "string", "enum": ["resolve", "fcp"] },
                    "timelineId": { "type": "string" }
                }
            }),
        },
        McpToolDefinition {
            name: "manage_exports".to_string(),
            description: "Lists export jobs or cancels an active export by jobId.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "action": { "type": "string", "enum": ["list", "cancel", "clear_finished"] },
                    "jobId": { "type": "string" }
                },
                "required": ["action"]
            }),
        },

        // 2. Media Library
        McpToolDefinition {
            name: "get_media".to_string(),
            description: "Enumerates media assets, folders, and timelines in the project library.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "ids": { "type": "array", "items": { "type": "string" } },
                    "folder": { "type": "string" },
                    "pending": { "type": "boolean" }
                }
            }),
        },
        McpToolDefinition {
            name: "inspect_media".to_string(),
            description: "Deep inspection of an asset: stream properties, dimensions, audio layout, or sampled frames.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "mediaRef": { "type": "string" },
                    "clipId": { "type": "string" },
                    "maxFrames": { "type": "integer" },
                    "startSeconds": { "type": "number" },
                    "endSeconds": { "type": "number" },
                    "wordTimestamps": { "type": "boolean" },
                    "overview": { "type": "boolean" }
                },
                "required": ["mediaRef"]
            }),
        },
        McpToolDefinition {
            name: "search_media".to_string(),
            description: "Search media library by visual content or spoken dialogue.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "query": { "type": "string" },
                    "scope": { "type": "string", "enum": ["visual", "spoken", "both"] },
                    "mediaRef": { "type": "string" },
                    "limit": { "type": "integer" }
                },
                "required": ["query"]
            }),
        },
        McpToolDefinition {
            name: "import_media".to_string(),
            description: "Imports external media file, directory, or URL into project library.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "source": {
                        "type": "object",
                        "properties": {
                            "path": { "type": "string" },
                            "url": { "type": "string" },
                            "bytes": { "type": "string" }
                        }
                    },
                    "folder": { "type": "string" }
                },
                "required": ["source"]
            }),
        },
        McpToolDefinition {
            name: "capture_frame".to_string(),
            description: "Render and capture a single frame snapshot as PNG/JPEG base64.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "frame": { "type": "integer" },
                    "format": { "type": "string", "enum": ["png", "jpeg"] },
                    "outputPath": { "type": "string" }
                },
                "required": ["frame"]
            }),
        },
        McpToolDefinition {
            name: "organize_media".to_string(),
            description: "Move media assets into folders/bins or tag them in the media library.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "mediaRef": { "type": "string" },
                    "folder": { "type": "string" },
                    "action": { "type": "string", "enum": ["move", "delete", "tag"] }
                },
                "required": ["mediaRef", "action"]
            }),
        },

        // 3. Clips & Editing
        McpToolDefinition {
            name: "manage_tracks".to_string(),
            description: "Add, remove, mute, hide, lock, reorder, or rename video/audio tracks.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "action": { "type": "string", "enum": ["add", "remove", "mute", "unmute", "hide", "show", "reorder", "rename", "lock", "unlock"] },
                    "trackId": { "type": "string" },
                    "trackType": { "type": "string", "enum": ["video", "audio"] },
                    "targetIndex": { "type": "integer" },
                    "name": { "type": "string" }
                },
                "required": ["action"]
            }),
        },
        McpToolDefinition {
            name: "manage_clip_links".to_string(),
            description: "Link or unlink audio and video clip segments.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "action": { "type": "string", "enum": ["link", "unlink"] },
                    "clipIds": { "type": "array", "items": { "type": "string" } }
                },
                "required": ["action", "clipIds"]
            }),
        },
        McpToolDefinition {
            name: "add_clips".to_string(),
            description: "Append or insert clips onto tracks at specified frame positions.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "trackIndex": { "type": "integer" },
                    "clips": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "mediaRef": { "type": "string" },
                                "startFrame": { "type": "integer" },
                                "durationFrames": { "type": "integer" },
                                "inPoint": { "type": "integer" }
                            },
                            "required": ["mediaRef", "startFrame", "durationFrames"]
                        }
                    }
                },
                "required": ["trackIndex", "clips"]
            }),
        },
        McpToolDefinition {
            name: "insert_clips".to_string(),
            description: "Ripple insert clips into a track at a specific frame, pushing downstream clips.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "trackIndex": { "type": "integer" },
                    "atFrame": { "type": "integer" },
                    "mediaRef": { "type": "string" },
                    "durationFrames": { "type": "integer" }
                },
                "required": ["trackIndex", "atFrame", "mediaRef", "durationFrames"]
            }),
        },
        McpToolDefinition {
            name: "move_clips".to_string(),
            description: "Frame-accurate move of clips to target track or frame position.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "moves": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "clipId": { "type": "string" },
                                "trackIndex": { "type": "integer" },
                                "startFrame": { "type": "integer" }
                            },
                            "required": ["clipId", "trackIndex", "startFrame"]
                        }
                    }
                },
                "required": ["moves"]
            }),
        },
        McpToolDefinition {
            name: "remove_clips".to_string(),
            description: "Lift delete (leaves empty gap) or ripple delete clips from tracks.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "clipIds": { "type": "array", "items": { "type": "string" } },
                    "ripple": { "type": "boolean", "description": "If true, closes gap after deleting." }
                },
                "required": ["clipIds"]
            }),
        },
        McpToolDefinition {
            name: "split_clips".to_string(),
            description: "Razor/split a clip into two segments at a specified frame.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "clipId": { "type": "string" },
                    "atFrame": { "type": "integer" }
                },
                "required": ["clipId", "atFrame"]
            }),
        },
        McpToolDefinition {
            name: "ripple_delete_ranges".to_string(),
            description: "Batch ripple delete frame ranges across all unlocked tracks.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "ranges": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "startFrame": { "type": "integer" },
                                "endFrame": { "type": "integer" }
                            },
                            "required": ["startFrame", "endFrame"]
                        }
                    }
                },
                "required": ["ranges"]
            }),
        },
        McpToolDefinition {
            name: "swap_clip_media".to_string(),
            description: "Replace media source of an existing clip while preserving timing and effects.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "clipId": { "type": "string" },
                    "newMediaRef": { "type": "string" }
                },
                "required": ["clipId", "newMediaRef"]
            }),
        },
        McpToolDefinition {
            name: "set_clip_properties".to_string(),
            description: "Adjust volume, speed, opacity, blend mode, transform, or crop for a clip.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "clipId": { "type": "string" },
                    "volumeDb": { "type": "number" },
                    "opacity": { "type": "number" },
                    "speed": { "type": "number" },
                    "blendMode": { "type": "string" },
                    "transform": {
                        "type": "object",
                        "properties": {
                            "scaleX": { "type": "number" },
                            "scaleY": { "type": "number" },
                            "translationX": { "type": "number" },
                            "translationY": { "type": "number" },
                            "rotationDegrees": { "type": "number" }
                        }
                    }
                },
                "required": ["clipId"]
            }),
        },
        McpToolDefinition {
            name: "copy_clip_settings".to_string(),
            description: "Copy effects, color grade, or transforms from one clip to target clips.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "sourceClipId": { "type": "string" },
                    "targetClipIds": { "type": "array", "items": { "type": "string" } },
                    "properties": { "type": "array", "items": { "type": "string" } }
                },
                "required": ["sourceClipId", "targetClipIds"]
            }),
        },
        McpToolDefinition {
            name: "set_keyframes".to_string(),
            description: "Add or update keyframes for clip properties (opacity, transform, volume).".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "clipId": { "type": "string" },
                    "property": { "type": "string" },
                    "keyframes": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "frame": { "type": "integer" },
                                "value": { "type": "number" }
                            },
                            "required": ["frame", "value"]
                        }
                    }
                },
                "required": ["clipId", "property", "keyframes"]
            }),
        },
        McpToolDefinition {
            name: "apply_layout".to_string(),
            description: "Arrange clips into layout presets (Picture-in-Picture, 2-up split, 4-up grid).".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "layout": { "type": "string", "enum": ["pip", "split2", "grid4"] },
                    "slots": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "clipId": { "type": "string" },
                                "slotIndex": { "type": "integer" }
                            },
                            "required": ["clipId", "slotIndex"]
                        }
                    }
                },
                "required": ["layout", "slots"]
            }),
        },
        McpToolDefinition {
            name: "sync_clips".to_string(),
            description: "Align audio/video clips by source timecode or audio waveform peak.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "referenceClipId": { "type": "string" },
                    "targetClipIds": { "type": "array", "items": { "type": "string" } },
                    "mode": { "type": "string", "enum": ["auto", "timecode", "audio"] }
                },
                "required": ["referenceClipId", "targetClipIds"]
            }),
        },
        McpToolDefinition {
            name: "undo".to_string(),
            description: "Reverts the last editing action on the timeline.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {}
            }),
        },

        // 4. Multicam
        McpToolDefinition {
            name: "manage_multicam".to_string(),
            description: "Create or dissolve multicam clip sync groups.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "action": { "type": "string", "enum": ["create", "dissolve"] },
                    "name": { "type": "string" },
                    "angleMediaRefs": { "type": "array", "items": { "type": "string" } }
                },
                "required": ["action"]
            }),
        },
        McpToolDefinition {
            name: "change_cam".to_string(),
            description: "Switch active camera angle on a multicam clip.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "clipId": { "type": "string" },
                    "angleIndex": { "type": "integer" }
                },
                "required": ["clipId", "angleIndex"]
            }),
        },
        McpToolDefinition {
            name: "get_multicam".to_string(),
            description: "Inspect available camera angles and sync status of a multicam group.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "multicamId": { "type": "string" }
                },
                "required": ["multicamId"]
            }),
        },

        // 5. Transcript & Speech
        McpToolDefinition {
            name: "get_transcript".to_string(),
            description: "Retrieve word-level timecoded transcript for media or timeline.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "mediaRef": { "type": "string" },
                    "startFrame": { "type": "integer" },
                    "endFrame": { "type": "integer" }
                }
            }),
        },
        McpToolDefinition {
            name: "remove_words".to_string(),
            description: "Ripple delete specific word time ranges (filler word removal).".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "wordIndices": { "type": "array", "items": { "type": "integer" } },
                    "mediaRef": { "type": "string" }
                },
                "required": ["wordIndices"]
            }),
        },
        McpToolDefinition {
            name: "remove_silence".to_string(),
            description: "Automatically ripple cut silent pauses exceeding a threshold duration.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "minimumPauseSeconds": { "type": "number" },
                    "speechPaddingSeconds": { "type": "number" },
                    "trackIndex": { "type": "integer" }
                }
            }),
        },
        McpToolDefinition {
            name: "detect_beats".to_string(),
            description: "Detect musical rhythmic beats and mark them as timeline markers.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "mediaRef": { "type": "string" },
                    "createMarkers": { "type": "boolean" }
                },
                "required": ["mediaRef"]
            }),
        },

        // 6. Text & Titles
        McpToolDefinition {
            name: "add_texts".to_string(),
            description: "Add title or text overlay clips to a video track with typography and placement.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "trackIndex": { "type": "integer" },
                    "texts": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "text": { "type": "string" },
                                "startFrame": { "type": "integer" },
                                "durationFrames": { "type": "integer" },
                                "fontName": { "type": "string" },
                                "fontSize": { "type": "number" },
                                "color": { "type": "string" }
                            },
                            "required": ["text", "startFrame", "durationFrames"]
                        }
                    }
                },
                "required": ["trackIndex", "texts"]
            }),
        },
        McpToolDefinition {
            name: "update_text".to_string(),
            description: "Update text content, font styling, or animation preset on an existing text clip.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "clipId": { "type": "string" },
                    "text": { "type": "string" },
                    "fontName": { "type": "string" },
                    "fontSize": { "type": "number" },
                    "color": { "type": "string" }
                },
                "required": ["clipId"]
            }),
        },
        McpToolDefinition {
            name: "add_captions".to_string(),
            description: "Generate and burn-in subtitle cards synchronized to speech audio.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "trackIndex": { "type": "integer" },
                    "style": { "type": "object" }
                }
            }),
        },

        // 7. Color & Effects
        McpToolDefinition {
            name: "apply_color".to_string(),
            description: "Apply color adjustments (exposure, contrast, saturation, temperature, tint, shadows, highlights).".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "clipId": { "type": "string" },
                    "exposure": { "type": "number" },
                    "contrast": { "type": "number" },
                    "saturation": { "type": "number" },
                    "temperature": { "type": "number" },
                    "tint": { "type": "number" },
                    "shadows": { "type": "number" },
                    "highlights": { "type": "number" }
                },
                "required": ["clipId"]
            }),
        },
        McpToolDefinition {
            name: "apply_effect".to_string(),
            description: "Apply or remove GPU shader effects (Gaussian Blur, Sharpen, Vignette, Glow, Edge Detect, etc.).".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "clipId": { "type": "string" },
                    "effectType": { "type": "string" },
                    "params": { "type": "object" },
                    "action": { "type": "string", "enum": ["add", "update", "remove"] }
                },
                "required": ["clipId", "effectType"]
            }),
        },
        McpToolDefinition {
            name: "inspect_color".to_string(),
            description: "Inspect color grading parameters and histogram values for a clip.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "clipId": { "type": "string" }
                },
                "required": ["clipId"]
            }),
        },
        McpToolDefinition {
            name: "denoise_audio".to_string(),
            description: "Apply voice clarity and background noise reduction to audio clips.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "clipId": { "type": "string" },
                    "intensity": { "type": "number" }
                },
                "required": ["clipId"]
            }),
        },

        // 8. Generative AI
        McpToolDefinition {
            name: "list_models".to_string(),
            description: "List available AI generative models for speech, image, video, and music.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {}
            }),
        },
        McpToolDefinition {
            name: "generate_video".to_string(),
            description: "Generate AI video clip from text prompt or image reference.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "prompt": { "type": "string" },
                    "durationSeconds": { "type": "number" }
                },
                "required": ["prompt"]
            }),
        },
        McpToolDefinition {
            name: "generate_image".to_string(),
            description: "Generate AI image asset and add to project library.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "prompt": { "type": "string" },
                    "aspectRatio": { "type": "string" }
                },
                "required": ["prompt"]
            }),
        },
        McpToolDefinition {
            name: "generate_audio".to_string(),
            description: "Generate AI voice narration (TTS) or background music.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "prompt": { "type": "string" },
                    "audioType": { "type": "string", "enum": ["speech", "music"] }
                },
                "required": ["prompt"]
            }),
        },
        McpToolDefinition {
            name: "upscale_media".to_string(),
            description: "Upscale resolution of a media asset using AI super-resolution.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "mediaRef": { "type": "string" },
                    "scaleFactor": { "type": "integer", "enum": [2, 4] }
                },
                "required": ["mediaRef"]
            }),
        },

        // 9. Meta & Skills
        McpToolDefinition {
            name: "send_feedback".to_string(),
            description: "Send user or agent feedback report.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "message": { "type": "string" },
                    "category": { "type": "string" }
                },
                "required": ["message"]
            }),
        },
        McpToolDefinition {
            name: "read_skill".to_string(),
            description: "Load detailed instructions for an agent filmmaking skill.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "id": { "type": "string" }
                },
                "required": ["id"]
            }),
        },
        McpToolDefinition {
            name: "manage_skills".to_string(),
            description: "Create, update, or remove reusable filmmaking skills.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "action": { "type": "string", "enum": ["create", "update", "remove"] },
                    "id": { "type": "string" },
                    "name": { "type": "string" },
                    "instructions": { "type": "string" }
                },
                "required": ["action"]
            }),
        },
    ]
}
