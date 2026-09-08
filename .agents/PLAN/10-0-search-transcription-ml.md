# Phase 10.0 — Search, transcription, visual ML (`clawvinci-search`)

Semantic footage search (visual + text embedding), transcript search, and
the transcription pipeline. Lower porting cost than the architecture scan
initially suggested — **transcription itself is a cloud API client, not
on-device ML** (confirmed by reading `Transcription/TranscriptionBackend.swift`:
it submits jobs to a Convex backend action and subscribes to results, no
local Whisper-class model involved). The actual on-device ML in this area
is narrower: just the SigLIP2 visual embedder.

## Source inventory

`Sources/PalmierPro/Search/` (1,165 LOC, 10 files) +
`Sources/PalmierPro/Transcription/` (757 LOC, 6 files):

| macOS file | LOC | Rust destination | Notes |
|---|---|---|---|
| `Transcription/TranscriptionBackend.swift` | 120 | `clawvinci-search::transcription::backend` | Cloud API client (Convex `action`/`subscribe` calls) — depends on Phase 12's Convex-equivalent client existing first, or a minimal standalone HTTP client if that's not ready yet. Not ML. |
| `Transcription/Transcription.swift` | 350 | `clawvinci-search::transcription` | Orchestration/state around a transcription job — check its `Speech`/`SpeechVAD` import flagged in the earlier architecture scan; likely an on-device fallback path gated similarly to Phase 9's speech features. Resolve alongside Phase 9's VAD/speech decision, don't treat as a separate question. |
| `Transcription/TranscriptCache.swift` | 152 | `clawvinci-search::transcription::cache` | Local caching of transcript results — portable. |
| `Transcription/CloudTranscription.swift` | 63 | `clawvinci-search::transcription::backend` | Merge with `TranscriptionBackend`. |
| `Transcription/TranscriptSearch.swift` | 37 | `clawvinci-search::transcript_search` | Text search over transcript content — portable string logic. |
| `Transcription/WordCutPlanner.swift` | 35 | `clawvinci-search::transcription::word_cut` | Plans word-level cuts (feeds `remove_words`/`remove_silence` MCP tools, Phase 8) — portable. |
| `Search/SearchIndexCoordinator.swift` | 329 | `clawvinci-search::index` | Orchestrates building/updating the visual+text search index across the media library. |
| `Search/Models/VisualEmbedder.swift` | 88 | `clawvinci-search::embed::visual` | **On-device ML** — CoreML SigLIP2 image+text encoder. See below. |
| `Search/Models/VisualModelLoader.swift` | 121 | `clawvinci-search::embed::loader` | Model download/load state machine (`notInstalled` → `downloading` → `preparing` → `ready` → `failed`) — port the state machine as-is, swap the CoreML load call for an ONNX Runtime session load. |
| `Search/Models/ModelDownloader.swift` | 173 | `clawvinci-search::embed::download` | Downloads the model at runtime (not bundled) — straightforward HTTP download logic, portable almost as-is. |
| `Search/Models/TextTokenizer.swift` | 24 | `clawvinci-search::embed::tokenizer` | SigLIP2's text tokenizer — check whether this wraps `swift-transformers`' `Tokenizers` package (in `Package.swift` dependencies) or is a from-scratch implementation; if the former, the Rust side needs an equivalent tokenizer (e.g. `tokenizers` crate from Hugging Face, which the model's actual tokenizer config — exported alongside the model per `models/siglip2/export_tokenizer.py` — should load directly into). |
| `Search/Indexing/VisualIndexer.swift`, `FrameSampler.swift`, `EmbeddingStore.swift` | 87 / 117 / 121 | `clawvinci-search::index::*` | Frame sampling from video (depends on Phase 2 decode) + embedding storage — portable orchestration logic once the embedder itself exists. |
| `Search/Query/VisualSearch.swift` | 59 | `clawvinci-search::query` | Query-time embedding + nearest-neighbor search against `EmbeddingStore`. |
| `Search/SearchIndexConfig.swift` | 46 | `clawvinci-search::config` | Portable config/flags. |

## Visual embedding model — same conversion-pipeline reuse as Phase 9's beat model

`models/siglip2/` documents the exact same pattern as `beat_this`: a
Hugging Face checkpoint (`google/siglip2-base-patch16-256`), converted via
`convert.py` to Core ML (with an 8-bit palettize quantization option, and
a separate `export_tokenizer.py`). **This is the highest-leverage single
task in this phase**: SigLIP2 is a standard Hugging Face Transformers
model, and Hugging Face's own `optimum` tooling (or plain
`torch.onnx.export`) exports it to ONNX directly from the same checkpoint
— likely *less* conversion work than the existing CoreML path (which
needed custom einops-rewriting for `beat_this` but SigLIP2 is architecturally
more standard). Run via `onnxruntime` + DirectML on Windows, same as
Phase 9's beat model — share the ONNX Runtime integration work between
the two phases rather than building it twice.

## Revision (post-shipment): transcription backend needs rework

**Phase 10 is implemented and CI-green, but its transcription backend
(`clawvinci-search::transcription::backend.rs`) hardcodes
`https://api.palmier.io`** — a Palmier-operated endpoint Clawvinci has no
right to call. This predates `PLAN.md` Decision 10 (no accounts/backend/
telemetry — everything BYOK or local). The fix (BYOK config + a new local
on-device Whisper-class path) is scoped into
`PLAN/12-0-auth-backend-telemetry-updater.md` section 1, not redone here —
this note exists so anyone reading this phase doc in isolation knows it's
not the current state of that one file.

## Definition of done for Phase 10

- `clawvinci-search` crate compiles against `clawvinci-media` (Phase 2)
  and whatever Phase 12 client exists for the cloud transcription backend
  (or a minimal standalone one if Phase 12 hasn't landed yet — don't block
  this phase on Phase 12's full scope).
- SigLIP2 ONNX model runs on Windows via ONNX Runtime/DirectML, producing
  image and text embeddings; cosine-similarity search over a small known
  media set returns results matching the macOS app's CoreML-based search
  for the same query (spot-check parity, not bit-exact — embedding models
  can have minor numeric drift across runtimes and that's expected).
- Model download/load state machine reproduces the five states
  (`notInstalled`/`downloading`/`preparing`/`ready`/`failed`) with correct
  transitions and error surfacing.
- Transcription: cloud job submission/subscription/result-fetch works
  against the real backend (depends on Phase 12 auth being available to
  authenticate the request — sequence this phase's transcription work
  after Phase 12 starts, even though the crate itself can be scaffolded
  earlier).
- Transcript search and word-cut planning ported and tested against known
  transcript fixtures.
