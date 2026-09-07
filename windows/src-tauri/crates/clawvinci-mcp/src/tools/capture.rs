// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only
// Derived from Sources/PalmierPro/Agent/Tools/ToolExecutor+CaptureFrame.swift (GPLv3).

use crate::result::ToolResult;
use crate::state::McpState;
use clawvinci_render::compositor::composite_frame;
use clawvinci_render::plan::CompositionBuilder;
use serde_json::{json, Value};
use std::collections::HashMap;

pub fn capture_frame(args: &Value, state: &mut McpState) -> ToolResult {
    let frame = args.get("frame").and_then(|v| v.as_i64()).unwrap_or(0);
    let target_frame = frame.max(0) as usize;

    let timeline = state.editor.timeline();
    let plan = match CompositionBuilder::build_frame_plan(timeline, target_frame) {
        Ok(p) => p,
        Err(e) => return ToolResult::error(format!("Failed to build frame plan for frame {frame}: {e}")),
    };

    let sources = HashMap::new();
    let rendered = match composite_frame(&plan, &sources) {
        Ok(f) => f,
        Err(e) => return ToolResult::error(format!("Failed to composite frame {frame}: {e}")),
    };

    let b64 = bytes_to_base64(&rendered.data);

    if let Some(output_path) = args.get("outputPath").and_then(|v| v.as_str()) {
        if let Err(e) = std::fs::write(output_path, &rendered.data) {
            return ToolResult::error(format!("Failed to write captured frame to '{output_path}': {e}"));
        }
        ToolResult::json(&json!({
            "frame": target_frame,
            "width": rendered.width,
            "height": rendered.height,
            "outputPath": output_path
        }))
    } else {
        ToolResult::with_text_and_image(
            format!("Captured frame {target_frame} ({}x{})", rendered.width, rendered.height),
            b64,
            "image/png",
        )
    }
}

fn bytes_to_base64(bytes: &[u8]) -> String {
    const CHARSET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let b0 = chunk[0];
        let b1 = if chunk.len() > 1 { chunk[1] } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] } else { 0 };

        out.push(CHARSET[(b0 >> 2) as usize] as char);
        out.push(CHARSET[(((b0 & 0x03) << 4) | (b1 >> 4)) as usize] as char);
        if chunk.len() > 1 {
            out.push(CHARSET[(((b1 & 0x0f) << 2) | (b2 >> 6)) as usize] as char);
        } else {
            out.push('=');
        }
        if chunk.len() > 2 {
            out.push(CHARSET[(b2 & 0x3f) as usize] as char);
        } else {
            out.push('=');
        }
    }
    out
}
