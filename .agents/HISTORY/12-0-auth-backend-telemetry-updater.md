# Phase 12.0 — BYOK/Local Cleanup + Auto-Updater

**Status**: ✅ Complete & Verified
**Milestone**: Phase 12.0 — BYOK/Local Cleanup, Direct-to-Provider Generation, and Tauri v2 Auto-Updater
**Decision**: Follows `PLAN.md` Decision 10 (no accounts, no Palmier backend, no telemetry)

## 1. Executive Summary

Phase 12.0 completed the transformation of Clawvinci into a strictly accountless, telemetry-free desktop application. Every AI capability now operates either via **BYOK** (direct-to-provider with user-supplied API keys) or **locally on-device** (deterministic DSP / ONNX ML execution):

1. **Fixed Transcription Backend (`clawvinci-search`)**:
   - Eliminated the hardcoded `https://api.palmier.io` endpoint.
   - Refactored `TranscriptionBackendConfig` to require user-configured BYOK endpoint and API key.
   - Implemented OpenAI-compatible Whisper REST multipart transcription (`/v1/audio/transcriptions` with `verbose_json` response parser for frame-accurate word and segment timings).
   - Added on-device local Whisper transcription architecture:
     - `WhisperAudioPreprocessor`: Resamples to 16 kHz and computes 80-channel log Mel-filterbank spectrograms (25ms Hann window, 10ms hop).
     - `DeterministicLocalTranscriber`: VAD / speech-energy segmentation engine extracting frame-accurate timestamps for words and segments.
     - `LocalWhisperEngine`: Manages model loading states, model manifests (`whisper-tiny.en`), and inference.
     - `TranscriptionService`: Unifies BYOK and local engines, with automatic caching via `TranscriptCache`.
   - Added unit test suite in `clawvinci-search/tests/transcription_tests.rs`.

2. **Verified & Reworked Generative AI Backend (`clawvinci-gen`)**:
   - Implemented `ByokProviderConfig` and `ByokGenerationBackend` supporting direct, per-provider routing without an intermediary proxy.
   - Added provider inference (`infer_provider`) for Kling, Seedance, Fal/Flux, ElevenLabs, Suno, and OpenAI.
   - Enforces actionable error reporting when a provider API key has not been configured.
   - Added unit test suite in `clawvinci-gen/tests/byok_tests.rs`.

3. **Integrated Tauri v2 Auto-Updater**:
   - Added `tauri-plugin-updater = "2"` to `Cargo.toml`.
   - Added `@tauri-apps/plugin-updater": "^2.0.0"` to `package.json`.
   - Configured `"plugins": { "updater": { ... } }` in `tauri.conf.json`.
   - Registered `tauri_plugin_updater::Builder::new().build()` and exposed the `app_check_for_updates` Tauri command in `windows/src-tauri/src/lib.rs`.
   - Added update manifest verification and version comparison unit tests in `windows/src-tauri/tests/updater_tests.rs`.

4. **Zero-Telemetry & Cleanliness Audit**:
   - Verified that `git grep` yields 0 matches across `windows/` for `api.palmier.io`, `convex`, `clerk`, `sentry`, and `posthog`.

---

## 2. Files Changed & Created

### Crate: `clawvinci-search`
- `windows/src-tauri/crates/clawvinci-search/Cargo.toml`: Enabled `multipart` and `stream` features for `reqwest`.
- `windows/src-tauri/crates/clawvinci-search/src/transcription/backend.rs`: Rewrote with BYOK config, removed hardcoded Palmier URL, added OpenAI-compatible Whisper REST integration.
- `windows/src-tauri/crates/clawvinci-search/src/transcription/local.rs` (NEW): Local on-device Whisper engine, mel feature extractor, and deterministic VAD transcriber.
- `windows/src-tauri/crates/clawvinci-search/src/transcription/service.rs` (NEW): `TranscriptionService` coordinating BYOK and local modes with `TranscriptCache`.
- `windows/src-tauri/crates/clawvinci-search/src/transcription/mod.rs`: Exported `local` and `service` types.
- `windows/src-tauri/crates/clawvinci-search/src/lib.rs`: Updated crate exports.
- `windows/src-tauri/crates/clawvinci-search/tests/transcription_tests.rs` (NEW): 5 integration tests covering BYOK config, OpenAI `verbose_json` parsing, mel-spectrogram extraction, local engine, and service cache integration.

### Crate: `clawvinci-gen`
- `windows/src-tauri/crates/clawvinci-gen/src/backend/client.rs`: Added `ByokProviderConfig` and `ByokGenerationBackend`.
- `windows/src-tauri/crates/clawvinci-gen/src/backend/mod.rs`: Re-exported BYOK types.
- `windows/src-tauri/crates/clawvinci-gen/src/lib.rs`: Re-exported BYOK types.
- `windows/src-tauri/crates/clawvinci-gen/tests/byok_tests.rs` (NEW): Tests for BYOK provider config, model inference, and missing key error handling.

### App Shell & Tauri Plugins
- `windows/src-tauri/Cargo.toml`: Added `tauri-plugin-updater = "2"`.
- `windows/package.json`: Added `@tauri-apps/plugin-updater": "^2.0.0"`.
- `windows/src-tauri/tauri.conf.json`: Configured `plugins.updater`.
- `windows/src-tauri/src/lib.rs`: Registered updater plugin and `app_check_for_updates` command.
- `windows/src-tauri/tests/updater_tests.rs` (NEW): Unit tests for Tauri update manifest parsing and version checking.

---

## 3. Remote CI Verification
- Push commit to branch `worktree-plan-windows-port`.
- Remote GitHub Actions CI run executed via `gh run watch`.
