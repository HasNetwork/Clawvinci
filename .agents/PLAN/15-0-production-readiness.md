# Phase 15.0 — Production readiness / commercial release

Formalizes the "5-Step Roadmap to 100% Commercial Release" the coder agent
wrote into `Handover.md` §9 after Phase 13, as a real tracked phase instead
of a paragraph in a handover doc. **Sequenced after Phase 14** — packaging
and distributing an app whose flagship AI features are mocked would just
ship the problem Phase 14 exists to fix; get the AI pipelines real first.

Unlike Phases 0–13 (porting behavior from the macOS reference) and Phase 14
(making already-claimed work actually real), this phase is genuinely new
scope: closing the gap between "feature-complete and CI-green" and
"something you'd hand a real user on a clean Windows PC."

## 1. Bundle FFmpeg as a Tauri sidecar

- Current: `clawvinci-media` shells out to `ffmpeg`/`ffprobe` on `PATH`.
  Fails on any machine without them pre-installed.
- Download pinned `ffmpeg.exe`/`ffprobe.exe` builds in release CI, package
  as Tauri external binaries (`windows/src-tauri/binaries/`), and update
  `clawvinci-media`'s process-spawn code to resolve the bundled path first,
  falling back to `PATH` only if explicitly configured to (useful for
  developer builds, not the shipped installer).
- Verify the license terms of whichever FFmpeg build is bundled are
  compatible with GPLv3 distribution (LGPL vs GPL build flavor matters
  here — check before picking a source).

## 2. Code-sign the installer

- Current: NSIS installer `.exe` is unsigned — triggers Windows Defender
  SmartScreen "unknown publisher" warnings.
- Acquire an Authenticode code-signing certificate, store it as a GitHub
  Actions secret, and sign the installer in the release workflow
  (`.github/workflows/release.yml`). Note real cost/lead-time here (EV
  certs in particular require identity verification) — flag back to the
  user if this blocks the phase rather than silently skipping it.

## 3. GPU hardware swapchain for the viewport

- Current: playback composites frames via the CPU `composite_frame` path
  and transfers image buffers into the webview — works, but won't hit
  smooth 4K scrub performance.
- Connect the wgpu DX12 (primary, Vulkan fallback per Decision 7) render
  target directly to a native child HWND or a WebView2 composition target,
  so presentation happens on the GPU without a CPU roundtrip per frame.
- This changes the playback pipeline's performance characteristics, not
  its correctness contract — the shared `composite_frame` invariant
  (Phase 4/7) must still hold; this phase changes *where frames get
  presented*, not which function produces them.

## 4. Real-world media stress testing

- Current: verified against standard MP4/H.264, WAV audio, synthetic test
  vectors.
- Stress-test against: high-bitrate ProRes, variable-frame-rate smartphone
  video, 10-bit 4:2:2 footage, corrupt/truncated container headers, very
  long-duration files. Record failures as bugs against the owning crate
  (likely `clawvinci-media`/`clawvinci-render`), not as scope for this
  phase to silently absorb — a decode bug is a `clawvinci-media` bug
  regardless of which phase found it.

## 5. Timeline canvas interaction polish

- Current: click-to-seek, split, trim, ripple delete, and IPC commands
  work; no multi-clip selection or snapping.
- Implement multi-clip rubberband/lasso selection and magnetic snapping
  guides on the canvas timeline (Phase 6's UI shell).

## 6. In-app AI model downloader (depends on Phase 14)

- Once Phase 14 makes the local Whisper/SigLIP2 download mechanisms real
  (not simulated), add the Settings-pane UI (Phase 13's `ModelsPane`) to
  trigger and show progress for those downloads — the backend mechanism is
  Phase 14's job; this is just the UI surfacing it.

## Definition of done for Phase 15

- Clean-PC install test: a Windows 10 22H2 VM with nothing pre-installed
  can install and run Clawvinci, decode/export real media, without any
  manually-installed dependency.
- Installer is signed; no SmartScreen "unknown publisher" warning on a
  fresh install.
- Playback hits real interactive scrub rates on a representative 4K
  timeline (measure and record the actual number — don't just assert
  "smooth").
- Stress-test media set (ProRes/VFR/10-bit/corrupt-header samples)
  documented with pass/fail per case; failures filed as bugs, not silently
  patched over inside this phase.
- Canvas supports multi-select + snapping, verified interactively (per
  root `AGENTS.md`'s UI-testing rule: manual verification required, agent
  provides the test plan, doesn't self-certify).
