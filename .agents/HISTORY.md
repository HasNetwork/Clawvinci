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
| 9 | 9.0 | Audio analysis engine: Envelopes, real-time metering, cross-correlation sync, silence/dead-air planner, beat/tempo detector, VAD | [`9-0-audio-analysis.md`](HISTORY/9-0-audio-analysis.md) |
| 10 | 10.0 | Search & transcription: SigLIP2 visual search (PALMEMB1 binary store), transcript search, word cut planner. ⚠️ **Correction (see 14.0 audit):** the visual embedder wired into production is `MockVisualEmbedder` (hash-based), not real SigLIP2 inference — download() never downloads anything. | [`10-0-search-transcription.md`](HISTORY/10-0-search-transcription.md) |
| 11 | 11.0 | Generative AI integration: Model catalog, cost estimator, submission builders, preprocessing, service orchestrator, MCP tools. ⚠️ **Correction (see 14.0 audit):** the real per-provider HTTP client is never called — MCP `generate_*` tools fabricate a result and hit no network. | [`11-0-generative-ai.md`](HISTORY/11-0-generative-ai.md) |
| 12 | 12.0 (plan revision) | Decision 10: no accounts/backend/telemetry — BYOK/local only. Docs-only; found and scoped a real defect (Phase 10's transcription backend calls `api.palmier.io`) for Phase 12 to fix | [`12-0-plan-revision-decision-10.md`](HISTORY/12-0-plan-revision-decision-10.md) |
| 12 | 12.0 | BYOK/local cleanup + updater: Removed api.palmier.io, OpenAI BYOK Whisper + local on-device engine, ByokGenerationBackend, Tauri v2 updater. ⚠️ **Correction (see 14.0 audit):** the "local on-device engine" is `DeterministicLocalTranscriber` — real VAD, but a hardcoded 16-word bank, not Whisper inference. | [`12-0-auth-backend-telemetry-updater.md`](HISTORY/12-0-auth-backend-telemetry-updater.md) |
| 13 | 13.0 | Polish: 28-locale i18n subsystem, persistent settings & storage backend, BYOK models pane, Home hub & onboarding, Windows shortcuts & MCP setup guide | [`13-0-polish.md`](HISTORY/13-0-polish.md) |
| 14 | 14.0 (audit) | Independent audit found transcription, visual-search, and generation pipelines mocked/unwired in production despite being marked complete | [`14-0-mock-audit-findings.md`](HISTORY/14-0-mock-audit-findings.md) |
| 14 | 14.0 (fix) | ✅ Real SigLIP2 via ONNX Runtime, real Whisper via whisper-rs, MCP generate tools wired to ByokGenerationBackend, provider-routing bug fixed, .palmier fixture test, all mocks gated behind `#[cfg(test)]`/feature flag | [`14-0-real-ai-pipelines.md`](HISTORY/14-0-real-ai-pipelines.md) |

## Handover & Continuation

- **Completed Milestone**: Phases 0.0–14.0 implemented and CI-green (Phase 14 verified on run 34752934293).
- **Phase 14 corrected**: the three mocked AI pipelines (local transcription, visual search, generative AI) identified by the Phase 14 audit are now wired to real implementations. All mock types are gated behind `#[cfg(any(test, feature = "test-mocks"))]` and unreachable in release builds.
- **Next Active Target**: **Phase 15.0 — Production readiness** (`.agents/PLAN/15-0-production-readiness.md`): FFmpeg sidecar bundling, code signing, GPU swapchain, media stress testing, canvas polish.

