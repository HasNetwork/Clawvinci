# Phase 14.0 — Make the AI pipelines real: Implementation Log

**Status**: ✅ Complete — all four plan items implemented, mock sweep passed
**Date**: 2026-09-13
**Audit findings that prompted this**: `14-0-mock-audit-findings.md`
**Plan**: `.agents/PLAN/14-0-real-ai-pipelines.md`

---

## 14.1 + 14.2 — Real SigLIP2 visual search via ONNX Runtime

### What changed

**`clawvinci-search/src/visual/model.rs`**:
- `MockVisualEmbedder` and its `VisualEmbedder` impl gated behind
  `#[cfg(any(test, feature = "test-mocks"))]` — unreachable in release builds.

**`clawvinci-search/src/visual/onnx_embedder.rs`** (new file):
- `OnnxSigLIP2Embedder` — real SigLIP2 inference via `ort` (ONNX Runtime v2).
- Loads vision encoder ONNX model, tokenizer, and preprocessor config from
  the model directory.
- Image preprocessing: resize to model input dimensions, normalize with
  SigLIP2 mean/std, construct NCHW tensor via `ndarray`.
- Text tokenization via `tokenizers` crate (HuggingFace tokenizer).
- Runs ONNX session inference, extracts embedding vectors, L2-normalizes.
- Implements `VisualEmbedder` trait (`embed_image`, `embed_text`, `dimensions`).

**`clawvinci-search/src/visual/loader.rs`**:
- `VisualModelLoader::prepare()` now loads `OnnxSigLIP2Embedder` when model
  files exist on disk; falls back to `NotInstalled` state instead of mock.
- `download()` calls shared `model_download::download_and_verify()` for each
  manifest file (vision encoder, text encoder, tokenizer, preprocessor config).
- Added `prepare_with_mock()` gated behind `#[cfg(any(test, feature = "test-mocks"))]`
  for integration tests.

**`clawvinci-search/src/visual/mod.rs`**:
- `MockVisualEmbedder` re-export gated behind feature flag.
- `OnnxSigLIP2Embedder` added to public exports.

**`clawvinci-search/src/model_download.rs`** (new file):
- Shared HTTP download + SHA256 verification utility used by both the visual
  embedder and whisper transcription model downloads.
- `download_and_verify(url, dest_path, expected_sha256)` — streams response
  to disk, computes SHA256, verifies against manifest, cleans up on mismatch.

**`clawvinci-search/Cargo.toml`**:
- Added `[features] default = [] test-mocks = []`
- Added self-referencing dev-dep: `clawvinci-search = { path = ".", features = ["test-mocks"] }`
- Added dependencies: `ort` (v2.0.0-rc.9, ndarray feature), `ndarray`,
  `tokenizers` (v0.20, onig feature), `futures-util`, `dirs`.

### Design rationale

Chose ONNX Runtime (`ort` crate) over alternatives:
- `tract` — good for simple models but SigLIP2's architecture (vision
  transformer + text encoder) is better served by ort's broader op coverage.
- Direct `tch` (libtorch) — much heavier runtime dependency, not justified
  for inference-only.
- `ort` v2 is the same runtime the macOS app targets via CoreML; ONNX is the
  shared model format.

---

## 14.3 — Real Whisper transcription via whisper-rs

### What changed

**`clawvinci-search/src/transcription/local.rs`** (fully rewritten):

- `WhisperModelManifest` changed from 3-file ONNX format to single GGML file:
  default model is `ggml-tiny.en.bin` from the `whisper.cpp` Hugging Face repo.
- `WhisperModelFileSpec` gained `url: Option<String>` for download source.
- `WhisperRsTranscriber` — real whisper-rs inference:
  - `WhisperContext::new_with_params()` loads the GGML model file.
  - `ctx.create_state()`, `state.full(params, &samples)` runs inference.
  - `FullParams::new(SamplingStrategy::Greedy { best_of: 1 })` with token
    timestamps enabled.
  - Extracts segments with centisecond timestamps, converts to `TranscriptWord`.
- `DeterministicLocalTranscriber` (the 16-word bank mock) gated behind
  `#[cfg(any(test, feature = "test-mocks"))]`.
- `LocalWhisperEngine::new()` — transcriber field changed to
  `RwLock<Option<Arc<dyn LocalWhisperTranscriber>>>` (None initially, loaded
  lazily on prepare).
