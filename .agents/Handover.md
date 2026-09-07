# Clawvinci Project Handover & Implementation Guide

> **Target Audience**: Incoming AI Agent / Engineering Team  
> **Workspace Root**: `d:\projects\palmier-win\.claude\worktrees\plan-windows-port`  
> **Current Git Branch**: `worktree-plan-windows-port`  
> **Last Clean Commit**: `37ad69a`  
> **CI Status**: ✅ 100% Green (GitHub Actions Run `34102735695` — Check, Clippy, Test, Tauri Build & Artifact Upload)  
> **Current Milestone**: Phase 7.0 Complete & Verified; Ready to Implement Phase 8.0 (MCP Agent Layer)

---

## 1. Executive Summary & Project Mission

**Clawvinci** is the native Windows port of **Palmier Pro** — an AI-native desktop video editor featuring an embedded Model Context Protocol (MCP) server that empowers AI agents to edit timeline sequences directly alongside human editors.

### Key Architectural Pillars
- **Not a Simple Source Port**: Palmier Pro relies heavily on macOS-only frameworks (SwiftUI, AppKit, AVFoundation, CoreImage, Metal, MLX). Clawvinci is a **ground-up native Windows application** reproducing Palmier Pro's exact behavior, file format, visual aesthetics, and 53-tool MCP surface.
- **Core Technology Stack**:
  - **Backend**: Rust workspace (`windows/src-tauri/crates/*`) managing media decode/encode, timeline state, undo/redo history, frame compositing, and export.
  - **Frontend Shell**: Tauri v2 (`windows/src/*`) hosting a high-performance web frontend with vanilla HTML5/CSS/JavaScript and Canvas-based timeline rendering.
  - **Target Platform**: Windows 10 22H2+ (x86_64).
  - **Licensing**: GPL-3.0-only (mirroring the original Palmier Pro license).
- **Single Source of Truth**: The `.agents/` directory is the authoritative repository of decisions, plans, rules, and history.

### Product Mental Model: "The Cursor IDE of Video Editing"
A foundational mental model for Clawvinci is that it is **"Cursor IDE" but for Video Editing**:
- **In Software Engineering**: Modern AI-first editors (like Cursor or Windsurf) bridge human developers and AI models by granting agents direct, tool-driven manipulation of the workspace—reading files, analyzing ASTs, generating surgical diffs, and inspecting compiler feedback.
- **In Video Editing**: Clawvinci bridges human video creators and AI agents by granting models direct, tool-driven manipulation of the NLE timeline—reading media manifests, inspecting audio waveforms, executing frame-accurate ripple/overwrite edits, adjusting color grades, tuning transforms, and generating kinetic text via the Model Context Protocol (MCP).
- **Symmetric Collaboration**: Human editor and AI agent share the exact same domain state, see the exact same canvas, and operate through a unified, frame-accurate undo/redo history.

---

## 2. Inviolable Engineering Rules & Invariants

All future agents working on this codebase **must strictly abide by these constraints**:

1. **`Sources/` is Read-Only Reference**:
   - The original macOS Swift source code at the repository root (`Sources/`, `Metal/`, `Plugins/`, `models/`, `scripts/`, `mcpb/`) is strictly **read-only**.
   - **Never edit, move, or delete files outside of `windows/` and `.agents/`**.
2. **Unified Render Path (Do Not Re-implement)**:
   - Both live timeline preview and batch export **must share the exact same compositing pipeline**: `clawvinci_render::compositor::composite_frame` and `CompositionBuilder::build_frame_plan`.
   - Never write a second, standalone compositor for export or thumbnail generation.
3. **Integer Frame Timing**:
   - All timeline positions, durations, in/out points, and trims are integer frame counts (`i64` / `u64`). Floating-point time (`f64`) is converted only at external display boundaries. Exact timebase conversions must use rational arithmetic.
4. **Main Thread & UI Isolation**:
   - The Tauri main/UI thread must remain completely unblocked. All file I/O, FFmpeg probing/decoding/encoding, image processing, and heavy transforms must execute in background tasks (`tokio::task::spawn_blocking` or dedicated worker threads).
5. **Cooperative Cancellation**:
   - All asynchronous pipelines (preview rendering, export jobs, waveform extractions) must carry and check a `tokio_util::sync::CancellationToken` at frame boundaries to abort immediately without leaking system resources or orphaned files.
6. **Zero-Warning Tolerance & Remote Verification Only**:
   - Do **NOT** install or run `cargo`, `rustc`, or `npm` locally on the host machine. All verification runs remotely on GitHub Actions CI (`.github/workflows/ci.yml`).
   - All code must pass `cargo clippy --workspace -- -D warnings` with zero warnings.
   - All unit and integration tests must pass cleanly (`cargo test --workspace`).

