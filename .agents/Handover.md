# Clawvinci Project Handover & Implementation Guide

> **Target Audience**: Incoming AI Agent / Engineering Team  
> **Workspace Root**: `d:\projects\palmier-win\.claude\worktrees\plan-windows-port`  
> **Current Git Branch**: `worktree-plan-windows-port`  
> **Last Clean Commit**: `6cb398f`  
> **CI Status**: ✅ 100% Green (GitHub Actions Run `34041846290`)  
> **Current Milestone**: Phase 6.0 Complete & Verified; Ready to Implement Phase 7.0 (Export Engine)

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
6. **Zero-Warning Tolerance**:
   - All code must pass `cargo clippy --workspace -- -D warnings` with zero warnings.
   - All unit and integration tests must pass cleanly (`cargo test --workspace`).

---

## 3. Current Implementation Status (Phases 0 – 6 Complete)

| Phase | Module / Crate | Scope & Deliverables | Verification Status |
|---|---|---|---|
| **0.0** | `windows/` workspace | Foundation layout, 9 Cargo crates, Tauri v2 shell, GitHub Actions Windows CI (`.github/workflows/ci.yml`). | ✅ CI Green (`0-0-foundation.md`) |
| **1.0** | `clawvinci-model` | 21 domain models ported from Swift (`Timeline`, `Track`, `Clip`, `Timecode`, `ProjectFile`). Byte-compatible `.palmier` format + legacy decode fallback. | ✅ CI Green (`1-0-domain-model.md`) |
| **2.0** | `clawvinci-media` | FFmpeg probe, decode (`VideoStreamReader`), encode (`VideoStreamWriter`), waveform extraction, thumbnails, bounded concurrency. | ✅ CI Green (`2-0-media-engine.md`) |
| **3.0** | `clawvinci-timeline` | Core editing engine: `TimelineEditor`, undo/redo command history, ripple/overwrite edits, clip/track operations, editor coordinator. | ✅ CI Green (`3-0-timeline-editing-core.md`) |
| **4.0** | `clawvinci-render` | Playback engine: `FramePlan` builder, shared CPU compositor with blend modes, `PlaybackEngine` clock synchronization, Tauri IPC. | ✅ CI Green (`4-0-playback-engine.md`) |
| **5.0** | `clawvinci-render` | Effects pipeline: 12 Metal shader algorithms ported, `EffectRegistry` in canonical order, `.cube` LUT parser & tetrahedral interpolation, `fontdue` text layout & animators. | ✅ CI Green (`5-0-gpu-effects.md`) |
| **6.0** | `windows/src` & `lib.rs` | UI Shell: Premium Dark Design System, AppTheme tokens, interactive multi-track canvas timeline, inspector panel, media asset panel, 16 Tauri IPC commands. | ✅ CI Green (`6-0-ui-shell.md`, Run `34041846290`) |
| **7.0** | `clawvinci-export` | **Current Target**: Batch render-to-file, FCPXML 1.10–1.14 export, Premiere XMEML 4 export, self-contained `.palmier` bundle export, export queue. | 🚀 **Ready for Implementation** |
| **8.0–13.0** | Various | MCP Agent layer (53 tools), audio analysis (ONNX), search & transcription, generative AI, auth/telemetry, polish. | 📋 Planned in `.agents/PLAN/` |

---

## 4. Phase 7.0 — Export Engine Detailed Execution Blueprint

The incoming agent's primary assignment is to implement **Phase 7.0 (`clawvinci-export`)**.

### 4.1 Source Inventory & Rust Target Mapping
Reference source directory: `Sources/PalmierPro/Export/` (4,161 LOC across 9 files).

| macOS Swift Source | LOC | Rust Target in `clawvinci-export` | Role & Implementation Responsibility |
|---|---|---|---|
| `ExportOptions.swift` | 109 | `src/options.rs` | Export format, resolution, codec, and quality settings. |
| `XMLExporter.swift` | 656 | `src/xml.rs` | Apple/Premiere Pro XMEML 4 interchange format generator. |
| `FCPXMLExporter.swift` | 1,020 | `src/fcpxml.rs` | Final Cut Pro / DaVinci Resolve FCPXML (v1.10–1.14) interchange generator. |
| `PalmierProjectExporter.swift` | 165 | `src/project_bundle.rs` | Self-contained `.palmier` project bundle export (media copy + manifest rewrite). |
| `HDRVideoExporter.swift` | 291 | `src/hdr.rs` | HDR color space metadata & `libx265` Main10 10-bit encoding options. |
| `ExportService.swift` | 624 | `src/service.rs` | Orchestrates batch rendering: drives `composite_frame` per frame, feeds `VideoStreamWriter`. |
| `ExportQueue.swift` | 338 | `src/queue.rs` | Job queue manager with status lifecycle, reservation tracking, and cancellation. |
| `ExportTimelineAnalyticsSnapshot.swift` | 237 | `src/analytics.rs` | Timeline snapshot & telemetry metadata for export jobs. |
| `ExportView.swift` | 721 | `windows/src/index.html` | Export modal UI dialog in Tauri web frontend. |

