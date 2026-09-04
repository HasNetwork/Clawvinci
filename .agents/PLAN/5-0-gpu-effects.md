# Phase 5.0 — GPU effects pipeline

Ports the 12 Metal/Core Image kernels, the effect registry, and text
rendering/animation. Builds directly on Phase 4's `composite_frame` — this
phase is what actually runs *inside* that render pass, not a separate
system.

## Source inventory

`Metal/*.metal` (12 shaders) + `Sources/PalmierPro/Compositing/` (2,588
LOC, 23 files) + `Plugins/MetalCIKernelPlugin`:

| Metal kernel | Rust/WGSL destination | Notes |
|---|---|---|
| `Levels.metal` | `clawvinci-render::effects::levels` | |
| `GradeCurves.metal` | `clawvinci-render::effects::grade_curve` | Pairs with `Models/GradeCurve.swift` (Phase 1) — curve control points evaluated per-pixel. |
| `HueCurves.metal` | `clawvinci-render::effects::hue_curve` | Pairs with `Models/HueCurves.swift`. |
| `Wheels.metal` | `clawvinci-render::effects::color_wheels` | Lift/gamma/gain wheels — pairs with `Compositing/ColorWheels.swift` (74 LOC, the UI-facing math, portable logic). |
| `HighlightsShadows.metal` | `clawvinci-render::effects::highlights_shadows` | |
| `Clarity.metal` | `clawvinci-render::effects::clarity` | Local contrast — likely needs a blur pass; check the kernel's actual algorithm before assuming a single-pass shader suffices. |
| `Glow.metal` | `clawvinci-render::effects::glow` | Likely multi-pass (bloom-style blur + composite) — same caveat as Clarity. |
| `Vignette.metal` | `clawvinci-render::effects::vignette` | |
| `Grain.metal` | `clawvinci-render::effects::grain` | Time-varying (per-frame noise) — note `ResolvedEffectParams.frame` exists specifically to drive this; preserve the frame-seeded randomness so grain doesn't flicker inconsistently. |
| `ChromaKey.metal` | `clawvinci-render::effects::chroma_key` | Pairs with the chroma-key sampler overlay (Phase 6 UI) and `EditorViewModel+ChromaKey.swift` (Phase 3). |
| `EdgeRounding.metal` | `clawvinci-render::effects::edge_rounding` | Corner radius on video rects. |
| `LUTTetra.metal` | `clawvinci-render::effects::lut` | 3D LUT via tetrahedral interpolation — pairs with `Compositing/LUTLoader.swift` (119 LOC, `.cube` file parsing, fully portable Rust logic). |

`Compositing/Kernels/*.swift` (12 files, ~350 LOC total) are the Swift-side
`CIKernel`/`CIColorKernel` wrappers around each `.metal` file — these don't
port directly (no Core Image on Windows); their role becomes each effect's
Rust-side parameter marshaling into a WGSL compute/fragment shader, i.e.
merge their responsibility into the `effects::*` modules above rather than
keeping a parallel "kernel wrapper" layer.

| Other Compositing file | LOC | Rust destination |
|---|---|---|
| `TextFrameRenderer.swift` | 751 | `clawvinci-render::text` — largest single file in this area; text layout/rendering onto the frame. |
| `FrameRenderer.swift` | 469 | `clawvinci-render::frame` — per-frame render orchestration; this is close to where Phase 4's `composite_frame` and this phase's effect chain actually meet. |
| `EffectRegistry.swift` | 392 | `clawvinci-render::effects::registry` — see design below. |
| `TextAnimator.swift` | 135 | `clawvinci-render::text::animator` |
| `LUTLoader.swift` | 119 | `clawvinci-render::effects::lut::loader` — portable file-parsing logic. |
| `ColorScopes.swift` | 108 | `clawvinci-render::scopes` — waveform/vectorscope UI-data generation (feeds Phase 6's scope display, not the render path itself). |
| `TextTiltGeometry.swift` | 88 | `clawvinci-render::text::geometry` |
| `ColorWheels.swift` | 74 | `clawvinci-render::effects::color_wheels` (math half; UI half is Phase 6). |
| `CompositorInstruction.swift` | 56 | `clawvinci-render::frame` — per-clip render instruction type, likely folds into `FramePlan` from Phase 4. |

## `EffectRegistry` design — port the shape, not the mechanism

`EffectDescriptor` is a clean declarative pattern: `id`, `displayName`,
`category`, a list of `EffectParamSpec` (key/label/range/default/unit),
and an `apply` closure taking `(CIImage, ResolvedEffectParams, CGRect) ->
CIImage`. This maps well to Rust:

```rust
struct EffectDescriptor {
    id: &'static str,
    display_name: &'static str,
    category: &'static str,
    params: &'static [EffectParamSpec],
    linearizes: bool,
    resource_key: Option<&'static str>,
    apply: fn(&mut RenderPass, &ResolvedEffectParams, Rect),
}
```

— a static registry table (not a runtime plugin system; there are exactly
12 effects, known at compile time, so a `const`/`static` array beats
runtime registration machinery per Rule 2, no speculative abstraction).
Each `apply` binds a WGSL shader (compiled from the corresponding `.metal`
kernel's algorithm, not the Metal source itself — Metal Shading Language
and WGSL are different languages; port the *math*, cross-check output
against the macOS app's rendered output for the same input, don't
transliterate syntax) and its uniform buffer from `ResolvedEffectParams`.

`linearizes: bool` (whether the effect expects linear light) is a real
color-management detail worth preserving exactly — get it wrong and grade
curves/levels will look subtly different from the macOS app on the same
project.

## Build-time shader compilation

`Plugins/MetalCIKernelPlugin` is a Swift Package Manager build-tool plugin
that compiles `.metal` → `.metallib` at build time. The WGSL equivalent:
either compile WGSL shaders at build time via a `build.rs` (validating
syntax, embedding via `include_str!`) or at runtime via wgpu's shader
module creation with a startup-time validation pass. Prefer `build.rs`
validation — catching a shader syntax error at `cargo build` time instead
of at first use matches the spirit of what the Metal plugin already does.

## Definition of done for Phase 5

- Every one of the 12 effects implemented as a WGSL shader behind the
  `EffectDescriptor` registry, each with a golden-image test: apply the
  effect to a known test frame at known param values, compare against a
  reference image captured from the running macOS app for the same
  input (not just "renders without crashing" — visual correctness is the
  actual bar here, and is the only way to catch a subtly-wrong port of
  the underlying math).
- LUT loading (`.cube` parsing) + tetrahedral interpolation, tested
  against a known LUT file and known input/output color pairs.
- Text rendering: at minimum, positioned/styled text matching
  `Models/TextStyle.swift` (Phase 1) composited onto a frame, with the
  animation system (`TextAnimator`) driving position/opacity/scale over
  time.
- Effect chain ordering matches the macOS app's `EffectRegistry` category
  ordering exactly — effect order changes visual output, so this isn't
  cosmetic.
- Confirmed working through both consumers described in Phase 4 (preview
  and export) via the shared `composite_frame` path — don't consider this
  phase done if it only demonstrated working in a preview-only test
  harness.
