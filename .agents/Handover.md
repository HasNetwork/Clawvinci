# Clawvinci Project Handover & Implementation Guide

> **Document Purpose**: Authoritative orientation and implementation manual for the incoming AI agent / engineering team.  
> **Workspace Root**: `d:\projects\palmier-win\.claude\worktrees\plan-windows-port`  
> **Current Git Branch**: `worktree-plan-windows-port`  
> **Last Clean Commit**: `9c24e86`  
> **CI Verification Status**: ✅ **100% Green** on GitHub Actions Run `34150452889` (Check, Clippy, Test, Tauri Build & Artifact Upload)  
> **Active Target Milestone**: **Phase 9.0 — Audio Engine (`clawvinci-audio`)**

---

## 1. Executive Summary & Project Mission

**Clawvinci** is the native Windows port of **Palmier Pro** — an AI-native desktop non-linear video editor (NLE) featuring an embedded Model Context Protocol (MCP) server that empowers AI agents to edit timeline sequences directly alongside human editors.

### Product Mental Model: "The Cursor IDE of Video Editing"
A foundational mental model for Clawvinci is that it is **"Cursor IDE" but for Video Editing**:
- **In Software Engineering**: Modern AI-first editors (like Cursor or Windsurf) bridge human developers and AI models by granting agents direct, tool-driven manipulation of the workspace—reading files, analyzing ASTs, generating surgical diffs, and inspecting compiler feedback.
- **In Video Editing**: Clawvinci bridges human video creators and AI agents by granting models direct, tool-driven manipulation of the NLE timeline—reading media manifests, inspecting audio waveforms, executing frame-accurate ripple/overwrite edits, adjusting color grades, tuning transforms, and generating kinetic text via the Model Context Protocol (MCP).
- **Symmetric Collaboration**: Human editor and AI agent share the exact same domain state, see the exact same canvas, and operate through a unified, frame-accurate undo/redo history.

### Core Technology Stack
- **Backend (Rust Workspace)**: `windows/src-tauri/crates/*` managing media decode/encode, timeline state, undo/redo history, frame compositing, export, and MCP protocol execution.
- **Frontend Shell (Tauri v2 + Web)**: `windows/src/*` hosting a high-performance web frontend with vanilla HTML5/CSS/JavaScript and Canvas-based timeline rendering.
- **Target Platform**: Windows 10 22H2+ (x86_64).
- **Licensing**: GPL-3.0-only (mirroring original Palmier Pro).
- **Single Source of Truth**: The `.agents/` directory is the authoritative repository of decisions, plans, rules, and history.

---

## 2. Inviolable Engineering Rules & Invariants

All future agents working on this codebase **must strictly abide by these rules**:

1. **CRITICAL: Remote CI Verification Only (No Local Toolchain)**:
   - **Never install or run `cargo`, `rustc`, or `npm` commands locally on the host machine.** Developer tools are not installed on the user's system.
   - All compilation, linting, unit testing, and packaging must run remotely on **GitHub Actions CI** (`.github/workflows/ci.yml`).
   - Push code to `worktree-plan-windows-port` and monitor CI using GitHub CLI:
     ```powershell
     gh run list --limit 3
     gh run watch <RUN_ID>
     gh run view <RUN_ID> --log-failed
     ```
2. **`Sources/` is Read-Only Reference**:
   - The original macOS Swift source code at the repository root (`Sources/`, `Metal/`, `Plugins/`, `models/`, `scripts/`, `mcpb/`) is strictly **read-only GPLv3 reference**.
   - **Never edit, move, or delete files outside of `windows/` and `.agents/`**.
3. **Unified Render Path (Do Not Fork Compositing)**:
   - Both live timeline preview and batch export **must share the exact same compositing pipeline**: `clawvinci_render::compositor::composite_frame` and `CompositionBuilder::build_frame_plan`.
   - Never write a second, standalone compositor for export, thumbnail generation, or frame capture.
4. **Integer Frame Timing**:
   - All timeline positions, durations, in/out points, and trims are integer frame counts (`i64` / `u64`). Floating-point time (`f64`) is converted only at external display boundaries.
5. **Main Thread & UI Isolation**:
   - The Tauri main/UI thread must remain completely unblocked. All file I/O, FFmpeg probing/decoding/encoding, image processing, and heavy transforms must execute in background tasks (`tokio::task::spawn_blocking` or dedicated worker threads).