---

## 3. Current Implementation Status (Phases 0 – 7 Complete)

| Phase | Module / Crate | Scope & Deliverables | Verification Status |
|---|---|---|---|
| **0.0** | `windows/` workspace | Foundation layout, 9 Cargo crates, Tauri v2 shell, GitHub Actions Windows CI (`.github/workflows/ci.yml`). | ✅ CI Green (`0-0-foundation.md`) |
| **1.0** | `clawvinci-model` | 21 domain models ported from Swift (`Timeline`, `Track`, `Clip`, `Timecode`, `ProjectFile`). Byte-compatible `.palmier` format + legacy decode fallback. | ✅ CI Green (`1-0-domain-model.md`) |
| **2.0** | `clawvinci-media` | FFmpeg probe, decode (`VideoStreamReader`), encode (`VideoStreamWriter`), waveform extraction, thumbnails, bounded concurrency. | ✅ CI Green (`2-0-media-engine.md`) |
| **3.0** | `clawvinci-timeline` | Core editing engine: `TimelineEditor`, undo/redo command history, ripple/overwrite edits, clip/track operations, editor coordinator. | ✅ CI Green (`3-0-timeline-editing-core.md`) |
| **4.0** | `clawvinci-render` | Playback engine: `FramePlan` builder, shared CPU compositor with blend modes, `PlaybackEngine` clock synchronization, Tauri IPC. | ✅ CI Green (`4-0-playback-engine.md`) |
| **5.0** | `clawvinci-render` | Effects pipeline: 12 Metal shader algorithms ported, `EffectRegistry` in canonical order, `.cube` LUT parser & tetrahedral interpolation, `fontdue` text layout & animators. | ✅ CI Green (`5-0-gpu-effects.md`) |
| **6.0** | `windows/src` & `lib.rs` | UI Shell: Premium Dark Design System, AppTheme tokens, interactive multi-track canvas timeline, inspector panel, media asset panel, 16 Tauri IPC commands. | ✅ CI Green (`6-0-ui-shell.md`, Run `34041846290`) |
| **7.0** | `clawvinci-export` | Batch video render-to-file, FCPXML 1.10–1.14 export, Premiere XMEML 4 export, self-contained `.palmier` bundle export, export queue. | ✅ CI Green (`7-0-export.md`, Run `34102735695`) |
| **8.0** | `clawvinci-mcp` | **Current Target**: 53-tool MCP execution engine, embedded HTTP MCP server on `127.0.0.1:19789/mcp`, in-app agent chat orchestration. | 🚀 **Ready for Implementation** |
| **9.0–13.0** | Various | Audio analysis (ONNX), search & transcription, generative AI, auth/telemetry, polish. | 📋 Planned in `.agents/PLAN/` |

---

## 4. Phase 8.0 — MCP Agent Layer Detailed Execution Blueprint

The incoming agent's primary assignment is to implement **Phase 8.0 (`clawvinci-mcp`)**.

### 4.1 Architecture Overview
Phase 8 implements the core differentiating identity of Clawvinci: the Model Context Protocol (MCP) layer.
There are **two distinct operational modes** that consume the same underlying tool execution engine:

1. **Embedded HTTP MCP Server** (`Sources/PalmierPro/Agent/MCP/`):
   - Listens on `http://127.0.0.1:19789/mcp`.
   - Exposes standard Model Context Protocol endpoints (JSON-RPC 2.0 over HTTP/SSE) allowing external agents (Claude Code, Cursor, Codex, Claude Desktop) to connect directly:
     ```bash
     claude mcp add --transport http clawvinci http://127.0.0.1:19789/mcp
     ```
2. **In-App Agent Chat Orchestrator** (`Sources/PalmierPro/Agent/Chat/` and `Agent/Clients/`):
   - Internal agent panel driving LLM providers directly (Anthropic Claude, OpenAI, BYOK).
   - Manages message history, streaming token presentation buffer, system prompts (`AgentInstructions`), and multi-turn tool calling.
   - Executes tools against the exact same domain state.

### 4.2 The 53 Tool Definitions & Groupings
Reference source: `Sources/PalmierPro/Agent/Tools/ToolDefinitions.swift` (16,684 LOC across 27 files).

All 53 tools must be registered with exact schemas and routed through a central `ToolExecutor`:

