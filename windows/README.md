# Clawvinci (Windows Port)

Native Windows port of Palmier Pro — an AI-native video editor with embedded Model Context Protocol (MCP) server support.

## Architecture

- **Backend**: Modular Rust workspace located in `src-tauri/`
  - `clawvinci`: Tauri v2 application shell and lifecycle coordinator
  - `clawvinci-model`: Domain models and byte-compatible `.palmier` project format
  - `clawvinci-media`: FFmpeg decoding, probing, thumbnails, and waveforms
  - `clawvinci-timeline`: Timeline editing primitives, ripple/overwrite ops, and undo history
  - `clawvinci-render`: wgpu rendering engine (DirectX 12 primary, Vulkan fallback)
  - `clawvinci-export`: FCPXML/XML export engine and render queue
  - `clawvinci-mcp`: Embedded HTTP/SSE MCP server with 53 video editing tools
  - `clawvinci-audio`: Beat detection, silence removal, and audio metering
  - `clawvinci-search`: Semantic search, SigLIP2 visual embeddings, and transcripts
  - `clawvinci-gen`: Generative AI provider clients and submission handlers
- **Frontend**: Lightweight, high-performance web shell located in `src/` conforming to `AppTheme` design system.

## Platform Floor

- **OS**: Windows 10 22H2 or later (x86_64 and arm64)
- **GPU Backend**: DirectX 12 (primary) with Vulkan fallback via `wgpu`

## Continuous Integration & Build Pipeline

Continuous integration runs automatically on GitHub Actions on `windows-latest` runners:
- Automated `cargo check --workspace`
- Automated `cargo clippy --workspace -- -D warnings`
- Automated `cargo test --workspace`
- Automated Tauri release packaging producing Windows `.exe` installers uploaded as workflow artifacts.

## Local Development (Optional)

If developing locally on Windows:
1. Install Rust (`rustup default stable-x86_64-pc-windows-msvc`).
2. Ensure Visual Studio C++ Build Tools and Windows SDK are installed.
3. Install frontend dependencies:
   ```powershell
   npm install
   ```
4. Run development shell:
   ```powershell
   npm run tauri dev
   ```

## Project Documentation

- Root README: [`../README.md`](../README.md)
- Developer Handover & Status: [`../.agents/Handover.md`](../.agents/Handover.md)
- Phase Implementation Plans: [`../.agents/PLAN.md`](../.agents/PLAN.md)
- Implementation History: [`../.agents/HISTORY.md`](../.agents/HISTORY.md)

## License

GPLv3. See `LICENSE` for terms.