6. **Cooperative Cancellation**:
   - All asynchronous pipelines (preview rendering, export jobs, waveform extractions, MCP requests) must carry and check a `tokio_util::sync::CancellationToken` at frame boundaries to abort immediately without leaking resources or orphaned files.
7. **Symmetric Undo History**:
   - Route UI and Agent edits through the same domain mutation operations (`clawvinci_timeline::TimelineEditor`) and shared undo history. One coherent user or agent intent produces one undoable action.
8. **Zero-Warning Tolerance**:
   - All code must pass `cargo clippy --workspace -- -D warnings` with zero warnings on CI.

---

## 3. Current Implementation Status (Phases 0 – 8 Complete)

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
| **8.0** | `clawvinci-mcp` | 52-tool MCP execution engine, embedded HTTP/SSE MCP server on `127.0.0.1:19789/mcp`, in-app agent chat orchestration. | ✅ CI Green (`8-0-mcp-agent-layer.md`, Run `34150452889`) |
| **9.0** | `clawvinci-audio` | **CURRENT TARGET**: Multi-track audio mixer, stem separation, ducking, meter bridge, Web Audio/WASAPI pipeline. | 🚀 **Ready for Implementation** (`9-0-audio-analysis.md`) |
| **10.0** | `clawvinci-search` | Semantic search, local transcription / Whisper integration. | 📋 Planned (`10-0-search-transcription-ml.md`) |
| **11.0** | `clawvinci-gen` | Generative AI integration (speech synthesis, image generation, music). | 📋 Planned (`11-0-generative-ai.md`) |
| **12.0** | `clawvinci` (Tauri) | Auth, backend telemetry, auto-updater. | 📋 Planned (`12-0-auth-backend-telemetry-updater.md`) |
| **13.0** | Entire app | Production polish, performance profiling, final documentation. | 📋 Planned (`13-0-polish.md`) |

---

## 4. Phase 8.0 — MCP Agent Layer Detailed Execution Blueprint

The incoming agent's primary assignment is to implement **Phase 8.0 (`clawvinci-mcp`)**.

### 4.1 Two Distinct Components Sharing One Tool Executor
Do not conflate these two components — they serve different roles but execute the **exact same 53 tools**:

1. **Embedded HTTP MCP Server** (`Sources/PalmierPro/Agent/MCP/`):
   - Listens on `http://127.0.0.1:19789/mcp`.
   - Exposes standard Model Context Protocol (JSON-RPC 2.0 over HTTP with SSE streaming).
   - Allows external agents (Claude Code, Cursor, Codex, Claude Desktop) to connect and edit the active project:
     ```bash
     claude mcp add --transport http clawvinci http://127.0.0.1:19789/mcp
     ```
2. **In-App Agent Chat Orchestrator** (`Sources/PalmierPro/Agent/Chat/` and `Agent/Clients/`):
   - Embedded chat panel inside the Tauri app (`windows/src/index.html`).
   - Connects directly to LLM providers (Anthropic Claude Messages API with streaming, OpenAI, BYOK).
   - Manages message history, streaming token presentation buffer, system prompts (`AgentInstructions.swift`), and multi-turn tool calling loops.

Both components delegate every action to a shared **`ToolExecutor`** that mutates the active `TimelineEditor` through undoable domain commands.

---

### 4.2 Source Inventory Reference
Reference source directory: `Sources/PalmierPro/Agent/` (21,500+ LOC across 35 files):

