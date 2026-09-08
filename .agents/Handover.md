# Clawvinci Project Handover & Implementation Guide

> **Document Purpose**: Authoritative orientation and implementation manual for the incoming AI agent / engineering team.  
> **Workspace Root**: `d:\projects\palmier-win\.claude\worktrees\plan-windows-port`  
> **Current Git Branch**: `worktree-plan-windows-port`  
> **Last Clean Commit**: `fb7402c`  
> **CI Verification Status**: ✅ **100% Green** on GitHub Actions Run `34208682487` (Check, Clippy, Test, Tauri Build & Artifact Upload)  
> **Active Target Milestone**: **Phase 13.0 — Polish (localization, settings panes, home/onboarding, in-app help)**  
> **Completed Milestone**: **Phase 12.0 — BYOK/local cleanup + Auto-Updater** (Decision 10: zero telemetry, BYOK Whisper + local on-device ML, ByokGenerationBackend, Tauri v2 updater)

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

## 3. Current Implementation Status (Phases 0 – 11 Complete)

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
| **12.0** | `clawvinci-search` (fix), `clawvinci-gen` (verify), `clawvinci` (Tauri) | **Phase 12.0 Completed (Decision 10)**: no auth/backend/telemetry. Fixed Phase 10's transcription backend (removed `api.palmier.io` → BYOK + local on-device), ByokGenerationBackend direct-to-provider, added Tauri v2 auto-updater. | ✅ **Implemented & CI Verified** (`12-0-auth-backend-telemetry-updater.md`) |
| **13.0** | Entire app | **ACTIVE HANDOVER TARGET**: Production polish, performance profiling, final documentation. | 🚀 **Ready for Implementation** (`13-0-polish.md`) |

---

## 4. Phase 11.0 — Generative AI Integration (`clawvinci-gen`) Detailed Execution Blueprint

The incoming team's assignment is to implement **Phase 11.0 (`clawvinci-gen`)**.

### 4.1 Architecture & Mental Model
Unlike Phases 9 and 10 which involved on-device DSP and vector search, **Phase 11.0 is primarily a REST API client, catalog, and preprocessing engine**:
1. **No local ML model inference**: All generation providers (Seedance, Kling, Nano Banana Pro, Suno, ElevenLabs, etc.) are remote cloud services.
2. **Preprocessing Pipeline**: Prepares source video/audio from the timeline before submitting to generative APIs (trimming, format conversion, audio extraction) using `clawvinci-media`.
3. **Two-Way Timeline Integration**:
   - When generation starts: creates placeholder `MediaAsset` items with status `Generating` and inserts placeholder clips on the timeline.
   - When generation finishes: downloads generated media into the project's `.palmier` bundle, probes duration/resolution via `clawvinci-media`, updates asset status to `Ready`, and replaces or inserts clips on the timeline.
4. **Agent MCP Integration**: Exposes generation to external agents via `generate_speech`, `generate_image`, and `generate_music`.

---

### 4.2 Source Inventory Reference
Reference source directory: `Sources/PalmierPro/Generation/` (6,880 LOC across 36 files):