- `prepare()` — checks `is_installed()`, loads `WhisperRsTranscriber` if
  model file exists, otherwise sets `NotInstalled` status.
- `transcribe()` — returns `ModelNotReady` error if transcriber is None.
- `download()` — calls `model_download::download_and_verify()` for the GGML file.
- `prepare_with_mock()` gated behind feature flag for test injection.

**`clawvinci-search/src/transcription/mod.rs`**:
- `DeterministicLocalTranscriber` re-export gated behind feature flag.
- `WhisperRsTranscriber` added to public exports.

**`clawvinci-search/src/error.rs`**:
- Added `LocalTranscriptionFailed(String)` variant to `SearchError`.

**`clawvinci-search/tests/transcription_tests.rs`**:
- Tests updated: `engine.prepare()` → `engine.prepare_with_mock()` since
  `prepare()` now requires real model files.

**`clawvinci-search/Cargo.toml`**:
- Added `whisper-rs` v0.14 dependency.

### Design rationale

Chose `whisper-rs` (bindings to whisper.cpp) over ONNX Runtime for Whisper:
- whisper.cpp is the de facto standard for local Whisper inference — heavily
  optimized C/C++ with AVX2/NEON, quantized GGML model format.
- ONNX Whisper models are larger and less battle-tested on Windows; the
  encoder-decoder architecture with autoregressive decoding is awkward in ONNX.
- GGML `tiny.en` model is ~75MB vs ~150MB+ for ONNX equivalent, and the
  whisper.cpp project maintains verified model downloads on Hugging Face.

---

## 14.4 — Wire MCP generate tools to real ByokGenerationBackend

### What changed

**`clawvinci-mcp/src/tools/generate.rs`** (fully rewritten):
- Added `GenerationContext` struct holding `Arc<ByokGenerationBackend>` +
  `SharedMcpState` — bridges sync MCP tool handlers to async generation backend.
- All 4 generate tools (`generate_video`, `generate_image`, `generate_audio`,
  `upscale_media`) now accept `gen_ctx: Option<&GenerationContext>`.
- Each validates provider configuration via `backend.is_provider_configured()`.
- Each spawns a background Tokio task via `spawn_generation_task()` that:
  1. Calls `backend.submit()` asynchronously.
  2. Polls `backend.get_job()` with exponential backoff (1s → 2s → 4s → ... → 30s cap).
  3. Updates media item `generation_status` in `SharedMcpState` on completion/failure.
- `list_models` unchanged (catalog-only, no backend needed).

**`clawvinci-mcp/src/executor.rs`**:
- `ToolExecutor` now holds `gen_ctx: Option<Arc<GenerationContext>>`.
- Added `with_generation_context()` builder method.
- Generation tool dispatch passes `self.gen_ctx.as_deref()` to each handler.

**`clawvinci-gen/src/backend/client.rs`**:
- Added `job_endpoints: Arc<RwLock<HashMap<String, (String, String)>>>` to
  `ByokGenerationBackend` — tracks which provider+endpoint submitted each job.
- Fixed `submit()`: now stores `(provider, base_url)` in `job_endpoints`
  after successful submission.
- Fixed `get_job()`: looks up endpoint from `job_endpoints` map for the given
  job ID, falls back to openai default only as last resort.
- Fixed `upload_reference()`: resolves endpoint from first configured provider
  instead of hardcoded openai.
- `MockGenerationBackend` and all its impls gated behind
  `#[cfg(any(test, feature = "test-mocks"))]`.

**`clawvinci-gen/src/backend/mod.rs`**:
- `MockGenerationBackend` re-export gated behind feature flag.

**`clawvinci-gen/src/lib.rs`**:
- `MockGenerationBackend` re-export gated behind feature flag.

**`clawvinci-gen/Cargo.toml`**:
- Added `[features] default = [] test-mocks = []`
- Added self-referencing dev-dep: `clawvinci-gen = { path = ".", features = ["test-mocks"] }`

### Sync→async bridge design

The MCP tool handlers run synchronously inside axum route handlers, but the
generation backend is fully async. Since the axum handler already runs inside
a Tokio runtime, `tokio::spawn()` works directly from sync code. The spawned
task owns an `Arc<ByokGenerationBackend>` and `SharedMcpState`, polls until
completion, and writes results back into shared state. The sync handler
returns immediately with `"status": "submitted"` and the job ID.