| Swift Source File | LOC | Role in Palmier Pro | Target in `clawvinci-mcp` |
|---|---|---|---|
| `Tools/ToolDefinitions.swift` | 3,100 | Enum `ToolName` (53 cases) + JSON schema definitions for all tool arguments. | `src/protocol.rs` & `src/tools/mod.rs` |
| `Tools/ToolExecutor.swift` | 580 | Central dispatcher routing tool calls to handlers and wrapping receipts. | `src/executor.rs` |
| `Tools/ToolResult.swift` | 75 | Structured tool response envelope (`success`, `data`, `warnings`, `error`). | `src/result.rs` |
| `Tools/AgentInstructions.swift` | 420 | System prompt defining filmmaker agent persona and tool rules. | `src/instructions.rs` |
| `Tools/ToolExecutor+Projects.swift` | 240 | `manage_project`, `create_timeline`, `set_active_timeline`. | `src/tools/project.rs` |
| `Tools/ToolExecutor+Timeline.swift` | 680 | `get_timeline`, `set_project_settings`. | `src/tools/timeline.rs` |
| `Tools/ToolExecutor+InspectTimeline.swift` | 140 | `inspect_timeline` summary analysis. | `src/tools/inspect.rs` |
| `Tools/ToolExecutor+Markers.swift` | 110 | `manage_markers`. | `src/tools/markers.rs` |
| `Tools/ToolExecutor+Export.swift` | 390 | `export_project`, `manage_exports` (calls `clawvinci-export`). | `src/tools/export.rs` |
| `Tools/ToolExecutor+Media.swift` | 480 | `get_media`, `inspect_media`. | `src/tools/media.rs` |
| `Tools/ToolExecutor+Search.swift` | 130 | `search_media`. | `src/tools/search.rs` |
| `Tools/ToolExecutor+Import.swift` | 410 | `import_media`, `organize_media`. | `src/tools/import.rs` |
| `Tools/ToolExecutor+CaptureFrame.swift` | 80 | `capture_frame` (renders frame via `clawvinci-render`). | `src/tools/capture.rs` |
| `Tools/ToolExecutor+Clips.swift` | 1,650 | `manage_tracks`, `manage_clip_links`, `add_clips`, `insert_clips`, `move_clips`, `remove_clips`, `split_clips`, `ripple_delete_ranges`, `swap_clip_media`, `set_clip_properties`, `copy_clip_settings`, `set_keyframes`, `undo`. | `src/tools/clips.rs` |
| `Tools/ToolExecutor+Layout.swift` | 260 | `apply_layout` (grid, picture-in-picture, split screen). | `src/tools/layout.rs` |
| `Tools/ToolExecutor+Sync.swift` | 90 | `sync_clips`. | `src/tools/sync.rs` |
| `Tools/ToolExecutor+Multicam.swift` | 290 | `manage_multicam`, `change_cam`, `get_multicam`. | `src/tools/multicam.rs` |
| `Tools/ToolExecutor+Texts.swift` | 780 | `add_texts`, `update_text`. | `src/tools/text.rs` |
| `Tools/ToolExecutor+Captions.swift` | 170 | `add_captions`. | `src/tools/captions.rs` |
| `Tools/ToolExecutor+Color.swift` | 720 | `apply_color`, `set_color_grade`. | `src/tools/color.rs` |
| `Tools/ToolExecutor+Effect.swift` | 100 | `apply_effect`, `remove_effect`. | `src/tools/effects.rs` |
| `Tools/ToolExecutor+Denoise.swift` | 45 | `denoise_audio`. | `src/tools/audio_fx.rs` |
| `Tools/ToolExecutor+Transcription.swift` | 640 | `get_transcript`. | `src/tools/transcription.rs` |
| `Tools/ToolExecutor+Words.swift` | 330 | `remove_words`, `remove_silence`. | `src/tools/words.rs` |
| `Tools/ToolExecutor+Beats.swift` | 75 | `detect_beats`. | `src/tools/beats.rs` |
| `Tools/ToolExecutor+Generate.swift` | 860 | `generate_speech`, `generate_image`, `generate_music`. | `src/tools/generate.rs` |
| `MCP/MCPHTTPServer.swift` | 312 | Lightweight HTTP server listening on port 19789 with SSE support. | `src/server.rs` |
| `Chat/AgentService.swift` | 760 | Multi-turn chat orchestration loop, tool calling, context windowing. | `src/chat/service.rs` |
| `Clients/AnthropicProvider.swift` | 210 | Streaming client for Anthropic Claude Messages API. | `src/chat/anthropic.rs` |

---

### 4.3 Complete 53-Tool Catalog by Category

All 53 tools must be exposed with their standard parameters and return structured receipts:

#### 1. Projects & Timelines (9 tools)
1. `manage_project`: Create/open/save/close projects, query active state.
2. `get_timeline`: Return full or summarized JSON structure of the active timeline.
3. `inspect_timeline`: Diagnostic check (detect gaps, overlapping audio, missing media, offline clips).
4. `create_timeline`: Create a child or sibling timeline with custom resolution and fps.
5. `set_active_timeline`: Switch the current working timeline.
6. `manage_markers`: Add, update, delete, or seek timeline markers with labels and colors.
7. `set_project_settings`: Modify resolution, frame rate, timecode base, color space.
8. `export_project`: Enqueue batch render or interchange export (calls `clawvinci-export`).
9. `manage_exports`: Check status, list active jobs, or cancel export jobs in `ExportQueue`.

