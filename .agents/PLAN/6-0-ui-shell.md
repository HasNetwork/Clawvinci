# Phase 6.0 — UI shell (Tauri/web)

The frontend: timeline canvas, inspector, media panel, preview surface,
design-token system, and the Tauri IPC command surface that ties the
`windows/src` web UI to the `clawvinci-*` Rust crates. This phase has the
most files of any phase (SwiftUI/AppKit touch 229 of the 415 macOS
files) but the *least* logic to port faithfully — it's mostly new UI code
informed by the macOS app's behavior, not translated line-by-line the way
Phase 1's models were.

## Source inventory (what to study, not what to translate 1:1)

| Area | LOC | Files | What it teaches this phase |
|---|---|---|---|
| `Timeline/` | 8,078 | 26 | `TimelineView.swift` (2,070 LOC) + `TimelineInputController.swift` (1,569 LOC) — custom-drawn timeline with its own hit-testing, drag, snap (`SnapEngine.swift`), multicam (`MulticamEngine.swift`), keyframe lanes. `ClipRenderer.swift` (935 LOC) draws clips (waveforms, thumbnails, labels) directly rather than via retained-mode views — the Windows equivalent is an HTML5 `<canvas>` (or WebGL if profiling shows canvas 2D isn't fast enough at high zoom/many-clips) with the same custom hit-testing/drag/snap logic, in TypeScript. |
| `Inspector/` | 5,163 | 22 | Tabbed property editor (`Tabs/AdjustTab`, `AudioTab`, `TextTab`, `MulticamTab`, `AIEditTab`) driving `clawvinci-timeline` mutations — this is mostly conventional form UI, translate the *tab structure and field set*, not the SwiftUI code. |
| `MediaPanel/` | 6,400 | 29 | `MediaTab`, `AudioPanelTab`, `CaptionsTab`, `IndexTab` — asset grid, captions authoring (`CaptionBuilder.swift`, `CaptionSpecBuilder.swift` — portable logic, could live in `clawvinci-timeline` or a small `clawvinci-captions` module rather than being UI-only), transcript/marker browsing. |
| `Preview/PreviewContainerView.swift` + overlays | ~3,000 | 8 | Hosts the render surface (Phase 4's wgpu-in-canvas) plus interactive overlays: transform handles, crop, chroma-key sampler — these need real hit-testing against the frame plan, not just visual chrome. |
| `UI/` | 2,407 | 23 | `AppTheme.swift` — the design-token catalog (see below). Also `TimelineClipColors.swift`, `KeyframeControlStrip.swift`, `WorkspaceLayout.swift` (panel layout persistence). |
| `Toolbar/` | 222 | 1 | Top toolbar — straightforward. |

## Design tokens: `AppTheme` → CSS custom properties

`AppTheme.swift` is a real, disciplined design-token system: `Background`,
`Border`, `Text` color scales (each light/dark adaptive via
`NSColor(name:_:)` dynamic providers), plus `Spacing`, `FontSize`,
`FontWeight`, `Radius`, `BorderWidth`, `Opacity`, `IconSize`, `Shadow`,
`Anim` scales, enforced project-wide by root `AGENTS.md`'s Design System
rule ("All UI styling MUST use `AppTheme` constants... never hardcoded
numeric values"). Port the *policy*, not the Swift syntax:

- Every token category becomes a CSS custom property namespace (e.g.
  `--bg-base`, `--bg-surface`, `--space-xs`...`--space-xxl`,
  `--radius-xs`...`--radius-xl`), defined once, themed via light/dark media
  query or a `data-theme` attribute exactly the way `AppTheme.adaptive`
  branches on `NSAppearance` — extract the actual light/dark RGB values
  from `AppTheme.swift` when this phase starts (they're concrete numbers
  in the source, not something to re-derive from scratch).
- Carry the *rule* forward into `RULES.md` once this phase starts: no
  hardcoded pixel/color values in component code — same discipline, new
  mechanism (a lint step, e.g. stylelint custom-property enforcement or a
  simple grep-based CI check, replaces Swift's "AppTheme or nothing"
  convention since CSS doesn't enforce this at compile time the way a
  Swift enum namespace does).

## Tauri IPC command surface

Every mutation/query the UI needs against `clawvinci-timeline`,
`clawvinci-media`, `clawvinci-render`, etc. becomes a `#[tauri::command]`.
Design this surface deliberately, not as one command per Swift method:

- High-frequency state (playhead position, audio meters, render progress)
  should use Tauri's event system (push from Rust to frontend) rather than
  polling commands — matches root `AGENTS.md`'s "keep high-frequency
  observable state as narrow as possible... a progress counter, meter, or
  playhead update must not invalidate an entire panel."
- Mutation commands (add clip, split, set property, etc.) call directly
  into `clawvinci-timeline`'s ops (Phase 3) and return the resulting delta
  or updated `Timeline` slice — this is also the natural place to reuse
  Phase 3's `Command`-enum undo representation as the wire format, so the
  frontend and the MCP tool layer (Phase 8) describe edits the same way.
- Commands must not block the Tauri event loop — anything touching Phase 2
  media I/O or Phase 4 rendering runs on a background Tokio task, results
  delivered async, matching the "IPC boundary discipline" rule in
  `RULES.md`.

## Definition of done for Phase 6

- Design-token CSS file generated/hand-written from `AppTheme.swift`'s
  actual values, covering every category listed above, themed for both
  light and dark.
- Timeline canvas: renders tracks/clips from a `Timeline`, supports
  select/drag/trim/split via mouse input routed to Phase 3 mutation
  commands, snap-to-clip-edge behavior matching `SnapEngine`.
- Inspector: at least the Adjust tab (color/effects) and Audio tab wired
  to real `clawvinci-render`/`clawvinci-timeline` state — prove the
  IPC round-trip (UI edit → Rust mutation → undo entry → UI reflects new
  state) end-to-end before building out every remaining tab.
- Preview surface: Phase 4's wgpu output actually displayed in the Tauri
  window, scrubbable from the UI.
- Media panel: asset grid showing imported media with thumbnails (Phase 2
  thumbnail generation).
- No UI code reads or writes `clawvinci-*` state directly except through
  the Tauri command/event surface — verified by the crate boundary, not
  just convention.
