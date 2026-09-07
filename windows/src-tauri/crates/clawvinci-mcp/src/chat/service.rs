// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only
// Derived from Sources/PalmierPro/Agent/Chat/AgentService.swift (GPLv3).

use crate::chat::anthropic::{AnthropicClient, AnthropicContentBlock};
use crate::chat::session::ChatSessionStore;
use crate::chat::types::{AgentChatResponse, ChatContentBlock, ChatMessage};
use crate::executor::ToolExecutor;
use crate::state::SharedMcpState;
use std::sync::Arc;

pub struct AgentService {
    mcp_state: SharedMcpState,
    session_store: ChatSessionStore,
    executor: Arc<ToolExecutor>,
    anthropic: AnthropicClient,
}

impl AgentService {
    pub fn new(mcp_state: SharedMcpState) -> Self {
        Self {
            mcp_state,
            session_store: ChatSessionStore::new(),
            executor: Arc::new(ToolExecutor::new()),
            anthropic: AnthropicClient::new(),
        }
    }

    pub async fn send_message(
        &self,
        user_text: &str,
        session_id: Option<String>,
        api_key: Option<&str>,
    ) -> Result<AgentChatResponse, String> {
        let (sess_id, mut messages) = self.session_store.get_or_create(session_id).await;

        let user_msg = ChatMessage::user(user_text);
        self.session_store.append_message(&sess_id, user_msg.clone()).await;
        messages.push(user_msg);

        let mut tools_called = Vec::new();
        let mut final_reply = String::new();
        let max_turns = 10;

        for _turn in 0..max_turns {
            let response = self.anthropic.send_messages(api_key, &messages).await?;

            let mut tool_use_blocks = Vec::new();
            let mut text_parts = Vec::new();

            for block in response.content {
                match block {
                    AnthropicContentBlock::Text { text } => {
                        text_parts.push(text);
                    }
                    AnthropicContentBlock::ToolUse { id, name, input } => {
                        tool_use_blocks.push(ChatContentBlock::ToolUse { id, name, input });
                    }
                }
            }

            if !tool_use_blocks.is_empty() {
                let assistant_msg = ChatMessage::assistant_tool_use(tool_use_blocks.clone());
                self.session_store.append_message(&sess_id, assistant_msg.clone()).await;
                messages.push(assistant_msg);

                let mut tool_result_blocks = Vec::new();
                for block in tool_use_blocks {
                    if let ChatContentBlock::ToolUse { id, name, input } = block {
                        tools_called.push(name.clone());

                        let mut state = self.mcp_state.lock().await;
                        let result = self.executor.execute(&name, &input, &mut state);

                        tool_result_blocks.push(ChatContentBlock::ToolResult {
                            tool_use_id: id,
                            content: result.to_text(),
                            is_error: result.is_error,
                        });
                    }
                }

                let tool_msg = ChatMessage::tool_results(tool_result_blocks);
                self.session_store.append_message(&sess_id, tool_msg.clone()).await;
                messages.push(tool_msg);
            } else {
                final_reply = text_parts.join("\n");
                let assistant_msg = ChatMessage::assistant(&final_reply);
                self.session_store.append_message(&sess_id, assistant_msg).await;
                break;
            }
        }

        Ok(AgentChatResponse {
            reply: final_reply,
            tools_called,
            session_id: sess_id,
        })
    }

    pub async fn get_history(&self, session_id: Option<String>) -> Vec<ChatMessage> {
        let (_, messages) = self.session_store.get_or_create(session_id).await;
        messages
    }

    pub async fn clear_history(&self, session_id: &str) {
        self.session_store.clear_session(session_id).await;
    }
}
