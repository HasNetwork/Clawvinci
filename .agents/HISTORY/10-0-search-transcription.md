# Phase 10.0 — Search & Transcription Engine (`clawvinci-search`)

**Status**: ✅ Complete  
**CI Verification**: ✅ 100% Green on GitHub Actions Run `34195654610` (Check, Clippy, Test, Tauri Build & Artifact Upload)  
**Clean Commit**: `c2b1955`  
**Date**: 2026-09-08  

## What was built

Phase 10.0 ports semantic footage search, SigLIP2 visual embeddings, transcript search, and word-level cut planning from `Sources/PalmierPro/Search/` (10 files, 1,165 LOC) and `Sources/PalmierPro/Transcription/` (6 files, 757 LOC) into the native Rust crate `clawvinci-search`:

1. **Exact-Match & Multi-Term Transcript Search (`src/transcription/search.rs`):**
   - Direct port of `TranscriptSearch.swift`.
   - `TranscriptSearch::terms`: Tokenizes query strings, strips edge punctuation (e.g. `"budget,"` -> `"budget"`), and filters whitespace.
   - `TranscriptSearch::matches`: Case-insensitive substring matching across all query terms.
   - `TranscriptSearch::search`: Scans cached transcript segments across media assets, respecting limit bounds.

2. **Word Cut Planner & Gap Preservation (`src/transcription/word_cut.rs`):**
   - Direct port of `WordCutPlanner.swift`.
   - `CutAggressiveness`: Tight (60ms), Balanced (150ms), Loose (320ms) gap preservation presets.
   - `WordCutPlanner::cut_ranges`: Groups consecutive selected words into runs, applies `keep_gap_frames / 2` margin to adjacent unselected words, bounds cuts within `[clip_start, clip_end]`, and coalesces overlapping cuts.

3. **Disk & In-Memory Transcript Cache (`src/transcription/cache.rs`):**
   - Direct port of `TranscriptCache.swift`.
   - `TranscriptCache`: Bounded in-memory hash map with automatic disk persistence to JSON (`~/.cache/clawvinci/transcripts` or custom dir).
   - Key derivation: 32-character SHA256 hex digest of file path, modification timestamp, and file byte size (`"{path}|{mtime}|{size}"`).
   - Supports range filtering (`filter_range`) and source-time offsetting (`offsetting`).

4. **Cloud Transcription Client (`src/transcription/backend.rs`):**
   - Direct port of `TranscriptionBackend.swift` and `CloudTranscription.swift`.
   - Convex cloud action submission (`transcriptions:submit`), status polling (`transcriptions:byId`), and result payload downloading.

5. **Binary `EmbeddingStore` Format with Byte Parity (`src/visual/store.rs`):**
   - Direct port of `EmbeddingStore.swift`.
   - Magic constant: `b"PALMEMB1"` (8 bytes).
   - Followed by 4-byte LE length, UTF-8 JSON header (`model`, `modelVersion`, `samplerVersion`, `dim`, `count`).
   - Binary row payload: 24 bytes per row (`time: f64`, `shotStart: f64`, `shotEnd: f64`) followed by `dim * 2` bytes of little-endian IEEE 754 half-precision floats (`half::f16`).
   - `EmbeddingStore::save`, `load`, `read_header`, and `is_current` verified for byte-accurate roundtripping.

6. **Luma Grid & Scene Cut Frame Sampler (`src/visual/sampler.rs`):**
   - Direct port of `LumaGrid.swift` and `FrameSampler.swift`.
   - `LumaGrid`: 8x8 cell downsample, ITU-R BT.601 coefficients (`0.299*R + 0.587*G + 0.114*B`).
   - `FrameSamplerState`: Candidate time generator, scene-cut thresholding (`mean_diff > 12.0`), and coverage floor (8.0s) for long static shots.

7. **Visual Vector Search & Deduplication (`src/visual/search.rs`):**
   - Direct port of `VisualSearch.swift`.
   - Matrix dot-product cosine similarity over unit-normalized feature vectors.
   - Best-frame-per-shot filtering (`best_per_shot[shot_start]`) preventing single scenes from flooding search results.
   - Minimum score thresholding (floor 0.05) and relative cutoff (`top_score * 0.85`).

8. **Model Spec & Deterministic Embedder (`src/visual/model.rs` & `loader.rs`):**
   - Direct port of `VisualEmbedder.swift`, `ModelDownloader.swift`, and `VisualModelLoader.swift`.
   - `ModelSpec`: `siglip2-base-patch16-256`, 768 dimensions, 256px image size, 64 context length.
   - `MockVisualEmbedder`: Fast, deterministic L2-normalized feature generator for offline test execution.
   - `VisualModelLoader`: 6-state loader machine (`Unknown`, `NotInstalled`, `Downloading`, `Preparing`, `Ready`, `Failed`).

9. **Unified Search Index Coordinator (`src/coordinator.rs`):**
   - Direct port of `SearchIndexCoordinator.swift`.
   - Preflight checks for visual and transcript indexing needs.
   - Coordinates visual moment search and spoken dialogue queries.

10. **MCP Agent Layer Integration (`clawvinci-mcp`):**
    - `search_media`: Wired with `scope` (`"visual"`, `"spoken"`, `"both"`), returning `{ timelineFps, moments, spoken, index }`.
    - `get_transcript`: Wired with `granularity` (`"words"`, `"segments"`), project frame timing, and clip tracking.
    - `remove_words`: Uses `WordCutPlanner` with `CutAggressiveness` to compute exact frame cuts.

11. **Comprehensive Test Suite (`tests/search_tests.rs`):**
    - 7 integration tests covering transcript search, word cut planner, cache serialization, binary embedding store roundtrip, luma grid cut detection, visual search ranking, and index coordinator.
