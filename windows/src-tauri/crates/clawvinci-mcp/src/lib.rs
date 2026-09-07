// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only
//! Clawvinci MCP Server & Tool Layer (Phase 8):
//! Embedded HTTP/SSE Model Context Protocol server, 52-tool execution engine,
//! and in-app AI chat orchestration.

pub mod chat;
pub mod executor;
pub mod instructions;
pub mod protocol;
pub mod result;
pub mod server;
pub mod state;
pub mod tools;

pub use chat::AgentService;
pub use executor::ToolExecutor;
pub use instructions::SERVER_INSTRUCTIONS;
pub use protocol::{all_tool_definitions, McpToolDefinition};
pub use result::{ContentBlock, ToolResult};
pub use server::{create_mcp_router, start_mcp_server, DEFAULT_MCP_PORT};
pub use state::{McpMarker, McpMediaItem, McpSkill, McpState, SharedMcpState};
