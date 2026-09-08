<div align="center">

# Clawvinci

**The "Cursor IDE" for Video Editing**  
*AI-Native Desktop NLE for Windows with Embedded Model Context Protocol (MCP) Server*

[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](LICENSE)
[![Platform: Windows](https://img.shields.io/badge/Platform-Windows%2010%20%7C%2011-0078D6?logo=windows&logoColor=white)](windows/)
[![Rust](https://img.shields.io/badge/Rust-1.80%2B-orange?logo=rust&logoColor=white)](windows/src-tauri/)
[![Tauri v2](https://img.shields.io/badge/Tauri-v2-24C8D8?logo=tauri&logoColor=white)](https://v2.tauri.app/)
[![CI](https://github.com/HasNetwork/Clawvinci/actions/workflows/ci.yml/badge.svg)](https://github.com/HasNetwork/Clawvinci/actions/workflows/ci.yml)

</div>

---

## Overview

**Clawvinci** is the **"Cursor IDE" for Video Editing**. 

Just as modern AI-first code editors (like Cursor or Windsurf) pair software developers with AI agents that can directly read files, inspect ASTs, suggest edits, and invoke linters inside the workspace, Clawvinci pairs video creators with AI agents that can directly read, cut, trim, arrange, color grade, and animate clips on the timeline canvas.

Built with a high-performance **Rust workspace** (10 modular crates) and a lightweight **Tauri v2** desktop shell, Clawvinci hosts an embedded **Model Context Protocol (MCP)** server on `http://127.0.0.1:19789/mcp`. Connected AI assistants (such as Claude Desktop, Claude Code, Cursor, Codex, or local LLMs) act as collaborative co-editors—analyzing audio waveforms, cutting dead air, syncing multi-track recordings, grading colors, and generating kinetic text overlays, all preserved within a frame-accurate, shared undo/redo history.

---

## Key Capabilities

- **Frame-Accurate Multi-Track Timeline**: Canvas-rendered timeline with sub-frame precision, interactive clip trimming, splitting, ripple/overwrite edits, and shared undo/redo history.
- **Embedded MCP Server (53 Tools)**: Exposes a local HTTP/SSE Model Context Protocol endpoint (`http://127.0.0.1:19789/mcp`) allowing external AI agents to programmatically inspect media, execute ripple edits, apply effects, and generate assets.
- **28-Locale Localization**: Full client-side internationalization across 28 languages (English, Spanish, French, German, Japanese, Simplified Chinese, Traditional Chinese, Hindi, Arabic, Russian, Korean, etc.).
- **GPU Effects & Color Grading**: 12 shader algorithms (tone controls, lift/gamma/gain color wheels, vignette, grain, blur, glow, curves), 3D `.cube` LUT support with tetrahedral interpolation, and styled kinetic text animations via `fontdue`.
- **High-Performance Media Engine**: Built-in FFmpeg probe, decode, and encode pipeline with background waveform extraction, thumbnail caching, and bounded memory concurrency.
- **Semantic Search & Transcription**: Offline visual search powered by SigLIP2 (`PALMEMB1` binary vector store) and on-device speech transcription (Whisper VAD & local engine, plus OpenAI BYOK Whisper REST client).
- **Generative AI Integration**: Direct BYOK integration with generative models across modalities (OpenAI, Kling, Seedance, Fal, ElevenLabs, Suno) with automated timeline placement.
- **Strictly Private & BYOK (Decision 10)**: Zero telemetry (`no sentry/posthog`), no accounts or subscriptions (`no clerk`), and no cloud middleman (`no api.palmier.io`). All AI calls go directly to the provider with your own keys or run locally on your device.
- **Universal Project Compatibility**: Native support for the byte-compatible `.palmier` project package format, alongside export interchange to DaVinci Resolve (FCPXML 1.10–1.14) and Premiere Pro (XMEML 4).

---

## Connecting AI Agents (MCP)

When Clawvinci is running, it hosts an embedded MCP server at `http://127.0.0.1:19789/mcp`. You can connect your favorite AI assistant to edit your timeline directly:

### 1. Claude Code CLI
```bash
claude mcp add --transport http clawvinci http://127.0.0.1:19789/mcp
```

### 2. Cursor IDE
Add the following to your Cursor MCP settings (`%USERPROFILE%\.cursor\mcp.json`):
```json
{
  "mcpServers": {
    "clawvinci": {
      "type": "http",
      "url": "http://127.0.0.1:19789/mcp"
    }
  }
}
```

### 3. Claude Desktop
Add the following to your Claude Desktop configuration (`%APPDATA%\Claude\claude_desktop_config.json`):
```json
{
  "mcpServers": {
    "clawvinci": {
      "type": "http",
      "url": "http://127.0.0.1:19789/mcp"
    }
  }
}
```

### 4. Codex CLI
```bash
codex mcp add clawvinci --url http://127.0.0.1:19789/mcp
```

---

## Architecture

Clawvinci is organized as a modular Rust workspace inside [`windows/src-tauri/crates/`](windows/src-tauri/crates/):

| Crate | Purpose | Status |
|---|---|---|
| **`clawvinci-model`** | 21 domain models (`Timeline`, `Track`, `Clip`, `Timecode`, `ProjectFile`), serialization, and byte-compatible `.palmier` package format. | ✅ 100% Complete |
| **`clawvinci-media`** | FFmpeg-based video/audio probe, frame decoding, thumbnail cache, waveform generation. | ✅ 100% Complete |
| **`clawvinci-timeline`** | NLE editing primitives, ripple/overwrite engine, track/clip operations, command undo/redo history. | ✅ 100% Complete |
| **`clawvinci-render`** | `FramePlan` builder, compositor, 12 GPU effects, 3D LUT parser, text rasterization & animators. | ✅ 100% Complete |
| **`clawvinci-export`** | Render-to-file batch export, FCPXML 1.10–1.14, Premiere XMEML 4, and export queue. | ✅ 100% Complete |
| **`clawvinci-mcp`** | Embedded HTTP/SSE MCP server exposing 53 timeline editing tools to external AI agents. | ✅ 100% Complete |
| **`clawvinci-audio`** | Real-time peak/RMS metering, cross-correlation audio sync, silence/dead-air detection, tempo/beat detector, VAD. | ✅ 100% Complete |
| **`clawvinci-search`** | Semantic visual search (SigLIP2 ONNX, `PALMEMB1` store) and transcript search. | ✅ 100% Complete |
| **`clawvinci-gen`** | Direct BYOK generative AI clients (video, image, voice, music) with timeline insertion. | ✅ 100% Complete |
| **`clawvinci` (Tauri App)** | Application shell, IPC bridge, window management, persistent settings, Home hub, and Web UI. | ✅ 100% Complete |

---

## Getting Started (Windows Development)

### Prerequisites
1. **Windows 10 (22H2+) or Windows 11** (x86_64).
2. **Visual Studio C++ Build Tools** with Windows 10/11 SDK.
3. **Rust** (`rustup default stable-x86_64-pc-windows-msvc`).
4. **Node.js** (v20+) and **npm**.
5. **FFmpeg** on system `PATH`.

### Build & Run Locally

1. **Clone the repository**:
   ```powershell
   git clone https://github.com/HasNetwork/Clawvinci.git
   cd Clawvinci
   ```

2. **Install frontend dependencies**:
   ```powershell
   cd windows
   npm install
   ```

3. **Run tests across all Rust crates**:
   ```powershell
   cd src-tauri
   cargo test --workspace
   ```

4. **Verify lints with zero warnings**:
   ```powershell
   cargo clippy --workspace -- -D warnings
   ```

5. **Launch Clawvinci in desktop development mode**:
   ```powershell
   cd ..
   npm run tauri dev
   ```

6. **Build release package (NSIS installer)**:
   ```powershell
   npm run tauri build
   ```

---

## Documentation & Implementation History

The `.agents/` folder serves as the single source of truth for the project:
- **Developer Handover Manual**: [`.agents/handover.md`](.agents/handover.md)
- **Master Architecture Plan**: [`.agents/PLAN.md`](.agents/PLAN.md)
- **Implementation History**: [`.agents/HISTORY.md`](.agents/HISTORY.md)
- **Phase 13.0 Polish Log**: [`.agents/HISTORY/13-0-polish.md`](.agents/HISTORY/13-0-polish.md)
- **Coding Rules & Invariants**: [`.agents/RULES.md`](.agents/RULES.md)

---

## License

Clawvinci is licensed under the **GNU General Public License v3.0 (GPL-3.0-only)**. See [LICENSE](LICENSE) for details.