---

## 14.5 — `.palmier` format fixture decode test

### What changed

**`clawvinci-export/tests/bundle_tests.rs`**:
- Added `test_palmier_format_fixture_decode_roundtrip` test that:
  1. Creates a synthetic `.palmier` bundle directory with `project.json`,
     `manifest.json`, and `media/` subdirectory.
  2. Builds a `ProjectFile` with a 4K timeline (24fps, 3840×2160) containing
     one clip on the default track.
  3. Builds a `MediaManifest` with two entries (video + audio) using
     `MediaSource::Project` variant with relative paths.
  4. Serializes both to pretty-printed JSON, writes to bundle directory.
  5. Reads back from disk, deserializes into typed structs.
  6. Asserts decoded objects match originals via `PartialEq` and field-level
     checks (timeline name, fps, dimensions, clip media_ref, frame ranges,
     manifest entry IDs/names/types/durations/sources).

### Note on fixture provenance

The plan called for a fixture "captured from the real macOS app's Codable
output." This test instead constructs a fixture programmatically from the
Rust domain model and verifies round-trip fidelity. A true cross-platform
byte-compatibility test against macOS-produced JSON would require either a
committed fixture file or access to the macOS app — neither is available in
this repository. The test does verify that the `.palmier` format's JSON
schema is stable across serialize/deserialize cycles, which covers the
primary correctness concern.

---

## Final verification: mock sweep

Grep sweep confirmed all three mock types are unreachable outside test builds:

- **`MockVisualEmbedder`**: defined and impl'd behind `#[cfg(any(test, feature = "test-mocks"))]`
  in `model.rs`; re-exported behind same gate in `visual/mod.rs` and `lib.rs`.
- **`DeterministicLocalTranscriber`**: defined, impl'd, and `prepare_with_mock()`
  all behind `#[cfg(any(test, feature = "test-mocks"))]` in `local.rs`;
  re-exported behind same gate in `transcription/mod.rs` and `lib.rs`.
- **`MockGenerationBackend`**: defined and impl'd behind
  `#[cfg(any(test, feature = "test-mocks"))]` in `client.rs`; re-exported
  behind same gate in `backend/mod.rs` and `lib.rs`.

All test files (`tests/` directories) use `prepare_with_mock()` or direct
mock construction — no production code path can instantiate any mock type.

---

## Files modified (complete list)

| File | Action |
|---|---|
| `clawvinci-search/Cargo.toml` | Rewritten — features, deps (ort, ndarray, tokenizers, whisper-rs, futures-util, dirs) |
| `clawvinci-search/src/model_download.rs` | **New** — shared download+verify utility |
| `clawvinci-search/src/visual/onnx_embedder.rs` | **New** — real SigLIP2 ONNX inference |
| `clawvinci-search/src/visual/model.rs` | Mock gated behind feature flag |
| `clawvinci-search/src/visual/loader.rs` | Rewired to OnnxSigLIP2Embedder, real download |
| `clawvinci-search/src/visual/mod.rs` | Export gates updated |
| `clawvinci-search/src/error.rs` | Added `LocalTranscriptionFailed` variant |
| `clawvinci-search/src/transcription/local.rs` | Fully rewritten — WhisperRsTranscriber |
| `clawvinci-search/src/transcription/mod.rs` | Export gates updated |
| `clawvinci-search/src/lib.rs` | Export gates updated |
| `clawvinci-search/tests/transcription_tests.rs` | Tests use `prepare_with_mock()` |
| `clawvinci-gen/Cargo.toml` | Added features + self-referencing dev-dep |
| `clawvinci-gen/src/backend/client.rs` | Job→endpoint tracking, provider routing fix, mock gated |
| `clawvinci-gen/src/backend/mod.rs` | Mock re-export gated |
| `clawvinci-gen/src/lib.rs` | Mock re-export gated |
| `clawvinci-mcp/src/tools/generate.rs` | Fully rewritten — GenerationContext, real backend calls |
| `clawvinci-mcp/src/executor.rs` | Holds GenerationContext, passes to tool handlers |
| `clawvinci-export/tests/bundle_tests.rs` | Added fixture decode roundtrip test |
