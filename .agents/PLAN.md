# PLAN.md — Clawvinci (Windows port of Palmier Pro)

Index for the Windows port project. Detailed phase/subphase plans live in
`.agents/PLAN/<phase>-<subphase>.md`. Rules the coder agent must follow live in
`.agents/RULES.md`. Implementation history lives in `.agents/HISTORY.md`.

## What this is

Palmier Pro is a macOS-26/Apple-Silicon-only video editor with an embedded MCP
server that lets AI agents edit the timeline directly (`README.md`). The
published source (through the point macOS binaries went proprietary) is
GPLv3 — see `LICENSE`. This project is **not a source port**: SwiftUI, AppKit,
AVFoundation, CoreImage/Metal, and MLX have no Windows equivalent, and only
~27% of the Swift source (the Apple-free layer: domain models, the MCP tool
layer, API clients — see the architecture-scan findings below) survives
translation unchanged. Instead this is **Clawvinci**, a **ground-up Windows
application** that reproduces Palmier Pro's behavior, file formats, and MCP
tool surface, written natively for Windows, using the GPLv3 source as a
read-only behavioral and format reference (license permits copying — see
Decisions).

The original macOS source is preserved unmodified at the repo root as that
reference. Nothing under `Sources/`, `Metal/`, `Plugins/`, `models/`,
`scripts/`, `mcpb/` is to be edited by this project — treat it as read-only.
New Windows-port code lives in a new top-level layout defined in
`PLAN/0-0-foundation.md`.

## Decisions locked in (do not re-litigate without flagging why)

| # | Decision | Chosen |
|---|---|---|
| 1 | Core stack | **Rust core + Tauri v2 web UI**. Rust owns decode/compositing/encode/state; UI is HTML/CSS/TS talking to Rust over Tauri IPC. |
| 2 | v1 scope | **Full feature parity** with the macOS app, delivered in phases (below), not a cut-down v1. |
| 3 | Media engine | **FFmpeg** (via Rust bindings) for decode/encode/containers. |
| 4 | Licensing | **GPLv3, copy freely.** The Windows app ships GPLv3. Logic may be translated or lifted directly from the reference source; preserve GPLv3 headers/attribution per `RULES.md`. |
| 5 | Product naming/branding | **Clawvinci.** "Palmier Pro" branding (post-v0.7.6) is proprietary to Palmier, Inc. even though this source is GPLv3, so the Windows app ships under its own name. Applies to the Tauri product name/window title, crate prefix (`clawvinci-*`, see `PLAN/0-0-foundation.md`), and anything user-facing. |
| 6 | Windows OS floor | **Windows 10 22H2+** (not Win11-only). Every native API choice (WebView2 requirement, DX12 feature level, packaging) must keep this floor viable. |
| 7 | GPU backend | **DX12 primary via wgpu, Vulkan fallback.** No DX12-only assumption in `clawvinci-render`/Phase 4-5 code — wgpu's backend selection must actually try Vulkan when DX12 init fails, not just compile-support it. |
| 8 | CI | **GitHub Actions `windows-latest` runners by default.** Self-hosted is fine, and preferred, for any phase that needs extra native libraries (FFmpeg build deps, GPU drivers for wgpu/D3D12 integration tests) that a hosted runner can't provide cheaply — decide per-phase when that need is concrete, not speculatively now. |
| 9 | Project file format | **Original `.palmier` format, byte-compatible.** Same `project.json` shape (including the legacy bare-`Timeline` decode fallback) and package layout as the macOS app, so projects move between platforms. This is Phase 1's format decision — see `PLAN/1-0-domain-model.md`, now resolved to option 1 there. |

## Architecture scan findings (informs every phase below)

- 415 Swift files, 88,402 LOC total. 146 files import SwiftUI, 83 AppKit, 48
  AVFoundation, 25 CoreImage. 12 Metal shaders (`Metal/*.metal`) +
  `Plugins/MetalCIKernelPlugin` compile them into `.metallib`.
- **139 files / 23,892 LOC** import no Apple-only framework — mostly
  `Models/` (Timeline/Track/Clip/ProjectFile — plain `Codable, Sendable`
  value types, frame-domain integers, no `CGFloat` in the hot path),
  `Agent/Tools/` (MCP tool executor — 53 tools, HTTP+JSON), and
  `Generation/*Submission,Catalog,Edit` (AI-provider API clients). This is
  the cheapest layer to port and should be translated first — it defines the
  wire format and file format everything else builds against.
- `AGENTS.md` at repo root documents the engineering invariants the macOS
  team enforced (concurrency/main-actor discipline, file-I/O safety, undo
  semantics, performance rules, design-token discipline). These are
  **behavioral requirements to preserve**, not Swift-specific — `RULES.md`
  restates them for the Rust/Tauri context.
- Render/export/preview engine is the largest and riskiest area:
  `Preview/` (6,034 LOC — `VideoEngine`, `CompositionBuilder`,
  `ScrubAudioEngine`), `Compositing/` (2,588 LOC — `FrameRenderer`,
  `TextFrameRenderer`, `EffectRegistry`, 12 GPU kernels), `Export/`
  (4,161 LOC — FCPXML + native XML exporters, `ExportQueue`, HDR export).
