// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only

use clawvinci_export::queue::ExportQueue;
use clawvinci_mcp::chat::{AgentService, ChatMessage, ChatRole, ChatSessionStore};
use clawvinci_mcp::state::McpState;
use clawvinci_model::timeline::Timeline;
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::test]
async fn test_chat_session_store() {
    let store = ChatSessionStore::new();
    let (sess_id, msgs) = store.get_or_create(None).await;
    assert!(msgs.is_empty());

    let user_msg = ChatMessage::user("Hello Clawvinci");
    store.append_message(&sess_id, user_msg).await;

    let (_, msgs2) = store.get_or_create(Some(sess_id.clone())).await;
    assert_eq!(msgs2.len(), 1);
    assert_eq!(msgs2[0].role, ChatRole::User);
    assert_eq!(msgs2[0].text_content(), "Hello Clawvinci");

    store.clear_session(&sess_id).await;
    let (_, msgs3) = store.get_or_create(Some(sess_id)).await;
    assert!(msgs3.is_empty());
}

#[tokio::test]
async fn test_agent_service_offline_loop() {
    let timeline = Timeline::new(30, 1920, 1080);
    let mcp_state = Arc::new(Mutex::new(McpState::new(
        timeline,
        Vec::new(),
        ExportQueue::new(),
    )));

    let service = AgentService::new(mcp_state);
    let resp = service
        .send_message("Please add a title text", None, None)
        .await
        .expect("Agent message failed");

    assert!(!resp.reply.is_empty());
    assert!(resp.reply.contains("Clawvinci In-App Agent is active"));

    let history = service.get_history(Some(resp.session_id)).await;
    assert_eq!(history.len(), 2, "Expected user + assistant messages in history");
}
