# Phase 7.0 — Export Engine (`clawvinci-export`)

**Status**: ✅ Complete  
**Date**: 2026-09-07  

## What was built

Phase 7.0 implements the complete Export Engine for Clawvinci (`clawvinci-export`), bringing timeline batch video rendering, interchange XML format export, self-contained project bundle packaging, an asynchronous export queue, and complete Tauri UI integration:

1. **Export Options & Format Profiles (`src/options.rs`):**
   - Derived from `Sources/PalmierPro/Export/ExportOptions.swift`.
   - `ExportFormat`: H.264, H.265, ProRes, HEVC 10-bit HDR, Premiere XMEML 4, and DaVinci Resolve FCPXML.
   - `ExportResolution`: 720p, 1080p, 1440p (2K), 4K, Match Timeline, and Custom dimensions, enforcing strictly even pixel dimensions (rounded to multiples of 2, minimum 2px) to prevent codec macroblock errors.
   - `VideoCodec`: H.264, H.265, ProRes, and HEVC 10-bit HDR with container and encoder mappings.
   - `FCPXMLVersion`: 1.10 (broadest compatibility, DaVinci Resolve 18+), 1.11, 1.12, 1.13, 1.14.
   - `FCPXMLTarget`: DaVinci Resolve (with conform fit scaling adjustments) and Final Cut Pro.

2. **Premiere Pro / FCP7 XMEML 4 Serializer (`src/xml.rs`):**
   - Derived from `Sources/PalmierPro/Export/XMLExporter.swift`.
   - Generates compliant XMEML Version 4 XML documents.
   - Centralized `XMLNode` tree builder with safe escaping of XML entities (`&`, `<`, `>`, `"`, `'`).
   - Frame-accurate SMPTE timecode formatting with drop-frame calculation (29.97/59.94 fps using `;` separator and 9 dropped frames per 10-minute block).
   - Bottom-to-top track ordering matching FCP7 convention.
   - Generates `<clipitem>` nodes with frame-accurate `<start>`, `<end>`, `<in>`, `<out>` points.
   - Translates transforms into standard motion filters: Basic Motion (Center, Scale, CCW Rotation), Opacity, Crop, Audio Levels (dB), and Time Remap (speed ramps).
   - Windows path to URL normalization (`file://localhost/...`).

3. **DaVinci Resolve & Final Cut Pro FCPXML Serializer (`src/fcpxml.rs`):**
   - Derived from `Sources/PalmierPro/Export/FCPXMLExporter.swift`.
   - Generates version-gated FCPXML (1.10 to 1.14).
   - Resources block with rational frame duration `<format>` elements (`1001/24000s`, `1/24s`, `1/30s`, `1/60s`) and asset definitions.
   - Primary storyline on `<spine>` (with `<gap>` elements preserving timing), and connected secondary tracks on positive and negative `<lane>` attributes.
   - DaVinci Resolve conform adjustments: position in percentage of frame height, square coordinates, pre-divided by conform-fit scale.
   - Title/Text clip rendering with `<title>` and `<text-style-def>` preserving font family, font size, fill color, and alignment.
   - Speed changes and ramps via `<timeMap>`.

4. **Self-Contained Project Bundle Packager (`src/project_bundle.rs`):**
   - Derived from `Sources/PalmierPro/Export/PalmierProjectExporter.swift`.
   - Packages projects into standalone `.palmier` directories:
     - Staged in `.palmier-export-<uuid>.partial/` with RAII auto-cleanup on failure/cancellation.
     - Collects and copies external media into `media/import-<id-prefix>.<ext>`.
     - Deduplicates identical media sources.
     - Rewrites `MediaManifestEntry.source` to project-relative paths (`MediaSource::Project { relative_path: "media/..." }`).
     - Serializes `project.json` and `manifest.json`.
     - Atomically installs the package to the destination directory.
     - Returns a `ProjectBundleReport` detailing collected assets, missing files, copied bytes, and warnings.

5. **HDR10 & HLG 10-bit Video Encoding Settings (`src/hdr.rs`):**
   - Derived from `Sources/PalmierPro/Export/HDRVideoExporter.swift`.
   - Configures BT.2020 color primaries, SMPTE ST 2084 (PQ) or ARIB STD-B67 (HLG) transfer characteristics, and BT.2020 NCL matrix.
   - Generates FFmpeg `libx265` Main10 10-bit encode parameters with `hdr10=1:repeat-headers=1`.

