# Clawvinci — AI-Native Video Editor for Windows

Clawvinci is an AI-native desktop non-linear video editor (NLE) for Windows 10/11 featuring an embedded Model Context Protocol (MCP) server that empowers AI agents to edit timeline sequences directly alongside human editors.

The repository root references `.agents/` as the single authoritative source of truth for architecture, engineering rules, phase plans, and implementation history.

## Architecture

- **Stack**: Rust 1.80+ (workspace of 10 crates) + Tauri v2 + Vanilla HTML5/CSS/JavaScript.
- **Media Engine**: FFmpeg-backed probe, decode, encode, and waveform extraction.
- **Compositing & Effects**: Shared `FramePlan` builder, CPU/GPU compositing with 12 ported shader kernels, 3D `.cube` LUT tetrahedral interpolation, and kinetic text rendering.
- **Embedded MCP Server**: `http://127.0.0.1:19789/mcp` hosting 53 timeline editing and media generation tools.
- **Privacy & AI Discipline (Decision 10)**: Strictly **BYOK** (Bring Your Own Key) or local on-device ML. Zero telemetry (`no sentry/posthog`), no accounts (`no clerk`), no cloud proxy backend (`no api.palmier.io`).

## Core Invariants

1. **Remote CI Verification Only**:
   - **Never run `cargo`, `rustc`, or `npm` locally on the host machine.**
   - All compilation, testing, and linting must run via GitHub Actions CI (`.github/workflows/ci.yml`).
2. **Strict Invariant**:
   - All active code lives inside `windows/`. Do not create competing source directories at the root.
3. **Single Source of Truth (`.agents/`)**:
   - Full implementation plan: `.agents/PLAN.md` and `.agents/PLAN/<phase>-<subphase>.md`.
   - Coding and engineering rules: `.agents/RULES.md`.
   - Implementation logs and verification records: `.agents/HISTORY.md` and `.agents/HISTORY/<phase>-<subphase>.md`.
   - Handover guide: `.agents/handover.md`.
4. **Zero-Warning Tolerance**:
   - Every commit must pass `cargo clippy --workspace -- -D warnings` on CI without warnings.

## Workspace Layout

```
windows/
├── package.json               # Frontend dependencies & Tauri CLI scripts
├── src/                       # Web UI (HTML, CSS, JS, Locales)
│   ├── index.html             # Application shell & modals
│   ├── app-theme.css          # Design system & tokens
│   ├── i18n.js                # 28-locale client-side translation manager
│   └── locales/catalog.js     # Extracted string catalogs
└── src-tauri/                 # Rust Backend Workspace
    ├── Cargo.toml             # Workspace definition (10 crates)
    ├── tauri.conf.json        # Tauri v2 desktop & auto-updater configuration
    ├── src/                   # Main desktop binary & IPC command handlers
    ├── crates/
    │   ├── clawvinci-model    # 21 domain models & .palmier format
    │   ├── clawvinci-media    # FFmpeg media engine & waveforms
    │   ├── clawvinci-timeline # Timeline mutation core & undo history
    │   ├── clawvinci-render   # FramePlan builder & compositor
    │   ├── clawvinci-export   # Video export, FCPXML, XMEML & bundles
    │   ├── clawvinci-mcp      # Embedded MCP HTTP server (53 tools)
    │   ├── clawvinci-audio    # Audio DSP, metering, sync & silence detection
    │   ├── clawvinci-search   # SigLIP2 visual search & Whisper transcription
    │   └── clawvinci-gen      # BYOK generative AI clients (video, image, voice)
    └── tests/                 # Integration test suites
```

Refer to `.agents/RULES.md` for detailed coding rules.
