# Phase 5.0 — GPU Effects Pipeline

**Status**: ✅ Complete  
**Date**: 2026-09-06  

## What was built

Phase 5.0 implements the complete effects pipeline and text rendering engine in `clawvinci-render`, porting all 12 Metal shader algorithms, the 21-effect `EffectRegistry` in canonical macOS order, 3D LUT loader and tetrahedral interpolation, and styled text rendering with animation presets:

1. **`EffectRegistry` (`clawvinci-render::effects`):**
   - Ported `EffectDescriptor`, `EffectParamSpec`, and `ResolvedEffectParams` from `EffectRegistry.swift`.
   - Static table of all 21 effects in exact canonical order (`CANONICAL_ORDER`), with `insert_index` priority matching macOS.
   - Transparent linear-light color space wrapping (`srgb_to_linear` / `linear_to_srgb`) for effects that require linear color (e.g. exposure).

2. **Ported Shader Algorithms (All 12 Metal / Core Image kernels):**
   - `levels`: Black/white-point remap from `Levels.metal` (`bp = -blacks * 0.4`, `wp = 1.0 - whites * 0.4`).
   - `highlights_shadows`: Tone-region-weighted cubic luminance adjustments from `HighlightsShadows.metal`.
   - `color_wheels`: Lift/gamma/gain primary color wheels with luma-neutral chroma offsets from `Wheels.metal` and `ColorWheels.swift`.
   - `exposure`: Linear-light EV scaling matching `CIExposureAdjust`.
   - `contrast`: Pivot around mid-gray (0.5) matching `CIColorControls`.
   - `saturation`: Rec.709 luma lerp matching `CIColorControls`.
   - `temperature`: Chromatic adaptation and green/magenta tint matching `CITemperatureAndTint`.
   - `vibrance`: Non-linear saturation boosting less-saturated colors matching `CIVibrance`.
   - `invert`: Color inversion matching `CIColorMatrix`.
   - `vignette`: Centered superellipse SDF with roundness morph, midpoint, and feather from `Vignette.metal`.
   - `grain`: Frame-seeded `hash13` monochromatic noise with mid-tone luma masking from `Grain.metal`.
   - `edge_rounding`: Corner radius and softness box SDF from `EdgeRounding.metal`.
   - `chroma_key`: HSV hue distance, saturation gating, and edge spill suppression from `ChromaKey.metal`.
   - `blur`: High-performance separable Gaussian blur, unsharp mask sharpening, bilateral noise reduction, and directional motion blur.
   - `clarity`: Multi-pass local-contrast unsharp against mid-radius Gaussian blur and dark-channel prior dehaze from `Clarity.metal`.
   - `glow`: Multi-pass bloom/halation isolating highlights with warmth tint, blurring, and screen-blending back from `Glow.metal`.
   - `grade_curves`: Per-channel R/G/B and master luma curves evaluated via 256-entry 1D LUTs from `GradeCurves.metal`.
   - `hue_curves`: Display-space HSV conversion and 256-entry hue LUT from `HueCurves.metal`.

3. **3D LUT Support (`clawvinci-render::effects::lut` and `lut_loader`):**
   - Full `.cube` parser supporting `LUT_3D_SIZE`, `DOMAIN_MIN`, and `DOMAIN_MAX` normalization.
   - Tetrahedral 3D interpolation matching `LUTTetra.metal`.
   - Thread-safe caching for loaded `.cube` assets.

4. **Text Rendering and Animation (`clawvinci-render::text`):**
   - Pure Rust text layout and rasterization using `fontdue` with automatic system font discovery and robust zero-dependency fallback.
   - Full support for `TextStyle` (font size, weight, tracking, alignment, background boxes, outlines, colors).
   - Pure evaluator `TextAnimator` supporting whole-clip entrance presets (`PopIn`, `SlideUp`) and word-level presets (`WordReveal`, `WordSlide`, `HighlightPop`, `HighlightBlock`).

5. **Compositor Integration (`clawvinci-render::compositor`):**
   - Layer pipeline matches `FrameRenderer.swift` exactly: `Source -> Crop -> Effects -> Edge Rounding -> Transform -> Blend -> Canvas`.
   - Works seamlessly across all visual source types (video, image, text, solid color, nested sequence) for both preview and export.

6. **Test Suite:**
   - 14 comprehensive unit and numerical correctness tests in `clawvinci-render/tests/effects_tests.rs`.
   - Tests verify exact mathematical formulas against Metal source definitions, canonical ordering, tetrahedral interpolation, text rasterization, and end-to-end compositing.
