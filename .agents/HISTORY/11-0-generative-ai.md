# Phase 11.0 — Generative AI Integration (`clawvinci-gen`)

**Status**: ✅ Complete  
**CI Verification**: ✅ 100% Green on GitHub Actions Run `34208682487` (Check, Clippy, Test, Tauri Build & Artifact Upload)  
**Clean Commit**: `fb7402c`  
**Date**: 2026-09-08  

## What was built

Phase 11.0 implements generative AI provider catalogs, job submission payloads, media preprocessing, backend clients, and timeline lifecycle integration from `Sources/PalmierPro/Generation/` (36 files, 6,880 LOC) into the native Rust crate `clawvinci-gen`:

1. **Declarative Model Catalog & Capabilities (`src/catalog/models.rs`):**
   - Direct port of `ModelCatalog.swift`, `VideoModelConfig.swift`, `ImageModelConfig.swift`, `AudioModelConfig.swift`, and `UpscaleModelConfig.swift`.
   - Modalities: `Video`, `Image`, `Audio`, `Music`, `Upscale`.
   - Modality-specific capability structs: `VideoCaps`, `ImageCaps`, `AudioCaps`, `UpscaleCaps` capturing supported durations, resolutions, aspect ratios, reference counts, speed tiers, and voice lists.
   - Offline default catalog: Built-in registry preloaded with standard SOTA models (`seedance-v1-fast`, `kling-v1`, `nano-banana-pro`, `flux-1-schnell`, `eleven-multilingual-v2`, `suno-v3-5`, `real-esrgan-4x`).
   - Dynamic catalog updates: Supports runtime deserialization and indexing from backend responses.

2. **Accurate Credit Cost Estimator (`src/catalog/cost.rs`):**
   - Direct port of `CostEstimator.swift`.
   - Video cost: Base and resolution-specific rates, draft discounts, audio exclusion discounts (e.g. 0.85x), and source video rates.
   - Image cost: 2D resolution/quality matrix lookups (`"1024x1024|hd"`), quality-only fallbacks, and multi-image multipliers.
   - Speech cost: Pricing by character count (per 1,000 characters) or duration seconds.
   - Music cost: Span duration calculations based on rate per second.
   - Upscale cost: Duration scaling with 4K / high-scale multipliers.
   - Ceiling rounding: Integer credit quantization matching Apple implementation (`ceilCredits`).

3. **User Model Preferences (`src/catalog/preferences.rs`):**
   - Direct port of `ModelPreferences.swift`.
   - Manages disabled model IDs and default model selection per modality (`video`, `image`, `speech`, `music`, `upscale`).

4. **Structured Submission Payload Builders (`src/submission/`):**
   - Direct port of `VideoGenerationSubmission.swift`, `ImageGenerationSubmission.swift`, `AudioGenerationSubmission.swift`, and `MusicGenerationSubmission.swift`.
   - Strong type contracts for wire parameters: `VideoGenerationParams`, `ImageGenerationParams`, `AudioGenerationParams`, `UpscaleGenerationParams`.
   - Unified enum `BackendGenerationParams` serializing to backend JSON.
   - Assembly builders for reference assets, prompt formatting, aspect ratio configuration, and duration binding.

5. **Local Media Preprocessing Pipeline (`src/preprocessing/`):**
   - Direct port of `VideoTrimExtractor.swift`, `AudioTrackExtractor.swift`, and `ImageConverter.swift`.
   - `VideoTrimExtractor`: Utilizes `clawvinci_media::ffmpeg::FfmpegContext` to slice sub-frame ranges into temporary MP4 references.
   - `AudioTrackExtractor`: Extracts PCM audio tracks to WAV/M4A for speech synthesis and video-to-music generation.
   - `ImageConverter`: Automatically detects incompatible image formats (`.heic`, `.heif`, `.tiff`, `.bmp`) and transcodes them to standard JPEGs.

6. **Backend Client Architecture (`src/backend/`):**
   - Direct port of `GenerationBackend.swift`.
   - `GenerationBackendClient` trait supporting asynchronous job submission, status polling, reference uploading, and file streaming.
   - `HttpGenerationBackend`: Production client using `reqwest` for REST/Convex endpoints with token authorization.
   - `MockGenerationBackend`: Thread-safe, in-memory client providing deterministic offline test execution and synthetic media generation without network dependency.

7. **Generation Service & Timeline Orchestrator (`src/service.rs`):**
   - Direct port of `GenerationService.swift`.
   - Placeholder creation: Generates `MediaManifestEntry` with `"gen-<short_id>.<ext>"` naming, initial duration, and `"preparing"` status.
   - Job lifecycle: Submits request to backend, updates manifest to `"generating"`, polls status with backoff.
   - Finalization: Downloads generated result, probes media properties (duration, resolution, fps, audio presence) via `clawvinci_media::probe::probe_media`, updates manifest to `"ready"`, and records cost/refund receipts.
   - Cancellation: Cooperatively checks `tokio_util::sync::CancellationToken`, immediately halts downloads, and marks status as `"cancelled"`.
   - Timeline insertion helper: Seamlessly places clips on active `Timeline` tracks.

8. **AI-Driven Edit Actions (`src/edit/mod.rs`):**
   - Direct port of `EditAction.swift`.
   - `EditActionKind`: `Upscale`, `Edit`, `Rerun`, `LipSync`, `Reframe`, `GenerateMusic`, `GenerateSfx`, `CreateVideo`, `EnhanceDraft`.
   - Evaluates availability rules and plan tiers per media type.

9. **MCP Agent Layer Integration (`clawvinci-mcp`):**
   - Added `clawvinci-gen` dependency to `clawvinci-mcp`.
   - `list_models`: Returns complete, structured catalogs for Video, Image, Speech, Music, and Upscale models directly from `ModelCatalog::default_catalog()`.
   - `generate_video`: Validates model parameters, computes estimated credit costs, creates placeholder `McpMediaItem`s with `"generating"` status.
   - `generate_image`: Computes credit costs and creates placeholder items.
   - `generate_audio`: Routes speech and music requests, computes credit costs, registers items.
   - `upscale_media`: Computes upscale factors and costs, creates upscaled media references.

10. **Comprehensive Test Suite (`tests/gen_tests.rs`):**
    - 8 unit and integration tests covering:
      - Catalog default models and capability inspections.
      - Model preferences management.
      - Video, image, speech, music, and upscale cost estimation.
      - JSON submission serialization.
      - Mock backend job lifecycle, polling, and file downloads.
      - Service placeholder creation, manifest registration, finalization, and timeline clip placement.
      - Service cancellation via `CancellationToken`.
      - Edit action availability matrices.
      - Preprocessing timestamp math and image format conversion detection.

## Verification
- **GitHub Actions CI**: Run `34208682487` passed 100% green across all steps (Check, Clippy with zero warnings, Tests, and Tauri Windows x64 build).