- `Agent/` is the largest single area (16,684 LOC, 57 files) — the in-app
  agent chat/panel plus the 53-tool MCP executor. The tool executor itself
  is portable logic; the chat UI is not.
- `Transcription/` is **not on-device ML** — despite living next to the
  search/visual-embedding code, `TranscriptionBackend.swift` submits jobs
  to a cloud (Convex) backend and subscribes to results; there's no local
  Whisper-class model to port. The actual on-device ML footprint in the
  whole app is narrow and concrete: `beat_this` (beat detection) and
  SigLIP2 (visual search embedding), both already documented in
  `models/` with a PyTorch→CoreML conversion pipeline that re-targets to
  ONNX with modest extra work — see `PLAN/9-0-audio-analysis.md` and
  `PLAN/10-0-search-transcription-ml.md`. Apple's system `Speech`/
  `SpeechVAD`/`SpeechEnhancement` frameworks (voice activity, speaker ID,
  denoise) are the one area with no bundled model to re-export — flagged
  as an open per-feature decision in Phase 9, not silently assumed.

## Phase index

Every phase is now detailed (the user asked for full planning up front
rather than the phase-by-phase "grow in layers" default). Phases 2-13
were written from a second pass over the actual source — file inventories,
LOC, and representative file reads for each area — not from the
architecture-scan summary alone; each doc cites the real macOS files it's
grounded in.

| Phase | Scope | Doc |
|---|---|---|
| 0 | Foundation: repo layout, toolchain, CI, licensing carryover | [`PLAN/0-0-foundation.md`](PLAN/0-0-foundation.md) |
| 1 | Domain model & byte-compatible `.palmier` project file format | [`PLAN/1-0-domain-model.md`](PLAN/1-0-domain-model.md) |
| 2 | Media engine: FFmpeg decode/encode/probe, thumbnails, waveforms | [`PLAN/2-0-media-engine.md`](PLAN/2-0-media-engine.md) |
| 3 | Timeline editing core: clip/track mutation ops, ripple/overwrite, shared undo history | [`PLAN/3-0-timeline-editing-core.md`](PLAN/3-0-timeline-editing-core.md) |
| 4 | Preview/playback engine: render-graph builder, real-time A/V playback, scrub | [`PLAN/4-0-playback-engine.md`](PLAN/4-0-playback-engine.md) |
| 5 | GPU effects pipeline: 12 Metal kernels → WGSL, `EffectRegistry`, LUTs, text rendering & animation | [`PLAN/5-0-gpu-effects.md`](PLAN/5-0-gpu-effects.md) |
| 6 | UI shell (Tauri/web): timeline canvas, inspector, media panel, design tokens, IPC surface | [`PLAN/6-0-ui-shell.md`](PLAN/6-0-ui-shell.md) |
| 7 | Export: FCPXML + native XML exporters, export queue, HDR export, project bundle export | [`PLAN/7-0-export.md`](PLAN/7-0-export.md) |
| 8 | MCP agent layer: HTTP MCP server, all 53 tools, in-app agent chat orchestration | [`PLAN/8-0-mcp-agent-layer.md`](PLAN/8-0-mcp-agent-layer.md) |
| 9 | Audio analysis: beat detection (ONNX), sync, silence removal, metering; VAD/speaker-ID/enhancement flagged open | [`PLAN/9-0-audio-analysis.md`](PLAN/9-0-audio-analysis.md) |
| 10 | Search & transcription: SigLIP2 visual search (ONNX), transcript search — transcription itself is a cloud API client | [`PLAN/10-0-search-transcription-ml.md`](PLAN/10-0-search-transcription-ml.md) |
| 11 | Generative AI integration: provider catalog/submission/edit clients, preprocessing | [`PLAN/11-0-generative-ai.md`](PLAN/11-0-generative-ai.md) |
| 12 | Auth/backend/telemetry/updater: Clerk auth (needs a scoping decision), Convex client, Sentry/PostHog, Tauri updater | [`PLAN/12-0-auth-backend-telemetry-updater.md`](PLAN/12-0-auth-backend-telemetry-updater.md) |
| 13 | Polish: localization, settings panes, home/onboarding, in-app help | [`PLAN/13-0-polish.md`](PLAN/13-0-polish.md) |

Phases 2–4 are the critical path (nothing plays back or exports without
them) and the highest technical risk (FFmpeg↔Rust↔wgpu↔Tauri frame
pipeline, hitting interactive scrub rates, and — per Phase 4 — no
off-the-shelf equivalent to AVFoundation's composition objects). Phase
4's doc calls out a specific prototype to build *before* committing to
Phase 3's data shapes: decode one clip → trivial composite → present in a
Tauri window → confirm A/V-synced playback and scrub actually work. Do
that prototype early regardless of numeric phase order — it's the
riskiest unproven technical assumption in the whole plan.

A few phases carry an explicit open decision that needs the user's input
*when that phase starts*, not now (deferring per Rule 2 — these are
concrete only once their phase is actually being built): Phase 9's
VAD/speaker-ID/speech-enhancement model replacement, Phase 12's Clerk
auth integration approach on Windows, and Phase 1's exact JSON-shape
capture method. Each is called out in its phase doc; none blocks earlier
phases.

All open questions from the original scan (branding, OS floor, GPU backend,
CI) are resolved — see decisions 5–8 above. Nothing is blocking Phase 0.