6. **Batch Render Service (`src/service.rs`):**
   - Derived from `Sources/PalmierPro/Export/ExportService.swift`.
   - Orchestrates frame-by-frame batch rendering of a `Timeline` to an output video file:
     - Enforces the **Unified Render Path Invariant**: reuses `clawvinci_render::CompositionBuilder::build_frame_plan` and `clawvinci_render::compositor::composite_frame`.
     - Streams composited RGBA pixel buffers into `clawvinci_media::VideoStreamWriter`.
     - Checks `tokio_util::sync::CancellationToken` at every frame boundary for clean, cooperative cancellation.
     - Cleans up partial output files on cancellation or failure.
     - Emits real-time progress callbacks `(0.0 .. 1.0)`.

7. **Export Queue Manager (`src/queue.rs`):**
   - Derived from `Sources/PalmierPro/Export/ExportQueue.swift`.
   - Manages asynchronous export jobs with status state machine: `Queued` → `Preparing` → `Rendering` → `Completed` | `Failed` | `Canceling` → `Canceled`.
   - Prevents duplicate writes with `is_destination_reserved(&Path)`.
   - Supports cooperative cancellation by job ID.
   - Supports clearing finished jobs.

8. **Tauri IPC Command Surface (`windows/src-tauri/src/lib.rs`):**
   - Integrated `ExportQueue` into `AppState`.
   - Added 6 new IPC commands:
     - `export_enqueue_video`: Enqueues and spawns background batch video rendering.
     - `export_enqueue_timeline`: Exports timeline to FCPXML or XMEML.
     - `export_enqueue_project_bundle`: Packages self-contained `.palmier` bundle.
     - `export_queue_list`: Retrieves all active and completed jobs.
     - `export_queue_cancel`: Cancels an active or waiting job.
     - `export_queue_clear_finished`: Cleans finished jobs from the queue.

9. **Frontend Export Modal UI (`windows/src/index.html`):**
   - Added Export button to top navigation bar.
   - Modern, dark-themed Export Modal dialog with 4 tabs:
     - **Render Video**: Output path, resolution preset (720p, 1080p, 2K, 4K, Match Timeline), codec (H.264, H.265, ProRes, HDR), CRF, and bitrate.
     - **Interchange**: FCPXML (DaVinci Resolve / FCP dialect, version 1.10–1.14) or Premiere Pro XMEML 4.
     - **Project Bundle**: Package self-contained `.palmier` bundle.
     - **Queue & Activity**: Live monitor of export jobs with progress bars, status badges, cancellation buttons, and clear finished.

10. **Automated Unit & Integration Test Suite (`tests/`):**
    - `options_tests.rs`: Render size calculations for 720p/1080p/4K/vertical video, even dimension enforcement, format extensions.
    - `xml_tests.rs`: SMPTE timecode (drop/non-drop), XMEML 4 structure, track hierarchy, clip filters.
    - `fcpxml_tests.rs`: FCPXML 1.10 output, rational time duration, primary spine and lane tracks, text title styles, conform transform adjustments.
    - `bundle_tests.rs`: Self-contained project bundle creation, media copying, deduplication, and manifest path rewriting.
    - `queue_tests.rs`: Queue state machine, destination reservation, progress updates, cancellation, and cleanup.

## Verification & Invariants Preserved

- **Scope Discipline**: Edited only `windows/` and `.agents/`. All code under `Sources/` remains unmodified.
- **Rule Codification**: Updated `.agents/RULES.md` to formally prohibit local toolchain runs and mandate GitHub Actions CI verification.
- **Unified Render Path**: Batch export directly consumes `clawvinci_render::compositor::composite_frame` and `CompositionBuilder::build_frame_plan`.
- **Integer Frame Timing**: All internal timeline positions and durations are integer frames (`i64`).
- **Clippy Invariants**: All IPC payloads use dedicated DTO structs (avoiding `too_many_arguments`); zero `field_reassign_with_default`; no reachable unwrap calls in command paths.
- **Licensing**: All new files are licensed GPL-3.0-only with attribution headers.