| Swift Source Directory / File | LOC | Role in Palmier Pro | Target in `clawvinci-gen` |
|---|---|---|---|
| `GenerationService.swift` | 743 | Top-level job lifecycle orchestrator (placeholders, reference prep, submission, polling, timeline insertion). | `src/service.rs` |
| `GenerationBackend.swift` | 120 | Backend API client for generation jobs (Convex actions / REST endpoints). | `src/backend.rs` |
| `Catalog/ModelCatalog.swift` | 328 | Central model registry (`ModelKind`: Video, Image, Audio, Upscale). | `src/catalog/models.rs` |
| `Catalog/VideoModelConfig.swift` | 248 | Video generation model configs, supported aspect ratios, durations. | `src/catalog/video.rs` |
| `Catalog/ImageModelConfig.swift` | 135 | Image generation model configs, resolutions, aspect ratios. | `src/catalog/image.rs` |
| `Catalog/AudioModelConfig.swift` | 191 | Speech & voice generation configs (voices, styles, sample rates). | `src/catalog/audio.rs` |
| `Catalog/UpscaleModelConfig.swift` | 145 | Video upscaling / enhancement model configs. | `src/catalog/upscale.rs` |
| `Catalog/CostEstimator.swift` | 217 | Credit cost estimation per model, duration, quality, and resolution. | `src/catalog/cost.rs` |
| `Catalog/ModelPreferences.swift` | 40 | User default model preferences per modality. | `src/catalog/preferences.rs` |
| `Submission/VideoGenerationSubmission.swift` | 429 | Constructs provider-specific JSON payloads for video models. | `src/submission/video.rs` |
| `Submission/ImageGenerationSubmission.swift` | 75 | Constructs payloads for image generation models. | `src/submission/image.rs` |
| `Submission/AudioGenerationSubmission.swift` | 162 | Constructs payloads for speech synthesis / TTS models. | `src/submission/audio.rs` |
| `Submission/MusicGenerationSubmission.swift` | 95 | Constructs payloads for music / instrumental generation models. | `src/submission/music.rs` |
| `Edit/EditAction.swift` & `EditSubmitter*.swift` | ~700 | AI-driven timeline transforms (upscaling, re-voicing, audio transformation). | `src/edit/mod.rs` |
| `Preprocessing/VideoPreprocessor.swift` | 160 | Coordinates video source prep (resizing, codec compatibility). | `src/preprocessing/video.rs` |
| `Preprocessing/VideoTrimExtractor.swift` | 110 | Trims source clip ranges via FFmpeg for generation references. | `src/preprocessing/trim.rs` |
| `Preprocessing/AudioTrackExtractor.swift` | 65 | Extracts audio tracks to WAV for audio-to-video / speech transforms. | `src/preprocessing/audio.rs` |
| `Preprocessing/ImageConverter.swift` | 48 | Converts image formats and downsamples reference images. | `src/preprocessing/image.rs` |

---

### 4.3 Data Contracts & Catalog Models

