# Phase 2.0 — Media engine (`clawvinci-media`)

FFmpeg-backed decode/encode/probe layer. Nothing here builds a timeline or
composites anything — it's the substrate Phase 3 (editing) and Phase 4
(playback) both sit on: given a media file, produce frames, samples, and
metadata; given a render graph, produce an encoded file.

## What it replaces

AVFoundation, used across `Preview/VideoEngine.swift` (831 LOC),
`Preview/CompositionBuilder.swift` (992 LOC), `Export/ExportService.swift`
(624 LOC), `Models/MediaResolver.swift`, `Audio/AudioTrackReader.swift`,
`Audio/WaveformExtractor.swift`. AVFoundation gives the macOS app four
things `clawvinci-media` must reproduce:

1. **Probing** — container/track/codec/duration/frame-rate/color-space/
   rotation/audio-layout metadata, loaded asynchronously (root `AGENTS.md`:
   "Use AVFoundation asynchronous property loading... do not access
   deprecated synchronous properties that may block").
2. **Decode** — frame-accurate seek and sequential read, for both preview
   scrubbing and export.
3. **Encode** — H.264/HEVC/ProRes-class output with hardware acceleration.
4. **Composition primitives** — `AVMutableComposition` /
   `AVVideoComposition` / `AVAudioMix` let AVFoundation stitch multiple
   source clips into one timeline and hand frames to a custom compositor
   (`Compositing/CustomVideoCompositor.swift`) shared by preview and
   export. **FFmpeg has no equivalent composition object** — Phase 3/4 own
   building this ourselves on top of what Phase 2 provides (see
   "Boundary with Phase 3/4" below).

## Architecture

- Rust bindings: use an FFI crate (`ffmpeg-next` or equivalent maintained
  binding) over hand-rolled bindgen — don't reinvent FFmpeg bindings.
  Confirm the crate's maintenance status and FFmpeg version support before
  committing; if it's stale, hand-rolled minimal bindgen for just the
  needed surface (avformat/avcodec/swscale/swresample) is the fallback,
  not a blocker.
- Hardware accel: D3D11VA/NVDEC for decode, NVENC/QSV/AMF for encode where
  available, software fallback otherwise. Matches root `AGENTS.md`'s
  "reuse readers, render contexts... do not rebuild during continuous
  interaction" — decoder instances must be long-lived and reused across
  seeks, not recreated per frame.
- Async surface: every decode/encode/probe call returns a `Future`
  (Tokio), with cancellation propagated through the FFmpeg call loop by
  checking a `CancellationToken` between packet/frame steps — direct
  translation of root `AGENTS.md`'s "propagate cancellation... check
  between chunks when an underlying synchronous API cannot be cancelled,"
  since FFmpeg's C API is inherently synchronous per-call.
- Time representation: FFmpeg's `AVRational` timebase + `int64_t` PTS maps
  directly to the frame-domain integer time the `clawvinci-model` types
  already use (Phase 1) — no floating-point time in the decode/seek path.
- Pixel format: normalize decoded frames to a consistent internal format
  (e.g. planar YUV or RGBA depending on what Phase 5's wgpu pipeline wants
  as input) at the decode boundary, not scattered across callers.

## Boundary with Phase 3/4 (composition)

AVFoundation's composition objects don't have a drop-in FFmpeg
replacement, so this boundary needs a real design decision, made in
Phase 4 (`PLAN/4-0-*.md`, written when that phase starts) but flagged
here: `clawvinci-media` should expose primitive decode/seek operations
(open source, seek to frame, read N frames, read audio samples in a
range) and stay ignorant of "timeline" as a concept. The equivalent of
`CompositionBuilder` — resolving a `Timeline`'s tracks/clips/transforms
into a per-output-frame render plan (which source, which source time,
what transform/blend) — is Phase 3/4's job, built on top of this crate,
not inside it. Keep `clawvinci-media` a thin, timeline-unaware media I/O
layer; this is what makes it independently testable per AGENTS.md's
"grow in layers."

## Scope

- Probing: container/track/codec/duration/fps/rotation/color/audio-layout,
  async, matching `MediaResolver`'s role.
- Thumbnail generation: single-frame decode at arbitrary timeline position,
  bounded/cached per root `AGENTS.md` performance rules ("load and hydrate
  media lazily... do not decode thumbnails... until a consumer needs
  them").
- Waveform extraction: replaces `Audio/WaveformExtractor.swift` (62 LOC) —
  decode audio, downsample to a peak/RMS envelope for UI display.
- Decode: frame-accurate seek + sequential read for playback and export.
- Encode: H.264/HEVC output with configurable bitrate/quality, hardware
  accel where present. ProRes and HDR encode are more involved — de-risk
  ProRes-class output specifically in this phase's prototype since
  `Export/HDRVideoExporter.swift` (291 LOC macOS-side) depends on it.
- Bounded concurrency: cap simultaneous decoders/readers per root
  `AGENTS.md` ("bound concurrent decoders, readers, exports... waveform
  extraction").

## Definition of done for Phase 2

- `clawvinci-media` crate compiles and links FFmpeg on Windows (both via
  CI and confirmed on a real Windows 10 22H2 machine — dynamic-linking
  FFmpeg DLLs alongside the Tauri bundle needs to actually work, not just
  compile).
- Prototype: given a video file path, produce (a) probed metadata, (b) a
  JPEG/PNG thumbnail at an arbitrary frame, (c) a decoded frame sequence
  written back out as a re-encoded file — proves decode→(trivial
  passthrough)→encode round-trips correctly before any real compositing
  exists.
- Waveform extraction produces a peak envelope from an audio file.
- Cancellation test: start a long decode/encode, cancel mid-flight,
  confirm it actually stops promptly and cleans up (temp files, decoder
  handles) rather than completing anyway.
- No panics on malformed/corrupt media input — returns `Result::Err`
  (root `AGENTS.md`: "test with missing audio or video tracks, unusual
  containers, zero or indefinite duration, rotated media, alpha media,
  nonstandard sample rates").
