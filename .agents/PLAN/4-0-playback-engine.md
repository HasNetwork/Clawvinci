# Phase 4.0 — Preview/playback engine

Turns a `Timeline` (Phase 1) plus `clawvinci-media` decode primitives
(Phase 2) into live, scrubbable playback with synced audio. This is the
single highest-risk phase in the whole port: it's where "decode → GPU
composite → present in a window, in sync with audio, at interactive
frame rates" has to actually work, and nothing in Rust/wgpu/Tauri gives it
to you for free the way AVFoundation does on macOS.

## Source inventory

`Sources/PalmierPro/Preview/` (6,034 LOC, 21 files) +
`Compositing/CustomVideoCompositor.swift` (86 LOC, but load-bearing):

| macOS file | LOC | Role |
|---|---|---|
| `Preview/CompositionBuilder.swift` | 992 | Builds `AVMutableComposition`/`AVVideoComposition`/`AVAudioMix` from a `Timeline`: resolves tracks/clips into `TrackMapping`s (timeline track, nested sequence, or black-background gap), computes natural sizes and transforms per clip. **This is the render-graph builder** — the piece with no FFmpeg equivalent, flagged in Phase 2. |
| `Preview/VideoEngine.swift` | 831 | `@MainActor` wrapper around `AVPlayer` driving playback from the built composition: seek modes (`exact`, `interactiveScrub`, `audibleStepForward/Backward`), rebuild-on-edit generation counting, time observation, source-clip preview mode. |
| `Preview/PreviewContainerView.swift` | 1,002 | SwiftUI view hosting the player — Phase 6 territory, but read it to understand what state the engine must expose (playhead, zoom, overlays). |
| `Preview/ScrubAudioEngine.swift` / `ScrubAudioOutput.swift` | 581 / 269 | Synthesizes audible scrub audio (the "zipper" sound when dragging the playhead) — a real-time audio graph separate from normal playback audio. |
| `Compositing/CustomVideoCompositor.swift` | 86 | Implements `AVVideoCompositing`: per-frame Core Image render, **shared by both preview (`AVPlayerItem`) and export (`AVAssetExportSession`)** — this is the key architectural fact to preserve. One compositor, two consumers. |
| `Preview/TransformOverlayView.swift`, `CropOverlayView.swift`, `CanvasViewingOverlay.swift`, `SlipTwoUpView.swift`, `ChromaKeySamplerOverlayView.swift` | ~1,000 combined | Interactive on-canvas editing overlays — Phase 6 (UI), but they read/write transform state this engine owns. |
| `Preview/LottieVideoGenerator.swift`, `ImageVideoGenerator.swift` | 296 / 227 | Synthesize video from a Lottie animation or a still image (for text/graphics clips that aren't real video sources) — a Phase 4/5 rendering concern, not Phase 2 decode. |
| `Preview/FrameCaptureRenderer.swift` | 185 | "Capture frame" tool support (single-frame still export from the timeline) — thin, reuses the main render path. |
| `Preview/AlphaVideoNormalizer.swift` | 158 | Alpha-channel handling for overlay/PIP sources. |
| `Preview/NestFlattener.swift` | 74 | Flattens nested-sequence clips into the render graph — pairs with Phase 3's nesting ops. |

## The core architecture decision: one render path, two consumers

Preserve the macOS app's most important design property: **preview and
export must render identically**, because `CustomVideoCompositor` is
literally the same object AVFoundation calls for both. If Phase 4 and
Phase 7 (export) end up with two different Rust code paths that composite
a frame, they *will* drift — colors, transforms, or effect ordering
diverging between what the user sees while editing and what gets
exported. Design `clawvinci-render`'s core as a single function:

```
fn composite_frame(plan: &FramePlan, sources: &SourceFrames) -> Frame
```

where `FramePlan` is what `CompositionBuilder`'s Rust equivalent produces
(per-output-frame: which source(s), source time, transform, blend, active
effects) and `composite_frame` is the wgpu render pass Phase 5's effect
kernels plug into. The live player calls this once per displayed frame at
interactive rates; the exporter (Phase 7) calls it once per output frame
in a batch loop. Same function, same shader code, two callers — not two
implementations. This is worth stating explicitly here because it's easy
to accidentally violate once export and preview are built in different
sprints.

## Real-time playback design (no AVPlayer equivalent)

There is no off-the-shelf "give it a composition, get synced A/V
playback" component in the Rust/Tauri/wgpu stack — this has to be built:

- **Render loop**: decode-ahead a small window of frames per active
  source (bounded, per Phase 2's concurrency caps), composite via wgpu,
  present to a surface. Tauri's webview can host a `<canvas>` with a wgpu
  surface (via `wgpu`'s native window integration through Tauri's raw
  window handle) — confirm this integration path concretely in this
  phase's prototype before designing further around it; it's the
  riskiest single technical assumption in the whole project.
- **A/V sync**: audio clock drives the master clock (matches how AVPlayer
  behaves — video frames are dropped/held to match audio, not the
  reverse), decoded audio feeds directly to a low-latency output (e.g.
  `cpal`), video presentation timestamps are compared against the audio
  clock each frame.
- **Seek modes**: reproduce `PreviewSeekMode`'s four cases — exact seek
  (precise frame, used for programmatic/agent seeks), interactive scrub
  (throttled, prioritizes responsiveness over precision during a drag),
  audible step forward/backward (frame-step with the scrub-audio
  "zipper" — Phase 4 must also design a `clawvinci-audio`-adjacent scrub
  audio synthesizer equivalent to `ScrubAudioEngine`, or scope it out
  explicitly as a v-next nicety rather than silently dropping it — flag
  the choice, don't just skip it).
- **Cancellation/rebuild generations**: `VideoEngine` tracks a
  `rebuildGeneration` counter so an in-flight composition rebuild started
  before an edit doesn't clobber state after a newer edit lands — port
  this exact pattern (a monotonic generation counter checked before
  committing async work), it's a real race the macOS code had to solve.

## Definition of done for Phase 4

- Prototype (should land *before* the rest of this phase, and before
  Phase 3 finalizes its data shapes): open one video file, decode it,
  composite it through a trivial (no-op) `composite_frame`, present it in
  a Tauri window, play it back with synced audio, scrub it. This proves
  the wgpu-in-Tauri-window path and the A/V sync design before either is
  load-bearing for the rest of the app.
- `FramePlan` builder: given a `Timeline` with multiple tracks/clips
  (including a nested sequence and a gap), produces the correct
  per-output-frame plan — unit-testable without any real GPU/decode work,
  matching how `CompositionBuilder` is logic-testable independent of
  AVFoundation actually running.
- Real-time playback of a multi-track, multi-clip timeline with audio in
  sync, interactive scrub, and frame-step — measured, not just "feels
  fine" (root `AGENTS.md`: "measure before claiming a performance
  improvement... use a focused benchmark").
- Generation-counter correctness: an edit made mid-rebuild doesn't
  produce a stale frame — test it directly, it's a real race.
