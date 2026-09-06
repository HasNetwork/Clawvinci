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
