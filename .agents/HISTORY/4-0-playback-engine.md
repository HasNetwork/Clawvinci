# Phase 4.0 — Playback Engine

**Status**: ✅ Complete  
**Commits**: `6687c4c` through `982b9ff`  
**CI**: Run [#34011709384](https://github.com/HasNetwork/Clawvinci/actions/runs/34011709384) — all green (check, clippy, test, build)

## What was built

### clawvinci-render crate

The core rendering pipeline, designed around the "one function, two callers" principle: preview and export share a single `composite_frame` function to guarantee pixel-identical output.

**FramePlan / LayerPlan / AudioPlan** (`plan.rs`)
- `CompositionBuilder` translates a `Timeline` snapshot into a `FramePlan` — a declarative render instruction set.
- Each `LayerPlan` captures source reference, transform, crop, opacity, blend mode, and natural dimensions.
- Nested timelines (`ClipType::Sequence`) resolved recursively via `build_frame_plan_dyn` using `&dyn Fn` trait objects (avoids infinite monomorphization).
- `AudioPlan` / `AudioClipPlan` captures per-frame audio mix state for downstream audio rendering.

**Compositor** (`compositor.rs`)
- `composite_frame` takes a `FramePlan` and per-layer pixel buffers, composites bottom-to-top using Porter-Duff alpha blending.
- Full SVG blend mode support: Normal, Multiply, Screen, Overlay, Darken, Lighten, ColorDodge, ColorBurn, HardLight, SoftLight, Difference, Exclusion.
- CPU software renderer — platform-independent, CI-testable. Future Phase 5 adds GPU (wgpu/WGSL) path on top of this same `FramePlan`.

**PlaybackEngine** (`playback.rs`)
- State machine: Stopped → Playing → Paused, with frame-accurate seek.
- `rebuild_generation` counter (`AtomicU64`) prevents stale async renders from overwriting current state after timeline edits.
- `AudioClock`-based timing for A/V sync during playback.
- `ScrubAudioEngine` for interactive frame-step audio feedback.

### Tauri integration

- Playback commands exposed: `playback_play`, `playback_pause`, `playback_stop`, `playback_seek`, `playback_get_frame`.
- `PlaybackEngine` registered as Tauri managed state.
- `playback_get_frame` returns base64-encoded RGBA for canvas rendering (capacity computed via `div_ceil`).

### Frontend

- `index.html` updated with preview canvas and transport controls (Play/Pause/Stop/Seek).

### Tests

- `playback_tests.rs`: 10 comprehensive tests covering frame plan construction, speed/trim, multi-track compositing, gap detection, nested sequence resolution, black background, Normal/Multiply blend modes, and audio plan generation.

## Issues resolved during CI

| Issue | Root cause | Fix |
|---|---|---|
| `manual_div_ceil` clippy warning | Manual `(a + b - 1) / b` instead of `usize::div_ceil` | Used `usize::div_ceil()` |
| Infinite monomorphization recursion limit | Generic `FSize`/`FNest` closures re-wrapped with `&` on each recursive call for nested timelines | Introduced `build_frame_plan_dyn` / `build_layer_plan_dyn` using `&dyn Fn` trait objects |
| `Track` struct field mismatch | Tests used `locked: false` but struct has `sync_locked` + `display_height` | Updated all 10 Track constructions in tests |
| Lifetime on `dyn Fn` return | `&dyn Fn(&str) -> Option<&Timeline>` inferred `'static` for returned reference | Added explicit `'a` lifetime parameter |

## Files added/modified

- `windows/src-tauri/crates/clawvinci-render/src/plan.rs` — FramePlan builder
- `windows/src-tauri/crates/clawvinci-render/src/compositor.rs` — composite_frame + blend modes
- `windows/src-tauri/crates/clawvinci-render/src/playback.rs` — PlaybackEngine + ScrubAudioEngine
- `windows/src-tauri/crates/clawvinci-render/src/lib.rs` — module exports
- `windows/src-tauri/crates/clawvinci-render/Cargo.toml` — serde_json dependency
- `windows/src-tauri/crates/clawvinci-render/tests/playback_tests.rs` — test suite
- `windows/src-tauri/src/lib.rs` — Tauri playback commands
- `windows/src/index.html` — preview canvas + transport controls