#### 1. Catalog Models (`src/catalog/`)
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModelModality {
    Video,
    Image,
    Audio,
    Music,
    Upscale,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelCatalogEntry {
    pub id: String,
    pub display_name: String,
    pub modality: ModelModality,
    pub provider: String,
    pub default_cost: u32,
    pub supported_aspect_ratios: Vec<String>,
    pub supported_durations: Vec<f64>,
}
```

#### 2. Cost Estimator (`src/catalog/cost.rs`)
Calculates required credits before submission:
```rust
pub struct CostEstimator;
impl CostEstimator {
    pub fn estimate_video_cost(model_id: &str, duration_seconds: f64, resolution: &str) -> u32;
    pub fn estimate_image_cost(model_id: &str, count: usize) -> u32;
    pub fn estimate_speech_cost(model_id: &str, char_count: usize) -> u32;
    pub fn estimate_music_cost(model_id: &str, duration_seconds: f64) -> u32;
}
```

#### 3. Generation Request & Job Tracking (`src/backend.rs` & `src/service.rs`)
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GenerationJobStatus {
    Queued,
    Running,
    Succeeded,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationJob {
    pub id: String,
    pub status: GenerationJobStatus,
    pub progress: f64,
    pub result_url: Option<String>,
    pub error_message: Option<String>,
}
```

---

### 4.4 Preprocessing Pipeline (`src/preprocessing/`)

Preprocessing prepares local assets to be submitted as references:
- **`trim_video_reference`**: Uses `clawvinci_media::VideoStreamReader` / FFmpeg to slice an exact frame sub-range to MP4.
- **`extract_audio_reference`**: Extracts mono/stereo float PCM from timeline clips to WAV.
- **`convert_image_reference`**: Standardizes reference stills to PNG/JPEG with bounded maximum resolution.

---

### 4.5 Integration with MCP Agent Layer (`clawvinci-mcp`)

The MCP tool handlers in `windows/src-tauri/crates/clawvinci-mcp/src/tools/generate.rs` (stubbed in Phase 8) will be connected directly to `clawvinci-gen`:
1. **`list_models`**: Returns registered video, image, audio/TTS, and upscale models from `ModelCatalog`.
2. **`generate_video`**:
   - Inputs: `prompt`, `durationSeconds`, `model`, `aspectRatio`, `sourceVideoMediaRef`, `referenceImageMediaRefs`.
   - Preprocesses inputs, submits job, creates placeholder, and inserts video clip on completion.
3. **`generate_image`**:
   - Inputs: `prompt`, `aspectRatio`, `model`, `folder`.
   - Generates image asset, imports into package, and places on video track.
4. **`generate_audio`**:
   - Inputs: `prompt`, `audioType` ("speech" | "music"), `voice`, `durationSeconds`.
   - Generates speech or soundtrack audio and places on audio track.
5. **`upscale_media`**:
   - Inputs: `mediaRef`, `scaleFactor`.
   - Upscales video asset using remote upscale model.

---

## 5. Step-by-Step Implementation Sequence for Phase 11.0

The incoming team should follow this precise sequence:

1. **Update `windows/src-tauri/crates/clawvinci-gen/Cargo.toml`**:
   ```toml
   [dependencies]
   clawvinci-model = { path = "../clawvinci-model" }
   clawvinci-media = { path = "../clawvinci-media" }
   clawvinci-timeline = { path = "../clawvinci-timeline" }
   serde.workspace = true
   serde_json.workspace = true
   thiserror.workspace = true
   tokio.workspace = true
   tracing.workspace = true
   reqwest = { version = "0.12", features = ["json", "stream"] }
   uuid = { workspace = true, features = ["v4"] }
   chrono = { workspace = true }
   ```
2. **Implement Error Types (`src/error.rs`)**:
   - `GenError`: `Io`, `Serialization`, `ProviderError`, `InvalidInput`, `Timeout`, `PreprocessingFailed`.
3. **Implement Declarative Model Catalog (`src/catalog/`)**:
   - Port `ModelCatalog`, `VideoModelConfig`, `ImageModelConfig`, `AudioModelConfig`, `CostEstimator`, and `ModelPreferences`.
   - Provide default offline model definitions so unit tests run deterministically without internet.
4. **Implement Submission Payload Builders (`src/submission/`)**:
   - `video.rs`, `image.rs`, `audio.rs`, `music.rs`: serialize provider parameters matching `GenerationInput`.
5. **Implement Preprocessing Pipeline (`src/preprocessing/`)**:
   - Leverage `clawvinci-media` to extract clip ranges, export WAV audio tracks, and format reference stills.
6. **Implement Backend Job Client (`src/backend.rs`)**:
   - Job submission, status polling, and result downloading with cancellation token support.
   - Include mock client for testing and offline development.
7. **Implement Generation Service Orchestrator (`src/service.rs`)**:
   - Create placeholder assets with `GenerationStatus::Generating`.
   - Execute preprocessing, submit job, poll status, download finished file into `.palmier` bundle, probe via `clawvinci-media`, and notify completion.
8. **Wire `clawvinci-mcp/src/tools/generate.rs`**:
   - Connect `list_models`, `generate_video`, `generate_image`, `generate_audio`, and `upscale_media` to invoke `clawvinci-gen`.
9. **Write Integration Tests (`tests/gen_tests.rs`)**:
   - Test catalog lookup and cost estimations.
   - Test submission serialization.
   - Test preprocessing trim extraction on synthetic media.
   - Test mock generation lifecycle from placeholder to timeline insertion.
10. **Verify Remotely via GitHub Actions CI**:
    - Commit and push to `worktree-plan-windows-port`.
    - Run `gh run watch <RUN_ID>` and confirm Check, Clippy, Test, and Tauri Build are 100% Green.
    - Write `.agents/HISTORY/11-0-generative-ai.md` and update `.agents/HISTORY.md`.

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
