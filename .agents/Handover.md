# Clawvinci Project Handover & Implementation Guide

> **Document Purpose**: Authoritative orientation and implementation manual for the incoming AI agent / engineering team.  
> **Workspace Root**: `d:\projects\palmier-win\.claude\worktrees\plan-windows-port`  
> **Current Git Branch**: `worktree-plan-windows-port`  
> **Last Clean Commit**: `d41711f`  
> **CI Verification Status**: ✅ **100% Green** (GitHub Actions Run `34219245633`, Artifact `clawvinci-windows-x64`)  
> **Completed Milestones**: **Phases 0.0 through 13.0 — Complete native Windows Port of Palmier Pro** (Full feature parity, zero telemetry, strictly BYOK/local, embedded 53-tool MCP server, 28-locale i18n, persistent settings, Home hub, onboarding, Windows-native shortcuts and setup guides)

---

## 1. Executive Summary & Project Mission

**Clawvinci** is the native Windows port of **Palmier Pro** — an AI-native desktop non-linear video editor (NLE) featuring an embedded Model Context Protocol (MCP) server that empowers AI agents to edit timeline sequences directly alongside human editors.

### Product Mental Model: "The Cursor IDE of Video Editing"
A foundational mental model for Clawvinci is that it is **"Cursor IDE" but for Video Editing**:
- **In Software Engineering**: Modern AI-first editors (like Cursor or Windsurf) bridge human developers and AI models by granting agents direct, tool-driven manipulation of the workspace—reading files, analyzing ASTs, generating surgical diffs, and inspecting compiler feedback.
- **In Video Editing**: Clawvinci bridges human video creators and AI agents by granting models direct, tool-driven manipulation of the NLE timeline—reading media manifests, inspecting audio waveforms, executing frame-accurate ripple/overwrite edits, adjusting color grades, tuning transforms, and generating kinetic text via the Model Context Protocol (MCP).
- **Symmetric Collaboration**: Human editor and AI agent share the exact same domain state, see the exact same canvas, and operate through a unified, frame-accurate undo/redo history.

### Core Technology Stack
- **Backend (Rust Workspace)**: `windows/src-tauri/crates/*` managing media decode/encode, timeline state, undo/redo history, frame compositing, export, search/ML, audio DSP, MCP protocol execution, and generative AI services.
- **Frontend Shell (Tauri v2 + Web)**: `windows/src/*` hosting a high-performance web frontend with vanilla HTML5/CSS/JavaScript and Canvas-based timeline rendering.
- **Target Platform**: Windows 10 22H2+ (x86_64).
- **Licensing**: GPL-3.0-only (mirroring original Palmier Pro).
- **Single Source of Truth**: The `.agents/` directory is the authoritative repository of decisions, plans, rules, and history.

---

## 2. Inviolable Engineering Rules & Invariants

All incoming agents working on this codebase **must strictly abide by these rules**:

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

## 3. Current Implementation Status (Phases 0.0 – 13.0 Complete)

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
| **8.0** | `clawvinci-mcp` | 52-tool MCP execution engine, embedded HTTP/SSE MCP server on `127.0.0.1:19789/mcp`, in-app agent chat orchestration. | ✅ CI Green (`8-0-mcp-agent-layer.md`, Run `34151682590`) |
| **9.0** | `clawvinci-audio` | Audio analysis engine: Envelopes, real-time metering, cross-correlation sync, silence/dead-air planner, beat/tempo detector, VAD. | ✅ CI Green (`9-0-audio-analysis.md`, Run `34192487829`) |
| **10.0** | `clawvinci-search` | Semantic visual search (SigLIP2 / `PALMEMB1` binary embeddings), transcript search, word cut planner. | ✅ CI Green (`10-0-search-transcription.md`, Run `34195654610`) |
| **11.0** | `clawvinci-gen` | Generative AI provider catalog, submissions, edit clients, preprocessing, timeline insertion, and MCP tools. | ✅ CI Green (`11-0-generative-ai.md`, Run `34208682487`) |
| **12.0** | `clawvinci` (Cleanup) | **Decision 10 Compliance**: Purged `api.palmier.io`, OpenAI BYOK Whisper + local transcription engine, ByokGenerationBackend, Tauri v2 updater. | ✅ CI Green (`12-0-auth-backend-telemetry-updater.md`, Run `34216896609`) |
| **13.0** | `clawvinci` (Tauri + Web) | **Phase 13.0 Polish**: 28-locale i18n catalog, persistent settings & storage backend, BYOK models pane, Home hub & onboarding, Windows shortcuts & MCP setup guide. | ✅ **100% CI Green** (`13-0-polish.md`, Run `34219245633`) |

---

## 4. Architecture & Subsystem Summary (100% Completed)

Clawvinci represents a complete, ground-up rewrite of Palmier Pro for Windows 10/11:

1. **Domain Model (`clawvinci-model`)**:
   - 21 domain models matching Palmier Pro (`Timeline`, `Track`, `Clip`, `Timecode`, `ProjectFile`).
   - Byte-compatible `.palmier` package serialization + legacy format fallback.
