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

## Phase index

Each phase becomes a detailed doc in `PLAN/` when it is up next — writing
all of them now would be speculative (Rule 2/AGENTS.md "grow in layers").
Phase 0 and 1 are detailed now because they're immediately actionable and
every later phase depends on their output.

| Phase | Scope | Doc | Status |
|---|---|---|---|
| 0 | Foundation: repo layout, toolchain, CI, licensing carryover | [`PLAN/0-0-foundation.md`](PLAN/0-0-foundation.md) | **Detailed — ready to start** |
| 1 | Domain model & project file format (Rust structs, serde, byte-compatible `.palmier` package format) | [`PLAN/1-0-domain-model.md`](PLAN/1-0-domain-model.md) | **Detailed — ready to start** |
| 2 | Media engine: FFmpeg decode/encode wrapper, probing, thumbnails, waveforms | — | Not yet detailed |
| 3 | Timeline editing core: clip/track mutation ops, ripple/trim/split, shared undo history (mirrors "Editor mutations and undo" rule) | — | Not yet detailed |
| 4 | Preview/playback engine: GPU compositor, frame scheduling, audio scrub engine, A/V sync | — | Not yet detailed |
| 5 | GPU effects pipeline: port 12 Metal kernels → WGSL, `EffectRegistry`, LUTs, color wheels/curves, text rendering & animation | — | Not yet detailed |
| 6 | UI shell (Tauri/web): timeline view, inspector, media panel, preview canvas, design-token system (`AppTheme` equivalent) | — | Not yet detailed |
| 7 | Export: FCPXML + native XML exporters, export queue, HDR export, project export | — | Not yet detailed |
| 8 | MCP agent layer: HTTP MCP server on Windows, port all 53 tools (`Agent/Tools/ToolExecutor+*.swift`) | — | Not yet detailed |
| 9 | Audio analysis: beat detection, voice activity, speaker ID, silence removal, audio metering | — | Not yet detailed |
| 10 | Search/transcription/ML: transcription backend, embedding store, visual search — replace CoreML/MLX with ONNX Runtime | — | Not yet detailed |
| 11 | Generative AI integration: provider submission/catalog/edit clients (mostly portable REST logic) | — | Not yet detailed |
| 12 | Auth/backend/telemetry/updater: Clerk/Convex equivalents, Sentry/PostHog (both have Rust/JS SDKs), Windows updater replacing Sparkle | — | Not yet detailed |
| 13 | Polish: localization, settings UI, home/onboarding, in-app help | — | Not yet detailed |

Phases 2–4 are the critical path (nothing plays back or exports without
them) and the highest technical risk (FFmpeg↔Rust↔wgpu↔Tauri frame
pipeline, hitting 60fps scrub). Recommend prototyping the playback path
end-to-end (decode one clip → composite → present in a Tauri window) before
committing to the full Phase 3 timeline data model.

All open questions from the original scan (branding, OS floor, GPU backend,
CI) are resolved — see decisions 5–8 above. Nothing is blocking Phase 0.
