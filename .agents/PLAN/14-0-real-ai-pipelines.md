# Phase 14.0 — Make the AI pipelines real

**Why this phase exists**: an independent audit (`.agents/HISTORY/14-0-mock-audit-findings.md`)
found that three subsystems marked "complete" in Phases 10–12 are mocked or
unwired at the actual production call site — not test fixtures, but what the
running app calls at runtime. Clawvinci's entire identity is "AI-native video
editor"; these three *are* that identity. This phase makes each one real
before anything else (Phase 15's packaging/distribution work) is worth doing.

This is not new scope — it's finishing scope that was already signed off as
done. Do not repeat the mistake: **a mock reachable from a non-test code
path is not a valid Definition-of-done**, regardless of what CI says (CI
only proves compilation + whatever tests exist; it said nothing about
whether `MockVisualEmbedder` was wired into production, because no test
checked *that*). Every item below requires demonstrating real output against
real input before it's marked done — see the strengthened Verification rule
in `RULES.md`.

## 1. Local on-device transcription — currently fabricated

`clawvinci-search/src/transcription/local.rs`: `LocalWhisperEngine::prepare()`
unconditionally builds `DeterministicLocalTranscriber`, which does real
VAD/energy detection but then emits words from a **hardcoded 16-word bank**
regardless of actual speech content. No ONNX/`ort`/`tract`/`whisper-rs`
dependency exists in the crate. `WhisperModelManifest` lists model SHA256s
but there is no code path that ever downloads them.

- Add a real ONNX Runtime (`ort` crate) or `whisper-rs` inference path.
  Source a genuine Whisper-class ONNX export (e.g. a published
  `whisper.cpp`-compatible ONNX conversion, or export one via Hugging Face
  `optimum` from an open Whisper checkpoint) — there's no existing
  conversion script in `models/` to retarget for this one, unlike
  `beat_this`/SigLIP2, so budget real time for sourcing/validating it.
- Implement the actual model download (HTTP GET against the manifest's
  URLs, verify against the listed SHA256, write to
  `%LOCALAPPDATA%/Clawvinci/Models`) — `download()` must stop being a
  no-op.
- `DeterministicLocalTranscriber` may stay, but only reachable from
  `#[cfg(test)]` or behind an explicit test-only feature flag — never the
  default `prepare()` path in a release build.
- Keep the BYOK cloud path (OpenAI Whisper API) as-is if it already does a
  real HTTP call — verify it does, don't assume.

## 2. SigLIP2 visual search — currently a hash, not an embedding

`clawvinci-search/src/visual/loader.rs`: `VisualModelLoader::prepare()`
unconditionally instantiates `MockVisualEmbedder` (a hash-based
pseudo-embedding) and never checks `is_installed()`. `download()` is
literally commented "Simulates model downloading." `SearchIndexCoordinator`
has no way to ever get a real embedder.

- Wire real ONNX Runtime inference for the SigLIP2 checkpoint (the
  `models/siglip2/convert.py`/`export_tokenizer.py` pipeline this phase's
  original doc pointed at — re-verify that reuse plan still holds).
- Implement the real model download (same pattern as item 1's manifest
  download — share the download/verify code between the two rather than
  writing it twice).
- `MockVisualEmbedder` moves behind `#[cfg(test)]` — confirm nothing in
  `coordinator.rs` or any non-test path can reach it after the fix.
- Re-run the "cosine-similarity search over a small known media set"
  parity check the original Phase 10 doc required — this was apparently
  never actually done against a real embedder, only against the mock.

## 3. Generative AI — real client exists, never called

`clawvinci-gen/src/backend/client.rs` has genuine per-provider endpoints
(Kling, ElevenLabs, Suno, Fal, OpenAI) and is correctly structured — the gap
is entirely at the call site. `clawvinci-mcp/src/tools/generate.rs`'s
`generate_video`/`generate_image`/`generate_audio`/`upscale_media` fabricate
a UUID and a `gen-{id}.mp4`-style filename synchronously, write nothing to
disk, and return `"status":"ready"` immediately — zero network calls happen.

- Wire `generate.rs`'s tool handlers to actually call
  `clawvinci_gen::service::GenerationService` (submit → poll/subscribe →
  download → insert clip), matching what `clawvinci-gen`'s own Phase 11
  Definition of done already required and apparently never got connected
  end-to-end through the MCP layer.
- **Fix the real bug found alongside this**: `ByokGenerationBackend::get_job`
  and `::upload_reference` hardcode the OpenAI endpoint regardless of which
  provider actually submitted the job (`client.rs:224-227, 249-251`) — route
  based on the job's actual provider, not a fixed default.
- Re-run Phase 11's own Definition of done: "at least one real end-to-end
  generation round-trip against a live provider API" — this needs to
  actually happen now, with a real user-supplied key, not a mock backend.

## 4. `.palmier` byte-compatibility — currently unverified, not confirmed

Existing tests only round-trip Rust-encoded data against itself
(`roundtrip_tests.rs`) or hand-write a JSON literal asserting field names
(`legacy_compat_tests.rs`) — no fixture captured from the real macOS app's
actual `Codable` JSON output exists anywhere in the repo. This is lower
severity than items 1-3 (nothing user-facing is silently fake), but the
original Phase 1 Definition of done specifically called for this and it
wasn't done.

- Get (or faithfully reconstruct, field-by-field, from reading the actual
  Swift `Codable` conformances and their `CodingKeys`) at least one real
  `project.json` fixture matching what the macOS app would actually
  produce, and add a fixture-based decode test — not another
  self-round-trip.

## Definition of done for Phase 14

- Local transcription: a real audio file with actual speech, run through
  the local ONNX path, produces words matching what was said (not the
  16-word bank) — and through the BYOK path against a real API key.
- Visual search: a real embedder produces real SigLIP2 embeddings; a
  cosine-similarity query over a handful of real images returns sensible
  results (spot-check parity against the macOS app's search for the same
  query, per the original Phase 10 doc).
- Generation: at least one real end-to-end round trip (submit → poll →
  download → timeline insertion) against one live provider using a real
  user-supplied key. Provider-routing bug in `get_job`/`upload_reference`
  fixed and tested against more than one provider.
- `.palmier` format: at least one fixture-based decode test against a real
  (or faithfully reconstructed) macOS-produced `project.json`.
- Grep sweep confirms `DeterministicLocalTranscriber` and
  `MockVisualEmbedder` are unreachable outside `#[cfg(test)]`/test-only
  builds.
- `.agents/HISTORY.md` and `.agents/Handover.md` corrected to reflect
  actual state at every step — do not mark this phase done until every
  item above is independently demonstrable, not just "compiles and passes
  the tests that were written for it" (see the strengthened Verification
  rule in `RULES.md`).
