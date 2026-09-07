# Phase 8.0 — MCP Agent Layer (`clawvinci-mcp`)

**Status**: ✅ Complete  
**CI Verification**: ✅ 100% Green on GitHub Actions Run `34150452889` (Check, Clippy, Test, Tauri Build & Artifact Upload)  
**Date**: 2026-09-07  

## What was built

Phase 8.0 implements the complete MCP (Model Context Protocol) Agent Layer for Clawvinci (`clawvinci-mcp`), realizing the core product identity ("The Cursor IDE of Video Editing"): an embedded HTTP/SSE MCP server on `http://127.0.0.1:19789/mcp`, an authoritative 52-tool execution engine wired directly into `clawvinci-timeline` undoable operations, an in-app AI chat orchestrator with Anthropic Claude tool-use loops, and Tauri IPC desktop integration:

1. **Protocol Envelope & Complete 52-Tool Catalog (`src/protocol.rs`):**
   - Implements JSON-RPC 2.0 protocol specifications for Model Context Protocol (MCP 2024-11-05).
   - Exposes exact tool schemas matching `Sources/PalmierPro/Agent/Tools/ToolDefinitions.swift`:
     - **Project & Timelines (9 tools)**: `manage_project`, `get_timeline`, `inspect_timeline`, `create_timeline`, `set_active_timeline`, `manage_markers`, `set_project_settings`, `export_project`, `manage_exports`.
     - **Media Library (6 tools)**: `get_media`, `inspect_media`, `search_media`, `import_media`, `capture_frame`, `organize_media`.
     - **Clips & Timeline Editing (15 tools)**: `manage_tracks`, `manage_clip_links`, `add_clips`, `insert_clips`, `move_clips`, `remove_clips`, `split_clips`, `ripple_delete_ranges`, `swap_clip_media`, `set_clip_properties`, `copy_clip_settings`, `set_keyframes`, `apply_layout`, `sync_clips`, `undo`.
     - **Multicam (3 tools)**: `manage_multicam`, `change_cam`, `get_multicam`.
     - **Audio & Transcript (4 tools)**: `get_transcript`, `remove_words`, `remove_silence`, `detect_beats`.
     - **Text & Captions (3 tools)**: `add_texts`, `update_text`, `add_captions`.
     - **Color & Effects (4 tools)**: `apply_color`, `inspect_color`, `apply_effect`, `denoise_audio`.
     - **Generative Media (3 tools)**: `generate_media`, `check_generation`, `cancel_generation`.
     - **Meta & Agent Instructions (5 tools)**: `read_editor_instructions`, `list_skills`, `get_skill`, `post_feedback`, `report_activity`.

2. **Tool Execution Engine (`src/executor.rs` & `src/tools/*`):**
   - Unified `ToolExecutor` dispatching commands to 17 modular tool handlers.
   - Enforces the **Shared Mutation & Undo Invariant**: every timeline mutation (`add_clips`, `split_clips`, `remove_clips`, `set_clip_properties`, `apply_layout`, etc.) executes via `clawvinci_timeline::editor::TimelineEditor::perform`, placing user and AI edits onto the identical undo stack.
   - Preserves integer frame boundaries (`i64`) and converts properties into domain types (`ClipType`, `BlendMode`, `TextStyle`, `EffectParam`, `Transform`, `Crop`).

3. **Embedded HTTP & SSE MCP Server (`src/server.rs`):**
   - High-performance, asynchronous Axum service listening on `127.0.0.1:19789`.
   - `POST /mcp`: Handles standard MCP JSON-RPC 2.0 requests:
     - `initialize`: Protocol negotiation, server capabilities (`tools: {}`), and server info (`clawvinci-mcp v0.1.0`).
     - `tools/list`: Returns full 52-tool schema definitions with JSONSchema argument properties.
     - `tools/call`: Dispatches to `ToolExecutor` and formats `ToolResult` content blocks.
   - `GET /mcp/sse`: Server-Sent Events endpoint streaming server notifications and tool events.
   - `GET /health`: Fast health-check probe returning server status and active timeline ID.
   - Permissive CORS enabling connections from CLI tools (`claude mcp add ...`), browser extensions, and IDE plugins.

4. **In-App Agent Chat Orchestrator (`src/chat/*`):**
   - `types.rs`: Message envelopes, roles (`User`, `Assistant`, `Tool`), and session metadata.
   - `session.rs`: Thread-safe session management with thread storage and message history retrieval.
   - `anthropic.rs`: Anthropic Messages API client supporting multi-turn tool calling:
     - Formats Clawvinci tool schemas into Anthropic tool definitions.
     - Streams reasoning/text tokens and parses `tool_use` blocks.
     - Re-injects tool results as `tool_result` blocks for continuous multi-step agent reasoning.
   - `service.rs`: `AgentService` tying together the chat UI, LLM provider, and local `ToolExecutor`.

5. **Tauri IPC & Desktop Shell Integration (`windows/src-tauri/src/lib.rs`):**
   - Starts the HTTP MCP server as a background daemon upon application startup (`tauri::async_runtime::spawn`).
   - Registers Tauri IPC commands for frontend chat panels:
     - `agent_chat_send`: Dispatches user prompt to in-app agent, executing required tools and returning assistant response.
     - `agent_chat_history`: Retrieves conversation history for a given session.
     - `agent_chat_clear`: Resets a conversation session.
     - `mcp_server_status`: Returns current server port, running status, and tool catalog count.

6. **Automated Unit & Integration Test Suite (`tests/*`):**
   - `test_protocol.rs`: JSON-RPC serialization, 52-tool schema validation, tool result envelopes.
   - `test_tools.rs`: Full timeline edit cycle: `get_timeline` → `add_clips` → `split_clips` → `undo` verification, plus markers, text titles, color grading, and effects.
   - `test_server.rs`: Spawns live Axum test server on random port, exercises `initialize` and `tools/list` HTTP requests.
   - `test_chat.rs`: Multi-turn chat session management and history retention.

## Verification & Invariants Preserved

- **Scope Discipline**: Edited strictly within `windows/` and `.agents/`. All code under `Sources/` remains unmodified reference.
- **Shared Undo Invariant**: AI agent mutations flow through `state.editor.perform(...)`, ensuring Agent edits and UI edits are indistinguishable on the undo stack.
- **Remote CI Verification**: Validated remotely on GitHub Actions CI. Zero local cargo/npm toolchain invocations.
- **Port Stability**: Standardized on port `19789`, maintaining 100% compatibility with Palmier Pro's documented Claude Code integration (`claude mcp add --transport http clawvinci http://127.0.0.1:19789/mcp`).
