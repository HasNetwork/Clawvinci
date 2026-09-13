# Phase 15 · Item 3 — GPU hardware swapchain for the viewport

**Status**: 📋 Plan (awaiting approval). Parent: `.agents/PLAN/15-0-production-readiness.md` §3.
**Working model**: build in GitHub CI → download the `clawvinci-windows-x64`
artifact → run on the user's GPU laptop → user reports results. No local
toolchain; each iteration = one CI build + one manual run. Code must therefore
be **self-diagnosing**: log adapter/backend selection, surface creation, and
errors somewhere the user can copy back.

---

## 1. Findings that shape this plan

- **No wgpu exists anywhere.** The render crate's blurb says "wgpu effects,"
  but compositing is 100% CPU (`clawvinci_render::compositor::composite_frame`
  → `VideoFrame { width, height, data: Vec<u8> /* RGBA8 */ }`). Item 3 is a
  net-new GPU stack, not wiring existing infra.
- **The current preview is effectively broken.** `playback_render_frame`
  base64-encodes the RGBA buffer; the frontend sets `img.src =
  "data:image/raw;base64,…"`, which is not a decodable image type, so the
  `<img>` load fails and the canvas shows the "Clawvinci Frame N" fallback
  text. So this item builds the *first working* preview present path — there is
  no real before/after "smoothness" baseline to compare against, only "broken →
  GPU-presented."
- **Frame source is unchanged** (Decision-safe): `PlaybackEngine::
  render_current_frame() -> VideoFrame` stays the producer. This item changes
  *where the frame is presented*, honoring the shared-compositor invariant
  (Phase 4/7).
- **Decision 7**: DX12 primary, Vulkan fallback, via wgpu — the backend
  selection must actually try Vulkan if DX12 init fails, and we must log which
  backend won.

## 2. The core problem: presenting GPU output with an HTML UI ("airspace")

Tauri fills the OS window with a single WebView2 control. There are three ways
to get a GPU surface onto the viewport; only one is realistic here:

- **A. Native child-HWND overlay (chosen).** Create a Win32 child window
  parented to the Tauri window's HWND, positioned over the preview region;
  create a wgpu `Surface` from that child HWND; present frames there. The native
  surface floats *above* the webview — HTML cannot draw over it. Because the
  preview is a fixed rectangular region, this is acceptable, with two
  mitigations: (1) JS reports the preview rect so the overlay tracks
  layout/resize; (2) the overlay is hidden whenever a full-window HTML layer
  (Settings modal, onboarding, dropdown over the preview) is open.
- **B. WebView2 composition mode** (DComp visual / `ICoreWebView2Composition
  Controller`): composits cleanly with HTML, no airspace problem — but Tauri v2
  runs WebView2 in *windowed* mode and does not expose composition mode.
  Retrofitting means patching wry/Tauri. **Rejected** as out of proportion.
- **C. Faster in-webview present** (shared-memory RGBA → `ImageBitmap` → canvas):
  not a GPU swapchain; **rejected** — doesn't meet the item's intent.

**Chosen: Option A**, staged.

## 3. Threading & lifecycle risk (resolve in Stage 1)

- A wgpu `Surface` tied to a raw HWND is not freely `Send`/`Sync` and the child
  window's messages belong to the thread that created it. Tauri commands run on
  arbitrary worker threads, so the presenter cannot simply be a
  `tauri::State` poked from any command.
- **Design**: own the presenter on a **dedicated render thread**. Create the
  child window + wgpu device/surface there; drive it via an `mpsc` channel of
  messages (`SetViewport{rect,dpi}`, `Present{frame_index, rgba, w, h}`,
  `SetVisible(bool)`, `Resize`, `Shutdown`). Tauri commands just send messages.
  This sidesteps `Send` constraints and keeps all Win32/wgpu calls on one
  thread.
- Frames still originate on the app side: a command locks `AppState`, calls
  `render_current_frame()`, and ships the `VideoFrame.data` over the channel.
  (Later optimization: render on the render thread to avoid a copy — not in
  scope now.)

## 4. Staged implementation (each stage is independently CI-buildable & testable)

### Stage 0 — Prove wgpu initializes on the real GPU (no UI/present yet)
- Add deps to the **main crate** (`windows/src-tauri/Cargo.toml`): `wgpu`,
  `pollster`, `bytemuck`, `raw-window-handle`, and `windows` (Win32 features for
  child-window creation, added in Stage 1).
- New `windows/src-tauri/src/gpu_present.rs`: `fn probe() -> GpuProbe` that
  builds a wgpu `Instance` with `backends: DX12 | VULKAN`, requests the
  high-performance adapter, and returns `{ backend, adapter_name, device_ok }`.
- Command `gpu_probe()` returns that; also write it via `tracing`/to a log line.
- Frontend: call `gpu_probe()` on startup and log to console (and optionally show
  in Settings → General). **User test**: launch artifact, report the reported
  backend + adapter (confirms DX12-or-Vulkan works on the laptop).
- **Exit criteria**: adapter + device create successfully; backend logged.