#### 2. Media Library (6 tools)
10. `get_media`: Enumerate assets in `MediaManifest` with durations, codecs, resolutions.
11. `inspect_media`: Deep probe of a single asset (streams, channel layout, sample rate).
12. `search_media`: Filter media assets by name, tag, transcript keyword, or duration.
13. `import_media`: Register external media file into project manifest.
14. `capture_frame`: Render and export a single frame snapshot as PNG/JPEG.
15. `organize_media`: Move assets into virtual bins / folders in `MediaManifest`.

#### 3. Clips & Timeline Editing (15 tools)
16. `manage_tracks`: Add, remove, mute, lock, reorder, or rename video/audio tracks.
17. `manage_clip_links`: Link or unlink audio and video clips.
18. `add_clips`: Append or insert clips onto tracks at specified frame positions.
19. `insert_clips`: Three-point / four-point ripple insert pushing downstream clips.
20. `move_clips`: Frame-accurate move or track change with collision detection.
21. `remove_clips`: Lift edit (deletes clip, leaves gap) or ripple delete.
22. `split_clips`: Razor/slice clip at a specific frame timestamp into two segments.
23. `ripple_delete_ranges`: Batch ripple delete frame intervals across all unlocked tracks.
24. `swap_clip_media`: Replace media reference of a clip while preserving cuts and effects.
25. `set_clip_properties`: Adjust volume, speed, opacity, blend mode, fade-in, fade-out.
26. `copy_clip_settings`: Copy effects/grading from one clip to another.
27. `set_keyframes`: Add or update keyframes on opacity, position, scale, rotation, crop, volume.
28. `apply_layout`: Arrange clips into standard screen splits (PiP, 2-up, 4-up grid).
29. `sync_clips`: Align audio/video clips by timecode or audio waveform peak.
30. `undo`: Revert the last user or agent edit action.

#### 4. Multicam (3 tools)
31. `manage_multicam`: Create or dissolve multicam sync groups.
32. `change_cam`: Switch active camera angle on an edit cut point.
33. `get_multicam`: Inspect available camera angles and sync status.

#### 5. Transcript & Speech (4 tools)
34. `get_transcript`: Retrieve word-level timecoded transcript for media or timeline.
35. `remove_words`: Ripple delete specific word time ranges (filler word removal).
36. `remove_silence`: Automatically cut silent pauses exceeding a duration threshold.
37. `detect_beats`: Mark musical rhythmic beats as timeline markers.

#### 6. Text & Titles (3 tools)
38. `add_texts`: Add subtitle, lower-third, or title clips with styling and placement.
39. `update_text`: Edit text content, typography, font name, size, animation preset.
40. `add_captions`: Generate and burn-in synchronized subtitle cards.

#### 7. Color & Effects (10 tools)
41. `apply_color`: Adjust exposure, contrast, saturation, temperature, tint, shadows, highlights.
42. `apply_effect`: Add shader effects (Gaussian Blur, Sharpen, Vignette, Glow, Edge Detect, Hue Shift, Invert, Grain, Halftone, Bloom, Chromatic Aberration).
43. `remove_effect`: Delete an applied effect by ID.
44. `update_effect`: Adjust effect parameter uniforms.
45. `denoise_audio`: Apply noise reduction filter on audio clips.
46. `adjust_audio_levels`: Normalize dialog loudness (LUFS) or set peak gain.
47. `duck_audio`: Automatically reduce music volume under spoken dialog tracks.
48. `retime_clip`: Set speed percentage, optical flow remapping, or reverse playback.
49. `stabilize_video`: Trigger camera shake stabilization analysis.
50. `enhance_speech`: Apply speech isolation / voice clarity processing.

#### 8. Generation & AI (3 tools)
51. `generate_speech`: Synthesize AI voiceover narration from text.
52. `generate_image`: Generate AI background / asset image and place on timeline.
53. `generate_music`: Generate instrumental soundtrack matching target duration.

---

### 4.4 HTTP MCP Server Protocol Architecture
The MCP server must implement the standard Model Context Protocol:
- **Transport**: JSON-RPC 2.0 over HTTP (POST `/mcp`) and Server-Sent Events (GET `/mcp`).
- **Port**: Default `127.0.0.1:19789`.
- **Supported Methods**:
  - `initialize`: Returns server capabilities, name (`clawvinci`), version (`0.1.0`), and instructions.
  - `tools/list`: Returns JSON schemas for all 53 tools.
  - `tools/call`: Executes a tool by name with arguments and returns structured receipts.

