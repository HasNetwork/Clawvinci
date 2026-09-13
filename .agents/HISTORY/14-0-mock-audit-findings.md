# Phase 14.0 — Audit findings: mocked AI pipelines marked complete

**Status**: 🔴 Correction to the record — three subsystems documented as
complete in Phases 10–12 are not functional in production
**Date**: 2026-09-13
**Trigger**: independent post-completion audit requested by the user after
all 14 phases (0–13) were reported done and CI-green

## What was checked and why

After Phase 13 shipped, the user asked to independently verify specific
claims in `.agents/HISTORY.md`/`.agents/Handover.md` against the actual
code, rather than trust the self-reported "complete"/"CI Green" status. CI
passing only proves compilation + whatever tests were written — it doesn't
prove a mock isn't silently standing in for the real implementation at the
call site a real user would hit. That's exactly what was found.

## Findings

1. **Local transcription — `clawvinci-search/src/transcription/local.rs`**:
   `LocalWhisperEngine::prepare()` (line ~296) unconditionally builds
   `DeterministicLocalTranscriber`, which does real VAD/energy detection
   but then emits words from a hardcoded 16-word bank (`"the"`, `"video"`,
   `"editor"`, `"timeline"`, ...) regardless of actual audio content. No
   ONNX/`ort`/`tract`/`whisper-rs` dependency exists in the crate's
   `Cargo.toml`. `WhisperModelManifest` lists model SHA256s but no code
   path ever downloads them.
2. **SigLIP2 visual search — `clawvinci-search/src/visual/loader.rs`**:
   `VisualModelLoader::prepare()` (line ~126) unconditionally builds
   `MockVisualEmbedder` — a hash-based pseudo-embedding, not SigLIP2
   inference — and never checks `is_installed()`. `download()` (line
   ~150) is commented "Simulates model downloading" and just calls
   `prepare()`. `SearchIndexCoordinator` (`coordinator.rs:106`) has no way
   to ever obtain a real embedder at runtime.
3. **Generative AI — `clawvinci-mcp/src/tools/generate.rs`**: the real
   per-provider HTTP client (`clawvinci-gen/src/backend/client.rs`,
   correct endpoints for Kling/ElevenLabs/Suno/Fal/OpenAI) is never
   called. `generate_video`/`generate_image`/`generate_audio`/
   `upscale_media` synchronously fabricate a UUID and a filename, write no
   file, and return `"status":"ready"` immediately — zero network calls.
   Additional real bug found in the unwired client itself:
   `ByokGenerationBackend::get_job`/`::upload_reference` hardcode the
   OpenAI endpoint regardless of which provider actually submitted the job
   (`client.rs:224-227, 249-251`).
4. **`.palmier` byte-compatibility — lower severity**: existing tests only
   round-trip Rust-encoded data against itself, or hand-assert field names
   in a hand-written JSON literal. No fixture captured from the real
   macOS app's `Codable` output exists anywhere in the repo, so "byte-
   compatible" (Decision 9) is unverified, not confirmed.

## Confirmed NOT affected (checked in the same audit)

- Unified render path (`composite_frame` shared by preview and export) —
  genuinely one code path, no fork.
- Decision 10 (no Palmier backend/accounts/telemetry) — clean, zero
  surviving references anywhere in `windows/`.
- FCPXML/XMEML export — genuinely spec-shaped output.
- MCP tool count — actual registered count is **52**, not the 53 several
  docs still state (macOS reference had 53; one didn't make it across, or
  two got merged during porting — not yet investigated, tracked as a
  checklist item in Phase 14's doc if worth resolving).

## Root cause (for the record, not to assign blame — to prevent repeat)

Each of the three mocked subsystems has a real, correctly-structured
implementation sitting right next to the mock (a real HTTP client with
correct endpoints; real ONNX-shaped model-loading scaffolding). The mocks
were almost certainly built first as a way to keep the surrounding
orchestration code (`SearchIndexCoordinator`, `GenerationService`, the MCP
tool handlers) testable without real models/network access — a reasonable
intermediate step — but were never swapped out for the real thing before
the phase was marked done, and no test existed that would have caught a
mock reaching a production code path. `RULES.md` now has a strengthened
Verification rule targeting exactly this failure mode.

## What changed as a result

- `.agents/PLAN/14-0-real-ai-pipelines.md` (new) — the fix plan for items
  1–4 above.
- `.agents/PLAN/15-0-production-readiness.md` (new) — formalizes
  `Handover.md`'s post-Phase-13 roadmap paragraph as a real tracked phase,
  sequenced after Phase 14.
- `.agents/PLAN.md`, `.agents/HISTORY.md`, `.agents/Handover.md` — updated
  to stop reporting these three subsystems as complete; point at Phase 14.
- `.agents/RULES.md` — added: a mock reachable from a non-test code path
  is never a valid Definition-of-done, regardless of CI status.
