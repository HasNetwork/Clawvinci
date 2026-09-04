# Phase 11.0 — Generative AI integration (`clawvinci-gen`)

The "generate videos and images with SOTA models like Seedance, Kling,
Nano Banana Pro inside the timeline" feature (`README.md`). Mostly REST
API client logic against third-party generation providers plus local
preprocessing — one of the cheaper phases relative to its LOC count,
similar in character to Phase 1's finding that API-client code ports
almost mechanically.

## Source inventory

`Sources/PalmierPro/Generation/` (6,880 LOC, 36 files):

| Area | LOC | Files | Rust destination | Character |
|---|---|---|---|---|
| `GenerationService.swift` | 742 | 1 | `clawvinci-gen::service` | Top-level orchestrator — job lifecycle (submit, poll/subscribe, result, insert into timeline). Read first when this phase starts. |
| `GenerationBackend.swift` | (small, check on read) | 1 | `clawvinci-gen::backend` | Backend API surface — likely the Convex/Palmier backend client for generation jobs specifically, pairs with Phase 12's auth/backend client. |
| `Catalog/` | 1,163 | 6 | `clawvinci-gen::catalog` | `ModelCatalog.swift` (327), `VideoModelConfig.swift` (248), `CostEstimator.swift` (217), `AudioModelConfig.swift` (191), `ImageModelConfig.swift`, `UpscaleModelConfig.swift`, `ModelPreferences.swift` — declarative provider/model metadata (which models exist, their params, cost). Pure data + logic, ports directly, no platform dependency expected — confirm on read. |
| `Submission/` | ~700 (est. from partial scan) | 5 | `clawvinci-gen::submission` | `VideoGenerationSubmission.swift` (429), `AudioGenerationSubmission.swift` (162), `ImageGenerationSubmission.swift`, `MusicGenerationSubmission.swift`, `GenerationInput+AudioSource.swift` — builds provider-specific request payloads from `Models/GenerationInput` (Phase 1). REST client logic. |
| `Edit/` | ~700 (est.) | 8 | `clawvinci-gen::edit` | `EditAction.swift` (220), `AIEditMenu.swift`, `EditSubmitter.swift` + `+AudioTransform`/`+Seeds`/`+Upscale` extensions, `AudioTransformEditKind.swift`, `VideoToAudioEditKind.swift` — AI-driven edit operations (upscale, audio transform, etc.) submitted the same way as generation requests. |
| `Preprocessing/` | ~400 (est.) | 5 | `clawvinci-gen::preprocessing` | `VideoPreprocessor.swift`, `VideoTrimExtractor.swift`, `AudioTrackExtractor.swift`, `ImageConverter.swift`, `TrimmedSource.swift` — prepares source media for submission to a generation provider (trim, format-convert, extract). **Depends on Phase 2** (`clawvinci-media`) directly — this is where `clawvinci-gen` calls into the media engine, not a separate decode path. |
| `UI/` | 2,407 | 8 | Phase 6 (UI) | `GenerationView.swift` + extensions, `DropZoneView.swift`, `ReferenceControls.swift` — not this phase. |

## Design notes

- **No local model inference in this phase** — every provider (Seedance,
  Kling, Nano Banana Pro, and whatever else `ModelCatalog` lists) is a
  remote API. This phase is REST clients + request/response types +
  polling/webhook-style job tracking, not an ML-runtime phase like 9/10.
- **Preprocessing depends on Phase 2, not the other way around** — keep
  that dependency direction explicit in the crate graph
  (`clawvinci-gen` depends on `clawvinci-media`, never the reverse),
  matching the dependency discipline set in `PLAN/0-0-foundation.md`.
- **Results land back on the timeline as clips** via
  `Models/GenerationInput`/`MediaSource` (Phase 1) — confirm this phase's
  output contract matches what `clawvinci-timeline` (Phase 3) expects for
  inserting a generated asset, since generation results are effectively a
  specialized media-import path.
- **Cost estimation and model preferences** (`CostEstimator.swift`,
  `ModelPreferences.swift`) are pure logic/config — no reason these can't
  be ported early and tested standalone, ahead of the REST submission
  code, since they have no external dependency.

## Definition of done for Phase 11

- `clawvinci-gen` crate compiles against `clawvinci-media` (Phase 2) and
  `clawvinci-model` (Phase 1).
- Model catalog ported: given the same provider/model list as the macOS
  app, produces matching cost estimates and param sets.
- At least one real end-to-end generation round-trip against a live
  provider API (pick the simplest supported provider/modality first —
  likely image generation over video, given lower latency and simpler
  response handling) — submit, poll/subscribe, receive result, insert as
  a timeline clip via Phase 3.
- Preprocessing (trim/convert/extract) verified against Phase 2's decode/
  encode primitives with real media files, not just type-level plumbing.
- Job failure and cancellation surfaced correctly to the caller (UI or
  MCP `generate` tool from Phase 8) — no silent failures, matching the
  project's "fail loud" rule.
