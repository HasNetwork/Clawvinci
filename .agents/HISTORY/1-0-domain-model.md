# Phase 1.0 — Domain Model & Project File Format Implementation Log

- **Date**: 2026-09-04
- **Author**: Antigravity AI Agent
- **Target**: `windows/src-tauri/crates/clawvinci-model`

## Summary of Changes

Translated all 21 models from `Sources/PalmierPro/Models/` to Rust in `clawvinci-model`:

1. **Project File & Legacy Fallback (`project_file.rs`)**:
   - Implemented `ProjectFile` root matching `project.json`.
   - `ProjectFile::decode`: Two-step fallback decoding. Tries modern `ProjectFile` first; if it encounters a legacy bare `Timeline`, it wraps it into a single-timeline project with active and open timeline IDs set.
   - Enforces non-empty timelines per macOS specification; returns strongly typed `ModelError` on malformed inputs.

2. **Timeline Core & Clip Placement (`timeline.rs`)**:
   - `Timeline`: Canvas width/height, FPS, tracks, timeline markers, frame calculations (`total_frames`, `display_frames`).
   - `Track`: Media type (`ClipType`), muted/hidden/sync-locked states, display height, and clips collection.
   - `Clip`: Comprehensive clip metadata including media reference, playback speed, volume, fade ramps with linear and smooth interpolation, spatial `Transform`, source `Crop`, text attributes, keyframe tracks, effect stacks, and composite `BlendMode`.
   - `Transform`: Normalized canvas space placement with custom deserializer supporting legacy `x` and `y` keys (`centerX = x + width - 0.5`, `centerY = y + height - 0.5`). Implemented canvas boundary snapping.
   - `Crop`: Edge insets with `KeyframeInterpolatable` conformance.

3. **Keyframes & Interpolation (`keyframe.rs`)**:
   - `Keyframe<V>`: Generic keyframe supporting `Linear`, `Hold`, and `Smooth` interpolation.
   - `KeyframeTrack<V>`: Keyframe track with binary search lookup, sampling, keyframe movement, removal, and range queries.
   - `KeyframeInterpolatable` trait with implementations for `f64`, `AnimPair`, and `Crop`.
   - `AnimatableProperty`: Inspector and timeline lane property definitions.

4. **Typography & Styling (`text_style.rs`, `text_animation.rs`, `text_fill_mode.rs`, `text_layout.rs`)**:
   - `TextStyle`: Typography attributes matching macOS `TextStyle`, font scaling, tracking, line spacing, alignment, bold/italic traits, and font case transforms.
   - `Rgba`: Color container with hex string parsing and formatting.
   - `TextShadow`, `TextOutline`, `TextBackground`: Style containers with default constants.
   - `TextAnimation`: Entrance presets (`popIn`, `slideUp`, `typewriter`) and per-word presets (`wordReveal`, `wordSlide`, `highlightPop`, `highlightBlock`).
   - `WordTiming`: Word frame ranges for animated captions.
   - `TextFillMode`: `Color`, `Footage`, `Inverted`.
   - `TextLayout`: Natural bounding size calculations based on glyph dimensions, padding, borders, and shadows.

5. **Color Grading & Curves (`grade.rs`)**:
   - `GradeCurve`: Master and R/G/B piecewise-linear tone curves with `eval`.
   - `HueCurves`: Cyclic hue curve adjustments (hue-vs-hue, hue-vs-sat, hue-vs-lum) with wrapping interpolation.

6. **Effects & Layouts (`effect.rs`, `layout.rs`, `matte.rs`)**:
   - `Effect` & `EffectParam`: Clip effect stacks with static values, strings, and animated keyframe tracks.
   - `VideoLayout`: 13 video layouts (`full`, `side_by_side`, `top_bottom`, `pip_*`, `grid_*`, etc.) computing normalized layout slots.
   - `Matte` & `MatteAspect`: Aspect ratios with even-dimension constraints.

7. **Media Package Manifest & Resolution (`media_manifest.rs`, `media_resolver.rs`)**:
   - `MediaManifest` & `MediaFolder`: Package `media.json` format.
   - `MediaSource`: Tagged enum matching Swift `Codable` serialization (`{"project":{"relativePath":"..."}}` and `{"external":{"absolutePath":"..."}}`).
   - `GenerationInput`: AI provider prompt metadata and upscale parameters.
   - `MediaResolver`: Resolves asset IDs to file paths and detects missing offline assets.

8. **Multicam & Speakers (`multicam.rs`, `speaker.rs`)**:
   - `MulticamSource`: Multicam angles and audio stems with sync offsets.
   - `SpeakerRegistryEntry`: Speaker identity, color, and voice centroid.

9. **Comprehensive Test Suites (`tests/`)**:
   - `roundtrip_tests.rs`: End-to-end serialization and deserialization of complete project files, text styles, and curves.
   - `legacy_compat_tests.rs`: Bare `Timeline` legacy fallback decoding, legacy `Transform` `x`/`y` keys, and error handling.
   - `media_manifest_tests.rs`: Exact `MediaSource` JSON representation and `MediaResolver` path resolution.
   - `keyframe_interpolation_tests.rs`: Linear, Hold, and Smooth interpolation curves across keyframes and multi-value pairs.

## Verification

- Scope: All changes isolated to `windows/src-tauri/crates/clawvinci-model/` and workspace manifest.
- CI: Pushed to `HasNetwork/Clawvinci` on branch `worktree-plan-windows-port` for remote Windows verification.
