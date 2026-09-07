// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only
// Derived from Sources/PalmierPro/Agent/MCP/MCPHTTPServer.swift (GPLv3).

use crate::executor::ToolExecutor;
use crate::instructions::SERVER_INSTRUCTIONS;
use crate::protocol::{
    all_tool_definitions, error_codes, JsonRpcRequest, JsonRpcResponse, McpInitializeResult,
    McpServerInfo,
};
use crate::state::SharedMcpState;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use futures_util::stream::{self, Stream};
use serde_json::{json, Value};
use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio_util::sync::CancellationToken;
use tower_http::cors::CorsLayer;
use tracing::{error, info};

pub const DEFAULT_MCP_PORT: u16 = 19789;

#[derive(Clone)]
pub struct ServerState {
    pub mcp_state: SharedMcpState,
    pub executor: Arc<ToolExecutor>,
}

pub fn create_mcp_router(server_state: ServerState) -> Router {
    Router::new()
        .route("/mcp", post(handle_json_rpc))
        .route("/mcp", get(handle_sse))
        .route("/mcp", axum::routing::delete(handle_delete))
        .route("/", post(handle_json_rpc))
        .route("/", get(handle_sse))
        .route(
            "/.well-known/oauth-protected-resource",
            get(handle_oauth_discovery),
        )
        .layer(CorsLayer::permissive())
        .with_state(server_state)
}

async fn handle_json_rpc(
    State(server_state): State<ServerState>,
    Json(req): Json<JsonRpcRequest>,
) -> Response {
    let req_id = req.id.clone();
    match req.method.as_str() {
        "initialize" => {
            let result = McpInitializeResult {
                protocol_version: "2024-11-05".to_string(),
                capabilities: json!({
                    "tools": { "listChanged": true }
                }),
                server_info: McpServerInfo {
                    name: "clawvinci".to_string(),
                    version: "0.1.0".to_string(),
                },
                instructions: Some(SERVER_INSTRUCTIONS.to_string()),
            };
            Json(JsonRpcResponse::success(req_id, json!(result))).into_response()
        }
        "notifications/initialized" => StatusCode::NO_CONTENT.into_response(),
        "ping" => Json(JsonRpcResponse::success(req_id, json!({}))).into_response(),
        "tools/list" => {
            let tools = all_tool_definitions();
            Json(JsonRpcResponse::success(req_id, json!({ "tools": tools }))).into_response()
        }
        "tools/call" => {
            let params = req.params.unwrap_or(Value::Null);
            let tool_name = params.get("name").and_then(|v| v.as_str()).unwrap_or("");
            let tool_args = params.get("arguments").unwrap_or(&Value::Null);

            let mut state = server_state.mcp_state.lock().await;
            let result = server_state.executor.execute(tool_name, tool_args, &mut state);
            Json(JsonRpcResponse::success(req_id, result.to_mcp_result())).into_response()
        }
        unknown => Json(JsonRpcResponse::error(
            req_id,
            error_codes::METHOD_NOT_FOUND,
            format!("Method not found: '{unknown}'"),
        ))
        .into_response(),
    }
}

async fn handle_sse() -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let stream = stream::repeat_with(|| {
        Ok(Event::default().comment("keepalive"))
    });
    Sse::new(stream).keep_alive(KeepAlive::new().interval(Duration::from_secs(15)))
}

async fn handle_delete() -> StatusCode {
    StatusCode::OK
}

async fn handle_oauth_discovery() -> Json<Value> {
    Json(json!({
        "resource": format!("http://127.0.0.1:{DEFAULT_MCP_PORT}")
    }))
}

/// Spawns the MCP server on a background Tokio task.
pub fn start_mcp_server(
    port: u16,
    mcp_state: SharedMcpState,
    cancel_token: CancellationToken,
) -> tokio::task::JoinHandle<()> {
    let server_state = ServerState {
        mcp_state,
        executor: Arc::new(ToolExecutor::new()),
    };
    let app = create_mcp_router(server_state);

    tokio::spawn(async move {
        let addr = SocketAddr::from(([127, 0, 0, 1], port));
        let listener = match tokio::net::TcpListener::bind(addr).await {
            Ok(l) => {
                info!("Clawvinci MCP server listening on http://{}", addr);
                l
            }
            Err(e) => {
                error!("Failed to bind MCP server to port {port}: {e}");
                return;
            }
        };

        if let Err(e) = axum::serve(listener, app)
            .with_graceful_shutdown(async move {
                cancel_token.cancelled().await;
                info!("MCP server shutting down gracefully");
            })
            .await
        {
            error!("MCP server error: {e}");
        }
    })
}