---

## 5. Step-by-Step Implementation Sequence for Phase 8.0

1. **Crate Dependencies in `windows/src-tauri/crates/clawvinci-mcp/Cargo.toml`**:
   ```toml
   [dependencies]
   clawvinci-model = { path = "../clawvinci-model" }
   clawvinci-timeline = { path = "../clawvinci-timeline" }
   clawvinci-media = { path = "../clawvinci-media" }
   clawvinci-render = { path = "../clawvinci-render" }
   clawvinci-export = { path = "../clawvinci-export" }
   serde.workspace = true
   serde_json.workspace = true
   thiserror.workspace = true
   tokio.workspace = true
   tokio-util.workspace = true
   tracing.workspace = true
   uuid.workspace = true
   chrono.workspace = true
   axum = { version = "0.7", features = ["tokio"] }
   tower-http = { version = "0.5", features = ["cors", "trace"] }
   reqwest = { version = "0.12", features = ["json", "stream"] }
   ```
2. **Implement Core Protocol Models (`src/protocol.rs`)**:
   - Define `JsonRpcRequest`, `JsonRpcResponse`, `McpToolDefinition`, `ToolCallRequest`.
   - Port all 53 tool parameter JSON schemas from `Sources/PalmierPro/Agent/Tools/ToolDefinitions.swift`.
3. **Implement Central Dispatcher (`src/executor.rs`)**:
   - `ToolExecutor::execute(tool_name, arguments, &mut app_state) -> ToolResult`.
   - Every mutating tool invokes methods on `clawvinci_timeline::TimelineEditor`.
4. **Implement Functional Tool Handlers (`src/tools/*`)**:
   - Group implementations cleanly into `project.rs`, `media.rs`, `clips.rs`, `text.rs`, `effects.rs`, etc.
5. **Implement Lightweight HTTP Server (`src/server.rs`)**:
   - Bind `axum::Router` on `127.0.0.1:19789` with POST `/mcp` and GET `/mcp`.
6. **Implement In-App Agent Chat (`src/chat/*`)**:
   - `service.rs`: Chat message session store and orchestrator.
   - `anthropic.rs`: Streaming client for Anthropic Messages API.
7. **Wire Tauri IPC Commands in `windows/src-tauri/src/lib.rs`**:
   - Start MCP server background task on app startup.
   - Expose Tauri IPC commands: `agent_chat_send`, `agent_chat_history`, `agent_chat_clear`, `mcp_server_status`.
8. **Add Tests in `clawvinci-mcp/tests/`**:
   - Protocol serialization/deserialization tests.
   - Tool execution tests for clips, markers, text, and undo.
   - HTTP server endpoint tests.
9. **Verify Remotely via GitHub Actions CI**:
   - Push commit to `worktree-plan-windows-port`.
   - Run `gh run watch` to ensure zero clippy warnings and passing tests.
   - Record completion log in `.agents/HISTORY/8-0-mcp-agent-layer.md` and `.agents/HISTORY.md`.

---

## 6. Critical Gotchas & Repository Conventions Learned

- **`Timeline` Duration**: `Timeline` in `clawvinci-model` does **not** have a `.duration()` method. Always use `timeline.total_frames()` or `timeline.display_frames()`.
- **`MediaSource::External`**: The field is `absolute_path: String`, not `path`.
- **`ProjectFile` Construction**: Use `ProjectFile::new(vec![timeline])` (there is no `from_timeline`).
- **`TextStyle` Font Name**: Field is `font_name: String`, not `font_family`.
- **Clippy `too_many_arguments`**: Tauri commands taking >7 arguments trigger clippy errors. Always wrap multiple arguments into dedicated DTO structs.
- **Clippy `derivable_impls`**: Do not write manual `impl Default for Enum { fn default() -> Self { ... } }`. Use `#[derive(Default)]` with `#[default]` on the default variant.
- **Mutex Locks Across Await**: Do not hold `tokio::sync::Mutex` locks across long `.await` calls or HTTP streaming requests. Snapshot required state or pass atomic references.
- **PowerShell Syntax**: Use `;` rather than `&&` when chaining shell commands.

---

## 7. Developer Quick Reference

All remote verification commands:

```powershell
# Check recent CI workflow runs
gh run list --limit 3

# Watch active CI run
gh run watch <RUN_ID>

# View failure log if any step fails
gh run view <RUN_ID> --log-failed

# View full job steps
gh run view --job=<JOB_ID>
```
