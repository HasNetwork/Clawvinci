// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only

use clawvinci_export::queue::ExportQueue;
use clawvinci_mcp::executor::ToolExecutor;
use clawvinci_mcp::server::{create_mcp_router, ServerState};
use clawvinci_mcp::state::McpState;
use clawvinci_model::clip_type::ClipType;
use clawvinci_model::timeline::{Timeline, Track};
use serde_json::json;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;

#[tokio::test]
async fn test_mcp_http_server_endpoints() {
    let mut timeline = Timeline::new(30, 1920, 1080);
    timeline.tracks.push(Track::new(ClipType::Video));

    let mcp_state = Arc::new(Mutex::new(McpState::new(
        timeline,
        Vec::new(),
        ExportQueue::new(),
    )));

    let server_state = ServerState {
        mcp_state,
        executor: Arc::new(ToolExecutor::new()),
    };

    let app = create_mcp_router(server_state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind ephemeral test port");
    let port = listener.local_addr().unwrap().port();

    let cancel_token = CancellationToken::new();
    let cancel_clone = cancel_token.clone();

    let server_handle = tokio::spawn(async move {
        axum::serve(listener, app)
            .with_graceful_shutdown(async move {
                cancel_clone.cancelled().await;
            })
            .await
            .unwrap();
    });

    let client = reqwest::Client::new();
    let base_url = format!("http://127.0.0.1:{port}");

    // 1. Test POST /mcp (initialize)
    let init_resp = client
        .post(format!("{base_url}/mcp"))
        .json(&json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": { "name": "test-client", "version": "1.0.0" }
            }
        }))
        .send()
        .await
        .expect("Failed to send initialize");
    assert_eq!(init_resp.status(), 200);
    let init_body: serde_json::Value = init_resp.json().await.unwrap();
    assert_eq!(init_body["result"]["serverInfo"]["name"], "clawvinci");

    // 2. Test POST /mcp (tools/list)
    let list_resp = client
        .post(format!("{base_url}/mcp"))
        .json(&json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/list"
        }))
        .send()
        .await
        .expect("Failed to send tools/list");
    assert_eq!(list_resp.status(), 200);
    let list_body: serde_json::Value = list_resp.json().await.unwrap();
    let tools = list_body["result"]["tools"].as_array().unwrap();
    assert_eq!(tools.len(), 52);

    // 3. Test POST /mcp (tools/call - get_timeline)
    let call_resp = client
        .post(format!("{base_url}/mcp"))
        .json(&json!({
            "jsonrpc": "2.0",
            "id": 3,
            "method": "tools/call",
            "params": {
                "name": "get_timeline",
                "arguments": {}
            }
        }))
        .send()
        .await
        .expect("Failed to call get_timeline");
    assert_eq!(call_resp.status(), 200);
    let call_body: serde_json::Value = call_resp.json().await.unwrap();
    assert!(!call_body["result"]["isError"].as_bool().unwrap_or(false));

    // 4. Test GET /.well-known/oauth-protected-resource
    let oauth_resp = client
        .get(format!("{base_url}/.well-known/oauth-protected-resource"))
        .send()
        .await
        .expect("Failed to get oauth resource");
    assert_eq!(oauth_resp.status(), 200);

    // 5. Test GET /mcp (SSE stream)
    let sse_resp = client
        .get(format!("{base_url}/mcp"))
        .send()
        .await
        .expect("Failed to open SSE stream");
    assert_eq!(sse_resp.status(), 200);
    assert_eq!(
        sse_resp
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok()),
        Some("text/event-stream")
    );
    drop(sse_resp);
    drop(client);

    // Teardown server
    cancel_token.cancel();
    server_handle.abort();
}