---

### 4.2 Detailed Technical Architecture for Phase 7.0

#### 1. `options.rs`: Export Configuration
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExportFormat {
    Mp4,
    Mov,
    Webm,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VideoCodec {
    H264,
    Hevc,
    ProRes,
    Vp9,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExportResolution {
    Res720p,
    Res1080p,
    Res4k,
    Custom { width: u32, height: u32 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TimelineExportFormat {
    Fcpxml,
    Xmeml,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FCPXMLVersion {
    V1_10,
    V1_11,
    V1_12,
    V1_13,
    V1_14,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FCPXMLTarget {
    Resolve,
    FinalCutPro,
}
```

#### 2. `xml.rs`: Premiere Pro XMEML 4 Exporter
- Outputs standard Premiere Pro XMEML (Version 4).
- Structure:
  ```xml
  <?xml version="1.0" encoding="UTF-8"?>
  <!DOCTYPE xmeml>
  <xmeml version="4">
    <sequence id="sequence-1">
      <name>Timeline</name>
      <duration>...</duration>
      <rate>
        <timebase>24</timebase>
        <ntsc>FALSE</ntsc>
      </rate>
      <media>
        <video>
          <format>
            <samplecharacteristics>
              <width>1920</width>
              <height>1080</height>
            </samplecharacteristics>
          </format>
          <track>
            <clipitem id="clipitem-1">
              ...
            </clipitem>
          </track>
        </video>
        <audio>...</audio>
      </media>
    </sequence>
  </xmeml>
  ```
- Must translate clip transforms (center, scale, rotation, opacity) into `<filter>` motion effects.
- Translate volume into audio levels.
- Formats SMPTE timecode (properly handling 29.97 drop-frame vs non-drop-frame).

#### 3. `fcpxml.rs`: FCPXML Exporter (DaVinci Resolve & Final Cut Pro)
- Target dialect `FCPXMLVersion::V1_10` by default for maximum DaVinci Resolve compatibility.
- Format elements `<fcpxml version="1.10">` -> `<resources>` -> `<library>` -> `<event>` -> `<project>` -> `<sequence>` -> `<spine>`.
- Generates `<format>` definitions with exact frame duration ratios (`1001/24000s`, `1/24s`, `1/30s`, `1001/30000s`, `1/60s`).
- Clips on track 0 map to `<spine>`, while overlapping/upper tracks map to connected clips with `<lane>` attributes.
- Retiming and speed ramps map to `<timeMap>` nodes.
- When `target == FCPXMLTarget::Resolve`, account for DaVinci Resolve's conform scaling differences.

#### 4. `project_bundle.rs`: Self-Contained `.palmier` Bundle Exporter
- Creates a staged directory: `<dest_path>.palmier-export-<uuid>.partial/`.
- Subdirectories: `media/`, `thumbnails/`.
- Iterates over all assets in `MediaManifest`:
  - Deduplicates sources by content hash / unique identifier.
  - Copies source files into `media/import-<id-prefix>.<ext>`.
  - Rewrites `MediaManifestEntry.source` to `.project(relativePath: "media/...")`.
- Serializes `project.json` and `manifest.json`.
- Atomically replaces/renames the staging directory to `<dest_path>.palmier`.

#### 5. `hdr.rs`: HDR Color Spaces & Encoding Parameters
- Scoped in for SDR (BT.709) and HDR10 (BT.2020 / PQ / HLG).
- Configures FFmpeg `libx265` parameters:
  - Pixel format: `yuv420p10le`.
  - Color primaries: `bt2020`.
  - Transfer characteristics: `smpte2084` (PQ) or `arib-std-b67` (HLG).
  - Matrix coefficients: `bt2020nc`.
  - Mastering display metadata (`master-display`) and MaxCLL/MaxFALL.

#### 6. `service.rs`: Frame-by-Frame Batch Render Pipeline
```rust
pub async fn render_timeline_to_file(
    timeline: &Timeline,
    options: &VideoExportOptions,
    destination: &Path,
    cancel_token: CancellationToken,
    progress_callback: impl Fn(f64) + Send + 'static,
) -> Result<(), ExportError>
```
- **Execution Flow**:
  1. Initialize `VideoStreamWriter` at the specified resolution and frame rate.
  2. Compute total frame count from `timeline.duration()`.
  3. Instantiate `CompositionBuilder::new(...)` using `clawvinci-render`.
  4. Loop frame-by-frame from `0` to `total_frames - 1`:
     - Check `cancel_token.is_cancelled()`. If canceled, abort, clean up partial output, and return `ExportError::Cancelled`.
     - Build `FramePlan` for current frame.
     - Call `clawvinci_render::compositor::composite_frame(&plan, &assets, &lut_cache, &font_cache)`.
     - Pass the composited `RgbaImage` to `VideoStreamWriter::write_frame(...)`.
     - Calculate progress percentage and emit progress callback.
  5. Finalize `VideoStreamWriter::finish()` to mux and close the container cleanly.

#### 7. `queue.rs`: Serialized Export Queue
- Manages an in-memory queue of export jobs.
- State machine:
  `Queued` -> `Preparing` -> `Rendering` -> `Completed` | `Failed` | `Canceling` -> `Canceled`.
- `is_destination_reserved(path: &Path) -> bool`: prevents concurrent jobs from colliding on the same output path.
- Supports cooperative cancellation via job ID.
- Shared between Tauri IPC commands and future Phase 8 MCP tools (`export_project`, `manage_exports`).

---

### 4.3 Step-by-Step Implementation Instructions for the Next Agent

When beginning work on Phase 7.0, follow this exact sequence:

1. **Update `windows/src-tauri/crates/clawvinci-export/Cargo.toml`**:
   Add workspace dependencies:
   ```toml
   clawvinci-model = { path = "../clawvinci-model" }
   clawvinci-media = { path = "../clawvinci-media" }
   clawvinci-timeline = { path = "../clawvinci-timeline" }
   clawvinci-render = { path = "../clawvinci-render" }
   serde.workspace = true
   serde_json.workspace = true
   thiserror.workspace = true
   tokio.workspace = true
   tokio-util = { version = "0.7", features = ["sync"] }
   tracing.workspace = true
   uuid = { version = "1.10", features = ["v4", "serde"] }
   chrono = { version = "0.4", features = ["serde"] }
   image.workspace = true
   ```
2. **Implement Modules in `clawvinci-export/src/`**:
   - `options.rs`: Port export configuration enums and structs.
   - `xml.rs`: Implement XMEML 4 serializer with SMPTE timecode formatting.
   - `fcpxml.rs`: Implement FCPXML 1.10–1.14 generator with Resolve target adaptions.
   - `project_bundle.rs`: Implement self-contained `.palmier` bundle packager.
   - `hdr.rs`: Implement HDR metadata formatting and encoder argument builders.
   - `analytics.rs`: Port export timeline snapshot models.
   - `service.rs`: Implement batch render loop using `clawvinci_render::compositor::composite_frame`.
   - `queue.rs`: Implement `ExportQueue` state machine and cancellation.
   - `lib.rs`: Expose public API and define `ExportError`.
3. **Add Comprehensive Tests**:
   - `tests/xml_tests.rs`: Validate generated XML against XMEML schema and SMPTE frame calculations.
   - `tests/fcpxml_tests.rs`: Test DaVinci Resolve conform format generation, timeMap speed ramps, and multi-track lanes.
   - `tests/bundle_tests.rs`: Test `.palmier` project bundle export round-trip.
   - `tests/queue_tests.rs`: Test queue state transitions and cooperative cancellation mid-render.
4. **Wire Tauri IPC Commands in `windows/src-tauri/src/lib.rs`**:
   - Add `ExportQueue` to `AppState`.
   - Register commands:
     - `export_enqueue_video(options: VideoExportOptionsPayload)`
     - `export_enqueue_timeline(format: String, path: String)`
     - `export_enqueue_project_bundle(destination: String)`
     - `export_queue_list()`
     - `export_queue_cancel(job_id: String)`
5. **Add UI Export Dialog in `windows/src/`**:
   - Add Export modal button in `index.html` top navigation bar.
   - Form fields: Destination path, Format (MP4/MOV), Resolution (1080p/4K), Codec (H.264/HEVC), or Interchange (FCPXML/XMEML).
   - Render progress bar and cancellation trigger.
6. **Verify, Lint, & Commit**:
   - Run `cargo test --workspace`.
   - Run `cargo clippy --workspace -- -D warnings`.
   - Commit: `feat: Phase 7.0 Export engine, FCPXML/XMEML interchange, project bundle exporter, and queue`.
   - Update `.agents/HISTORY/7-0-export.md` and `.agents/HISTORY.md`.

---

## 5. Critical Gotchas & Coding Rules for This Workspace

- **Clippy `too_many_arguments`**:
  Tauri commands with more than 7 arguments will trigger clippy errors. Always bundle multi-argument payloads into dedicated DTO structs (e.g. `UpdateClipTransformPayload`, `VideoExportOptionsPayload`).
- **Clippy `field_reassign_with_default`**:
  Never construct a struct with `let mut x = Type::default(); x.field = val;`. Always use struct update syntax: `Type { field: val, ..Default::default() }`.
- **Mutex Lock Scopes in `AppState`**:
  Do not hold `tokio::sync::Mutex` or `std::sync::Mutex` locks across `.await` points when orchestrating render loops. Snapshot necessary immutable state before beginning async operations.
- **Windows Path Formatting**:
  Paths on Windows contain backslashes (`\`). When serializing into XML or URLs (`file:///`), convert Windows paths cleanly. When performing atomic renames, ensure target files are not kept open by active readers.

---

## 6. Developer Commands Quick Reference

All commands must be executed in PowerShell from the repository root:

```powershell
# Run all workspace unit and integration tests
cargo test --workspace

# Run strict clippy verification (must pass with 0 warnings)
cargo clippy --workspace -- -D warnings

# Build the entire Tauri application
cargo build --manifest-path windows/src-tauri/Cargo.toml

# Launch Tauri development mode with hot reload
npm --prefix windows run tauri dev
```
