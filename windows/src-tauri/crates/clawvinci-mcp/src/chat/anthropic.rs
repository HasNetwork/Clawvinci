// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only
// Derived from Sources/PalmierPro/Agent/Clients/AnthropicProvider.swift (GPLv3).

use crate::chat::types::{ChatContentBlock, ChatMessage, ChatRole};
use crate::instructions::SERVER_INSTRUCTIONS;
use crate::protocol::all_tool_definitions;
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnthropicResponse {
    pub id: Option<String>,
    #[serde(rename = "type")]
    pub response_type: Option<String>,
    pub role: Option<String>,
    pub content: Vec<AnthropicContentBlock>,
    pub stop_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AnthropicContentBlock {
    Text {
        text: String,
    },
    ToolUse {
        id: String,
        name: String,
        input: Value,
    },
}

pub struct AnthropicClient {
    client: reqwest::Client,
}

impl Default for AnthropicClient {
    fn default() -> Self {
        Self::new()
    }
}

impl AnthropicClient {
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(60))
            .build()
            .unwrap_or_default();
        Self { client }
    }

    pub async fn send_messages(
        &self,
        api_key: Option<&str>,
        messages: &[ChatMessage],
    ) -> Result<AnthropicResponse, String> {
        let key = match api_key.or(option_env!("ANTHROPIC_API_KEY")) {
            Some(k) if !k.trim().is_empty() => k.trim(),
            _ => {
                // Offline fallback when no key is configured
                return Ok(AnthropicResponse {
                    id: Some("offline-reply".to_string()),
                    response_type: Some("message".to_string()),
                    role: Some("assistant".to_string()),
                    content: vec![AnthropicContentBlock::Text {
                        text: "Clawvinci In-App Agent is active. To enable live Claude model inference, set your ANTHROPIC_API_KEY or configure BYOK in settings. External agents can connect directly via http://127.0.0.1:19789/mcp.".to_string(),
                    }],
                    stop_reason: Some("end_turn".to_string()),
                });
            }
        };

        let tools = all_tool_definitions()
            .into_iter()
            .map(|t| {
                json!({
                    "name": t.name,
                    "description": t.description,
                    "input_schema": t.input_schema
                })
            })
            .collect::<Vec<_>>();

        let anthropic_messages = messages
            .iter()
            .map(|m| {
                let role = match m.role {
                    ChatRole::User => "user",
                    ChatRole::Assistant => "assistant",
                    ChatRole::Tool => "user",
                };
                let content_blocks = m
                    .content
                    .iter()
                    .map(|b| match b {
                        ChatContentBlock::Text { text } => json!({
                            "type": "text",
                            "text": text
                        }),
                        ChatContentBlock::ToolUse { id, name, input } => json!({
                            "type": "tool_use",
                            "id": id,
                            "name": name,
                            "input": input
                        }),
                        ChatContentBlock::ToolResult {
                            tool_use_id,
                            content,
                            is_error,
                        } => json!({
                            "type": "tool_result",
                            "tool_use_id": tool_use_id,
                            "content": content,
                            "is_error": is_error
                        }),
                    })
                    .collect::<Vec<_>>();

                json!({
                    "role": role,
                    "content": content_blocks
                })
            })
            .collect::<Vec<_>>();

        let payload = json!({
            "model": "claude-3-5-sonnet-20241022",
            "max_tokens": 4096,
            "system": SERVER_INSTRUCTIONS,
            "messages": anthropic_messages,
            "tools": tools
        });

        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert(
            "x-api-key",
            HeaderValue::from_str(key).map_err(|e| e.to_string())?,
        );
        headers.insert(
            "anthropic-version",
            HeaderValue::from_static("2023-06-01"),
        );

        let resp = self
            .client
            .post("https://api.anthropic.com/v1/messages")
            .headers(headers)
            .json(&payload)
            .send()
            .await
            .map_err(|e| format!("HTTP request to Anthropic API failed: {e}"))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(format!("Anthropic API error ({status}): {body}"));
        }

        resp.json::<AnthropicResponse>()
            .await
            .map_err(|e| format!("Failed to parse Anthropic response: {e}"))
    }
}
