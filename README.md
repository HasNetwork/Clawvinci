> [!IMPORTANT]
> This repository contains **Clawvinci**, the native Windows port of Palmier Pro (the AI-native video editor with embedded MCP server). The original macOS Swift source code (releases through v0.7.6 published under GPLv3) is preserved at the repository root as a read-only behavioral and format reference. Clawvinci is developed natively for Windows under the same GPL-3.0 license.

<div align="center">

# Clawvinci

**AI-Native Desktop Video Editor for Windows**  
*Native Windows port of Palmier Pro with embedded Model Context Protocol (MCP) server*

[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](LICENSE)
[![Platform: Windows](https://img.shields.io/badge/Platform-Windows%2010%20%7C%2011-0078D6?logo=windows&logoColor=white)](windows/)
[![Rust](https://img.shields.io/badge/Rust-1.80%2B-orange?logo=rust&logoColor=white)](windows/src-tauri/)
[![Tauri v2](https://img.shields.io/badge/Tauri-v2-24C8D8?logo=tauri&logoColor=white)](https://v2.tauri.app/)
[![CI](https://github.com/palmier-io/palmier-pro/actions/workflows/ci.yml/badge.svg)](https://github.com/palmier-io/palmier-pro/actions/workflows/ci.yml)

<br />

<img src="./assets/palmier-ui.png" alt="Clawvinci UI" width="900" />

</div>

---

## Overview

**Clawvinci** brings the AI-native workflow of Palmier Pro to Windows. Designed from the ground up with a modular **Rust core** and a high-performance **Tauri v2** desktop shell, Clawvinci enables video editors and AI coding agents (Claude, Cursor, Codex, etc.) to collaborate directly on the same timeline in real time.

### Core Capabilities

- **Frame-Accurate Multi-Track Timeline**: Canvas-rendered timeline with sub-frame precision, interactive clip trimming, splitting, ripple/overwrite edits, and shared undo/redo history.
- **Embedded MCP Server**: Exposes a local HTTP/SSE Model Context Protocol endpoint (`http://127.0.0.1:19789/mcp`) with 53 specialized video editing tools that allow autonomous AI agents to inspect, cut, rearrange, and grade clips.
- **GPU Effects & Color Pipeline**: 12 shader algorithms (tone controls, lift/gamma/gain color wheels, vignette, grain, blur, glow, curves), 3D `.cube` LUT support with tetrahedral interpolation, and styled text overlays.
- **High-Performance Media Engine**: Built-in FFmpeg probe, decode, and encode pipeline with audio waveform generation and frame thumbnails.
- **Universal Project Compatibility**: Native support for the byte-compatible `.palmier` project bundle format, alongside export interchange to DaVinci Resolve (FCPXML) and Premiere Pro (XMEML).
- **Windows 10 & 11 Native**: Built for Windows 10 (22H2+) and Windows 11 on x86_64, utilizing DirectX 12 rendering via `wgpu` with automatic Vulkan fallback.

---

## Connecting AI Agents (MCP)

When Clawvinci is running, it hosts an embedded MCP server at `http://127.0.0.1:19789/mcp`. You can connect your favorite AI assistant to edit your timeline:

### Claude Code
```bash
claude mcp add --transport http clawvinci http://127.0.0.1:19789/mcp
```

### Codex
```bash
codex mcp add clawvinci --url http://127.0.0.1:19789/mcp
```

### Cursor
Add the following to your Cursor MCP settings (`~/.cursor/mcp.json` or `%USERPROFILE%\.cursor\mcp.json`):

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

### Claude Desktop
Configure your `claude_desktop_config.json`:

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

---

## Architecture

Clawvinci is organized as a clean Rust workspace inside [`windows/src-tauri/crates/`](windows/src-tauri/crates/):

| Crate | Purpose | Status |
|---|---|---|
| **`clawvinci-model`** | Domain models (`Timeline`, `Track`, `Clip`, `Timecode`), serialization, and `.palmier` format. | ✅ Complete |
| **`clawvinci-media`** | FFmpeg-based video/audio probe, frame decoding, thumbnail cache, waveform generation. | ✅ Complete |
| **`clawvinci-timeline`** | NLE editing primitives, ripple/overwrite engine, track/clip operations, command undo history. | ✅ Complete |
| **`clawvinci-render`** | `FramePlan` builder, compositor, 12 GPU effects, 3D LUT parser, text rasterization & animators. | ✅ Complete |
| **`clawvinci` (Tauri App)** | Application shell, IPC bridge (16+ commands), window management, Premium Dark UI. | ✅ Complete |
| **`clawvinci-export`** | Render-to-file batch export, FCPXML 1.10–1.14, Premiere XMEML 4, and export queue. | 🚀 In Progress (Phase 7.0) |
| **`clawvinci-mcp`** | Embedded HTTP MCP server exposing 53 editing tools to AI agents. | 📋 Planned (Phase 8.0) |
| **`clawvinci-audio`** | Beat detection (ONNX), silence removal, audio metering, and synchronization. | 📋 Planned (Phase 9.0) |
| **`clawvinci-search`** | Semantic visual search (SigLIP2 ONNX) and transcription search. | 📋 Planned (Phase 10.0) |
| **`clawvinci-gen`** | Generative AI integration (video and image generation providers). | 📋 Planned (Phase 11.0) |

---

## Getting Started (Windows Development)

### Prerequisites
1. **Windows 10 (22H2+) or Windows 11**.
2. **Visual Studio C++ Build Tools** with Windows 10/11 SDK.
3. **Rust** (`rustup default stable-x86_64-pc-windows-msvc`).
4. **Node.js** (v18+) and **npm**.

### Build & Run Locally

1. **Clone the repository**:
   ```powershell
   git clone https://github.com/palmier-io/palmier-pro.git
   cd palmier-pro
   ```

2. **Install frontend dependencies**:
   ```powershell
   npm --prefix windows install
   ```

3. **Run tests across all Rust crates**:
   ```powershell
   cargo test --workspace
   ```

4. **Verify lints**:
   ```powershell
   cargo clippy --workspace -- -D warnings
   ```

5. **Launch in development mode with live reload**:
   ```powershell
   npm --prefix windows run tauri dev
   ```

---

## Project Documentation & History

- **Developer Handover & Implementation Guide**: [`.agents/Handover.md`](.agents/Handover.md)
- **Phase Implementation Plans**: [`.agents/PLAN.md`](.agents/PLAN.md)
- **Implementation History**: [`.agents/HISTORY.md`](.agents/HISTORY.md)
- **Engineering Rules & Architecture Invariants**: [`.agents/RULES.md`](.agents/RULES.md)

---

## License

- The Clawvinci Windows codebase and original Palmier Pro GPL source code are published under the **GNU General Public License v3.0 (GPL-3.0-only)**. See [LICENSE](LICENSE) for full details.
- Palmier Pro binary releases for macOS after v0.7.6 are proprietary to Palmier, Inc.