2. **Media Engine (`clawvinci-media`)**:
   - High-performance FFmpeg-backed probe, decode, encode, and waveform generation.
   - Bounded concurrency with safe memory limits.
3. **Timeline Editing Core (`clawvinci-timeline`)**:
   - `TimelineEditor` with symmetrical command pattern for undo/redo history.
   - Ripple, overwrite, split, trim, and slide editing primitives.
4. **Playback & Rendering (`clawvinci-render`)**:
   - `FramePlan` builder, shared CPU compositor with blend modes and transforms.
   - Real-time `PlaybackEngine` clock synchronization and frame stepping.
5. **Effects Pipeline (`clawvinci-render`)**:
   - 12 GPU/CPU shader kernels ported from Metal shaders to Rust.
   - `EffectRegistry`, 3D `.cube` LUT parser with tetrahedral interpolation, `fontdue` kinetic text animator.
6. **UI Shell (`windows/src` & Tauri v2)**:
   - Modern dark design system (`AppTheme`), Canvas-based multi-track timeline, inspector, media panel.
7. **Export Engine (`clawvinci-export`)**:
   - Render-to-file, FCPXML 1.10–1.14 export, Premiere XMEML 4 export, self-contained `.palmier` bundle export, export queue.
8. **Embedded MCP Server (`clawvinci-mcp`)**:
   - Embedded HTTP/SSE server on `127.0.0.1:19789/mcp`.
   - 53 registered tools allowing external AI agents (Claude Desktop, Cursor, Codex) to read and edit the timeline directly.
   - In-app agent chat orchestration.
9. **Audio Analysis Engine (`clawvinci-audio`)**:
   - Real-time peak/RMS metering, cross-correlation audio sync, silence/dead-air detection, tempo/beat detector, VAD.
10. **Semantic Search & Transcription (`clawvinci-search`)**:
    - SigLIP2 visual search with `PALMEMB1` binary vector store, transcript search, cut planner.
11. **Generative AI Integration (`clawvinci-gen`)**:
    - Direct BYOK integration for OpenAI, Kling, Seedance, Fal, ElevenLabs, Suno; timeline insertion and preprocessing.
12. **BYOK & Privacy Invariant (Decision 10)**:
    - Zero telemetry, no cloud backend, no account/Clerk dependency. On-device local Whisper & BYOK OpenAI Whisper.
13. **Polish & Desktop Experience (Phase 13.0)**:
    - 28-locale client-side i18n system (`windows/src/i18n.js`).
    - Persistent settings (`%APPDATA%/Clawvinci/settings.json`) & storage cache management.
    - Home project hub with sample templates & 4-step onboarding wizard.
    - Windows-native shortcuts & 1-click MCP setup guide for Claude Desktop, Cursor, Claude Code, and Codex.

---

## 5. Build, Packaging & CI Operations

All compilation, linting, and testing are performed remotely via GitHub Actions:

```powershell
# 1. Trigger remote CI via Git Push
git add windows/ .agents/
git commit -m "feat: description"
git push origin worktree-plan-windows-port

# 2. Monitor CI Run
gh run list --limit 3
gh run watch <RUN_ID>

# 3. Inspect Artifacts
gh run view <RUN_ID>
```

Every CI run automatically compiles the Tauri application, executes all workspace tests, runs `cargo clippy --workspace -- -D warnings`, and packages `clawvinci-windows-x64.zip`.

---

## 6. Critical Gotchas & Repository Conventions Learned

Key conventions learned from Phases 0–10 that **must** be preserved:

- **Clippy on Rust 1.80+**:
  - Never write manual integer ceilings `(x + k - 1) / k` — use `x.div_ceil(k)` (`clippy::manual_div_ceil`).
  - Never iterate ranges to index slices (`for i in 0..len { arr[i] = ... }`) — use `arr.fill(...)`, `for item in &mut arr`, or `.iter().zip(...)` (`clippy::needless_range_loop`).
  - Use `std::slice::from_ref(item)` instead of `&[item.clone()]` (`clippy::cloned_ref_to_slice_refs`).
  - Use `.as_chunks::<N>().0.iter()` instead of `chunks_exact(N)` (`clippy::chunks_exact_to_as_chunks`).
- **Timeline Access**:
  - `Timeline` does **not** have a `.duration_frames()` or `.all_clips()` method.
  - To get duration frames: use `timeline.total_frames()`.
  - To get all clips: use `timeline.tracks.iter().flat_map(|t| t.clips.iter())`.
- **Media File Hashing & Caching**:
  - File cache keys use SHA256 over `"{path}|{mtime}|{size}"`, taking the first 32 characters of hex digest.
- **PowerShell Command Chaining**:
  - PowerShell on Windows does not support `&&`. Always use `;` (e.g. `git add . ; git commit -m "..." ; git push`).
- **Test Dependencies**:
  - Always add test-only crates (like `uuid` in tests) under `[dev-dependencies]` in `Cargo.toml`.

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
