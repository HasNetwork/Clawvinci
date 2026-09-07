// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only
// Derived from Sources/PalmierPro/Agent/Chat/ChatSessionStore.swift (GPLv3).

use crate::chat::types::ChatMessage;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

#[derive(Debug, Clone, Default)]
pub struct ChatSession {
    pub session_id: String,
    pub messages: Vec<ChatMessage>,
}

#[derive(Debug, Clone, Default)]
pub struct ChatSessionStore {
    sessions: Arc<Mutex<HashMap<String, ChatSession>>>,
}

impl ChatSessionStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn get_or_create(&self, session_id: Option<String>) -> (String, Vec<ChatMessage>) {
        let mut store = self.sessions.lock().await;
        let id = session_id.unwrap_or_else(|| Uuid::new_v4().to_string());
        let session = store.entry(id.clone()).or_insert_with(|| ChatSession {
            session_id: id.clone(),
            messages: Vec::new(),
        });
        (id, session.messages.clone())
    }

    pub async fn append_message(&self, session_id: &str, message: ChatMessage) {
        let mut store = self.sessions.lock().await;
        let session = store.entry(session_id.to_string()).or_insert_with(|| ChatSession {
            session_id: session_id.to_string(),
            messages: Vec::new(),
        });
        session.messages.push(message);
    }

    pub async fn clear_session(&self, session_id: &str) {
        let mut store = self.sessions.lock().await;
        if let Some(session) = store.get_mut(session_id) {
            session.messages.clear();
        }
    }
}