### Stage 1 — Child-HWND surface presenting a clear color
- On the render thread: create a child `HWND` (`CreateWindowExW` + `SetParent`
  to the main window's HWND, `WS_CHILD | WS_VISIBLE`), create a wgpu `Surface`
  from it, configure it, and render a solid clear color (magenta) so it's
  obviously visible.
- Get the main window HWND in Tauri's `.setup()` (window exists there);
  spawn the render thread with it.
- Commands: `gpu_set_viewport_rect(x,y,w,h,dpi)` (JS sends the
  `#preview-canvas` `getBoundingClientRect()` + `devicePixelRatio`) and
  `gpu_set_visible(bool)`. Reposition/resize the child window on rect changes;
  hide on modal open.
- Frontend: report the rect on load, on window resize, and on layout changes;
  hide the overlay when the Settings/onboarding modal opens.
- **User test**: a magenta rectangle sits exactly over the preview area, tracks
  window resize, and disappears when Settings opens. **This validates the whole
  airspace approach before any frame work.**
- **Exit criteria**: correctly-placed, resize-tracking, hideable GPU surface.

### Stage 2 — Stream composited frames to the surface
- Add a textured-quad pipeline: first WGSL in the project (`fullscreen_quad`
  vertex + sampled-texture fragment), a `wgpu::Texture` sized to the frame, a
  sampler, and a bind group.
- `gpu_present_frame(frame)` command: lock `AppState`, `seek` + `render_current_
  frame()`, send `Present{ rgba, w, h }` to the render thread, which uploads to
  the texture (`queue.write_texture`) and draws the quad. Recreate the texture
  when frame dimensions change.
- Frontend: replace `renderCurrentFramePreview()`'s base64 path with
  `gpu_present_frame(frame)` when GPU is active; keep the old canvas path as an
  automatic fallback if `gpu_probe()` failed.
- **User test**: scrubbing/seeking shows real composited frames on the GPU
  surface; report responsiveness.
- **Exit criteria**: real frames present; scrub feels interactive.

### Stage 3 — Robustness, DPI, measurement, cleanup
- DPI-correct sizing (physical px = logical × dpi); robust resize (reconfigure
  surface on `Resized`/rect change); teardown on window close (`Shutdown`
  message, destroy child window); handle surface-lost/outdated.
- Visibility sync for every HTML layer that can cover the preview (modals,
  dropdowns, onboarding, fullscreen).
- Measure scrub responsiveness on a representative 4K timeline and record the
  number (DoD asks for an actual figure, not "smooth").
- Retire the broken `image/raw` base64 path (keep a CPU fallback only for the
  no-GPU case).

## 5. Files touched (projected)

| File | Change |
|---|---|
| `windows/src-tauri/Cargo.toml` | add wgpu, pollster, bytemuck, raw-window-handle, windows (Win32) |
| `windows/src-tauri/src/gpu_present.rs` | **new** — probe, render thread, child HWND, surface, texture pipeline, message channel |
| `windows/src-tauri/src/lib.rs` | manage presenter handle; `.setup()` window-handle wiring; commands `gpu_probe`/`gpu_set_viewport_rect`/`gpu_set_visible`/`gpu_present_frame`; register in `invoke_handler` |
| `windows/src/index.html` | probe on load; report preview rect + dpi; present via GPU; hide overlay on modal; CPU fallback |
| shader (inline WGSL in `gpu_present.rs`) | fullscreen textured quad |

## 6. Risks

- **Airspace/occlusion** — native overlay can't be covered by HTML. Mitigated by
  fixed preview rect + hide-on-overlay. Residual: transient popups over the
  preview are hidden while shown; acceptable.
- **Thread/`Send` constraints** — resolved by the dedicated-render-thread +
  channel design; if it proves wrong, fall back to `wgpu::rwh` on the main
  window and a full-window surface behind a transparent webview (larger change).
- **Tauri/wry version coupling** — child-HWND parenting relies on the window's
  real HWND, which Tauri exposes; low risk but version-sensitive.
- **DPI mismatches** — preview misaligned on scaled displays; addressed in
  Stage 3, watch from Stage 1.
- **CI can't verify any of it** — every stage depends on the user's manual run;
  hence the heavy logging and small, independently-testable stages.

## 6a. Hardware requirement & graceful degradation (non-negotiable)

GPU is a **recommended**, not a **required**, capability. The app must launch
and edit on any Windows 10 22H2+ machine even if no usable GPU backend exists.

- The frame producer (`composite_frame`) is CPU and never depends on a GPU, so
  editing / export / timeline are unaffected regardless of GPU.
- Presentation degrades in this order: **DX12 → Vulkan → WARP (software DX12)
  → CPU preview path**. wgpu covers the first three; the last is our own
  fallback.
- **Scope consequence:** the CPU fallback must actually work. The current
  base64 preview is broken (invalid `image/raw` MIME), so fixing the non-GPU
  preview to a decodable encoding is part of this item — otherwise "falls back
  to CPU" means "falls back to a broken preview." `gpu_probe()` failure selects
  this path at runtime; no rebuild or flag needed.
- No feature is GPU-gated. GPU only makes the preview faster; its absence never
  blocks any action.

## 7. Definition of done (this item)

- wgpu initializes on the target GPU with a logged backend (DX12 or Vulkan
  fallback per Decision 7).
- The preview viewport is presented via a GPU surface, correctly positioned and
  DPI-correct, tracking resize and hiding under HTML overlays.
- Real composited frames display; scrub responsiveness on a 4K timeline is
  measured and recorded.
- Shared `composite_frame` remains the sole frame producer (no forked
  compositor).
- The broken `image/raw` base64 preview path is retired and replaced by a
  working CPU preview, used automatically when GPU init fails.
- Verified: the app launches and edits on a machine where `gpu_probe()` fails
  (or is forced to fail), using the CPU preview — GPU is recommended, not
  required.
