# Clawvinci Architecture Overview

Clawvinci is designed as a high-performance desktop NLE pairing a **Rust backend workspace** with a **Tauri v2 web shell**.

```
┌─────────────────────────────────────────────────────────────┐
│               Frontend UI (Tauri v2 Shell)                  │
│  Vanilla HTML5 Canvas Timeline + AppTheme CSS + 28 Locales  │
└──────────────────────────────┬──────────────────────────────┘
                               │ Tauri IPC (Asynchronous Commands)
┌──────────────────────────────▼──────────────────────────────┐
│                    Clawvinci Desktop Core                   │
│               (windows/src-tauri/src/lib.rs)                │
└──────┬───────────────────────┬───────────────────────┬──────┘
       │                       │                       │
┌──────▼──────────────┐ ┌──────▼──────────────┐ ┌──────▼──────────────┐
│  clawvinci-timeline │ │  clawvinci-render   │ │   clawvinci-mcp     │
│   TimelineEditor    │ │   Compositor &      │ │ Embedded HTTP/SSE   │
│ Symmetrical Command │ │    12 GPU Shaders   │ │   Server (Port      │
│  Undo/Redo History  │ │   FramePlan Builder │ │      19789)         │
└──────┬──────────────┘ └──────┬──────────────┘ └──────┬──────────────┘
       │                       │                       │
┌──────▼──────────────┐ ┌──────▼──────────────┐ ┌──────▼──────────────┐
│  clawvinci-media    │ │  clawvinci-export   │ │   clawvinci-gen     │
│  FFmpeg Probe/Codec │ │  Batch Video Export │ │ Direct BYOK Clients │
│ Waveform Extraction │ │ FCPXML 1.14 & XMEML │ │ (OpenAI, Kling, Fal)│
└──────┬──────────────┘ └──────┬──────────────┘ └─────────────────────┘
       │                       │
┌──────▼──────────────┐ ┌──────▼──────────────┐
│  clawvinci-model    │ │  clawvinci-audio    │
│ 21 Domain Entities  │ │  Peak/RMS Metering  │
│  .palmier Package   │ │ Silence/Dead-Air DSP│
└─────────────────────┘ └─────────────────────┘
```

---

## Workspace Crate Breakdown

1. **`clawvinci-model`**: Domain data models (`Timeline`, `Track`, `Clip`, `Timecode`, `ProjectFile`, `MediaManifest`). Defines byte-compatible `.palmier` package serialization.
2. **`clawvinci-media`**: FFmpeg integration for probing media metadata, decoding video frames, generating audio waveforms, and managing thumbnail caches.
3. **`clawvinci-timeline`**: NLE editing engine implementing the command pattern with symmetrical undo/redo history, ripple/overwrite edits, and track operations.
4. **`clawvinci-render`**: Frame compositing engine with 12 ported shader algorithms, 3D `.cube` LUT tetrahedral interpolation, and `fontdue` kinetic text rendering.
5. **`clawvinci-export`**: Batch video render-to-file, FCPXML 1.10–1.14 export, Premiere XMEML 4 export, self-contained project bundle export, and export queue.
6. **`clawvinci-mcp`**: Embedded HTTP/SSE server hosting 53 Model Context Protocol tools for AI agent automation.
7. **`clawvinci-audio`**: Audio DSP analysis engine (peak/RMS envelopes, cross-correlation alignment, silence detection, tempo/beat estimation).
8. **`clawvinci-search`**: Offline visual search via SigLIP2 (`PALMEMB1` vector store) and speech transcription (Whisper VAD and OpenAI BYOK client).
9. **`clawvinci-gen`**: Provider clients for video, image, and voice generative models with automated timeline insertion.
10. **`clawvinci` (Tauri App)**: Application entry point, window management, persistent `%APPDATA%` settings, Home hub, and 28-locale translation runtime.