| Group | Tools | Backing Operations |
|---|---|---|
| **Projects & Timelines** (9) | `manage_project`, `get_timeline`, `inspect_timeline`, `create_timeline`, `set_active_timeline`, `manage_markers`, `set_project_settings`, `export_project`, `manage_exports` | Queries/mutates project state, markers, settings, and enqueues jobs into `ExportQueue` (Phase 7). |
| **Media Library** (6) | `get_media`, `inspect_media`, `search_media`, `import_media`, `capture_frame`, `organize_media` | Media manifest enumeration, metadata inspection, frame capture via `clawvinci-render`. |
| **Clips & Timeline Editing** (15) | `manage_tracks`, `manage_clip_links`, `add_clips`, `insert_clips`, `move_clips`, `remove_clips`, `split_clips`, `ripple_delete_ranges`, `swap_clip_media`, `set_clip_properties`, `copy_clip_settings`, `set_keyframes`, `apply_layout`, `sync_clips`, `undo` | **Critical**: Must call straight into `clawvinci_timeline::TimelineEditor` so all agent edits produce unified, undoable actions. |
| **Multicam** (3) | `manage_multicam`, `change_cam`, `get_multicam` | Multi-angle clip grouping, active angle switching, and angle audio synchronization. |
| **Transcript & Speech** (4) | `get_transcript`, `remove_words`, `remove_silence`, `detect_beats` | Word-level ripple deletions, speech silence detection, and musical rhythm beat grid markers. |
| **Text & Titles** (3) | `add_texts`, `update_text`, `add_captions` | Adding subtitle/title clips, animated kinetic typography styles, and batch caption tracks. |
| **Color & Effects** (10) | `apply_color`, `apply_effect`, `remove_effect`, `update_effect`, `denoise_audio`, `adjust_audio_levels`, `duck_audio`, `retime_clip`, `stabilize_video`, `enhance_speech` | Video effects, LUTs, audio volume automation curves, and speed ramping. |
| **Generation & AI** (3) | `generate_speech`, `generate_image`, `generate_music` | Integrations with local or cloud generative backends. |

### 4.3 Invariant: Symmetric Undo History
Per `AGENTS.md`:
> *"Route UI and Agent edits through the same domain mutation operations and shared EditorUndo history. One coherent user intent should produce one undoable action."*

Every tool that modifies timeline tracks, clips, transforms, or keyframes must execute via `TimelineEditor` mutation commands. An undo performed by the user in the UI must revert an agent edit seamlessly, and the `undo` tool invoked by an agent must revert recent actions identically.

---

## 5. Step-by-Step Implementation Instructions for Phase 8.0

1. **Configure `clawvinci-mcp/Cargo.toml`**:
   - Add dependencies: `clawvinci-model`, `clawvinci-timeline`, `clawvinci-media`, `clawvinci-render`, `clawvinci-export`.
   - Add async web framework / HTTP listener: `axum` or `actix-web` with `tokio` for the lightweight HTTP MCP server.
   - Add serde, serde_json, schemars (for JSON schema generation), tracing, uuid, chrono.
2. **Implement Core Protocol & Types in `clawvinci-mcp/src/`**:
   - `protocol.rs`: JSON-RPC 2.0 types, MCP message models (`Initialize`, `CallTool`, `ListTools`).
   - `error.rs`: `McpError` handling with standard JSON-RPC codes.
   - `server.rs`: Lightweight Axum HTTP server hosting `GET /mcp` (SSE stream) and `POST /mcp` (tool execution).
   - `executor.rs`: Central `ToolExecutor` dispatching requests to handler functions.
3. **Implement Tool Handlers**:
   - `tools/project.rs`: Project management, active timeline, markers, settings.
   - `tools/clips.rs`: Clip manipulation, tracks, ripple delete, split, move, transform.
   - `tools/media.rs`: Manifest inspection, media search, import.
   - `tools/text.rs`: Subtitles, kinetic titles, styling.
   - `tools/effects.rs`: Color grading, filters, LUTs.
4. **Implement In-App Agent Chat**:
   - `chat/service.rs`: Agent session manager, conversational state, multi-turn tool loops.
   - `chat/client.rs`: HTTP client for Anthropic Claude messages API with streaming support.
5. **Wire Tauri IPC & State**:
   - Register MCP server background startup on app launch (bound to `127.0.0.1:19789`).
   - Add IPC commands for in-app chat: `agent_chat_send`, `agent_chat_history`, `agent_chat_clear`.
6. **Remote Verification**:
   - Commit and push to `worktree-plan-windows-port`.
   - Monitor GitHub Actions CI run via `gh run watch` to confirm zero clippy warnings and passing tests.
   - Update `.agents/HISTORY/8-0-mcp-agent-layer.md` and `.agents/HISTORY.md`.

---

## 6. Developer Commands Quick Reference

All verification must run remotely via GitHub Actions:

```powershell
# View recent runs
gh run list --limit 3

# Watch active run
gh run watch <RUN_ID>

# Inspect failures if any step fails
gh run view <RUN_ID> --log-failed
```
