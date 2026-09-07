# HISTORY.md — implementation log index

Index of completed subphases. Each entry links to a detailed log in
`.agents/HISTORY/<phase>-<subphase>.md`, written by the coder agent
immediately after that subphase's commit.
| Phase | Subphase | Summary | Log |
|---|---|---|---|
| 0 | 0.0 | Foundation: Windows layout, Cargo workspace, 9 crates, Tauri v2 shell, CI | [`0-0-foundation.md`](HISTORY/0-0-foundation.md) |
| 1 | 1.0 | Domain model: 21 models ported, byte-compatible .palmier project format, legacy fallback | [`1-0-domain-model.md`](HISTORY/1-0-domain-model.md) |
| 2 | 2.0 | Media engine: FFmpeg probe, decode, encode, waveforms, thumbnails, bounded concurrency | [`2-0-media-engine.md`](HISTORY/2-0-media-engine.md) |
| 3 | 3.0 | Timeline editing core: undo stack, ripple/overwrite engines, clip/track ops, editor coordinator | [`3-0-timeline-editing-core.md`](HISTORY/3-0-timeline-editing-core.md) |
| 4 | 4.0 | Playback engine: FramePlan builder, CPU compositor with blend modes, PlaybackEngine, Tauri integration | [`4-0-playback-engine.md`](HISTORY/4-0-playback-engine.md) |
| 5 | 5.0 | Effects pipeline: 12 shader algorithms ported, EffectRegistry, .cube LUTs & tetra interp, text rendering & animation | [`5-0-gpu-effects.md`](HISTORY/5-0-gpu-effects.md) |
| 6 | 6.0 | UI shell: Premium Dark Design System, AppTheme tokens, multi-track timeline canvas, inspector, media panel, Tauri IPC | [`6-0-ui-shell.md`](HISTORY/6-0-ui-shell.md) |
| 7 | 7.0 | Export engine: Batch render-to-file, FCPXML 1.10–1.14, Premiere XMEML 4, project bundle packager, export queue, UI modal | [`7-0-export.md`](HISTORY/7-0-export.md) |
| 8 | 8.0 | MCP agent layer: 52-tool execution engine, embedded HTTP/SSE server (port 19789), in-app chat orchestrator, Tauri IPC | [`8-0-mcp-agent-layer.md`](HISTORY/8-0-mcp-agent-layer.md) |

## Handover & Continuation

- **Completed Milestone**: Phase 8.0 (MCP Agent Layer) implemented, tested, and integrated. Ready for Phase 9.0 (Audio Engine — multi-track mixer, stem separation, ducking, meter bridge, Web Audio/WASAPI pipeline).

