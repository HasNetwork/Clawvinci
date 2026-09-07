// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only
// Derived from Sources/PalmierPro/Agent/Tools/ToolResult.swift (GPLv3).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum ContentBlock {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "image")]
    Image {
        data: String,
        #[serde(rename = "mimeType")]
        mime_type: String,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolResult {
    pub content: Vec<ContentBlock>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub is_error: bool,
}

impl ToolResult {
    pub fn new(content: Vec<ContentBlock>, is_error: bool) -> Self {
        Self { content, is_error }
    }

    pub fn ok(text: impl Into<String>) -> Self {
        Self {
            content: vec![ContentBlock::Text { text: text.into() }],
            is_error: false,
        }
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self {
            content: vec![ContentBlock::Text {
                text: message.into(),
            }],
            is_error: true,
        }
    }

    pub fn json<T: Serialize>(val: &T) -> Self {
        match serde_json::to_string_pretty(val) {
            Ok(s) => Self::ok(s),
            Err(e) => Self::error(format!("Failed to serialize tool output: {e}")),
        }
    }

    pub fn with_image(data: impl Into<String>, mime_type: impl Into<String>) -> Self {
        Self {
            content: vec![ContentBlock::Image {
                data: data.into(),
                mime_type: mime_type.into(),
            }],
            is_error: false,
        }
    }

    pub fn with_text_and_image(
        text: impl Into<String>,
        data: impl Into<String>,
        mime_type: impl Into<String>,
    ) -> Self {
        Self {
            content: vec![
                ContentBlock::Text { text: text.into() },
                ContentBlock::Image {
                    data: data.into(),
                    mime_type: mime_type.into(),
                },
            ],
            is_error: false,
        }
    }

    pub fn to_text(&self) -> String {
        self.content
            .iter()
            .filter_map(|block| match block {
                ContentBlock::Text { text } => Some(text.as_str()),
                ContentBlock::Image { .. } => None,
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    pub fn to_mcp_result(&self) -> serde_json::Value {
        serde_json::json!({
            "content": self.content,
            "isError": self.is_error
        })
    }
}
