# Phase 15.0 — Production readiness: Implementation Log

**Status**: 🟡 In progress — Items 1 and 6 implemented and CI-green; Items 2–5
pending (blocked on decisions/verification that require the user).
**Plan**: `.agents/PLAN/15-0-production-readiness.md`
**CI**: green on run **34754678806** (compile, clippy `-D warnings`, tests,
Tauri bundle including the FFmpeg sidecar).

---

## Item 1 — Bundle FFmpeg as a Tauri sidecar ✅

### What changed

**`clawvinci-media/src/ffmpeg.rs`**:
- `find_binaries()` gained a step 0: resolve `ffmpeg`/`ffprobe` next to the
  application executable (`current_exe().parent()`) before the env override,
  common install dirs, and PATH search. In the installed app Tauri places the
  external binaries alongside `clawvinci.exe`, so a clean Windows PC with no
  system FFmpeg works. PATH fallback is retained for developer builds.

**`tauri.conf.json`**:
- Declared `bundle.externalBin: ["binaries/ffmpeg", "binaries/ffprobe"]` so the
  bundler copies them next to the exe. On the host target the bundler expects
  `binaries/ffmpeg-x86_64-pc-windows-msvc.exe` (and ffprobe) to exist at build.

**`.github/workflows/ci.yml` and `release.yml`**:
- Pinned FFmpeg via `FedericoCarboni/setup-ffmpeg@v3` with
  `ffmpeg-version: ">=6.1.0"` (see fix note below) — its Windows source is
  gyan.dev, which ships **GPL** builds, compatible with Clawvinci's GPLv3
  distribution.
- Added a "Bundle FFmpeg sidecar binaries" step that copies the resolved
  `ffmpeg.exe`/`ffprobe.exe` (located via `Get-Command`) into
  `windows/src-tauri/binaries/` with the `-x86_64-pc-windows-msvc` suffix,
  before the Tauri build. Both workflows bundle, so both needed this.

**`.gitignore`**:
- Excluded `windows/src-tauri/binaries/` — the sidecar binaries are fetched in
  CI and never committed.

### License note

Confirmed: `FedericoCarboni/setup-ffmpeg@v3` sources Windows builds from
gyan.dev, which are GPL-licensed FFmpeg builds. Bundling a GPL FFmpeg inside a
GPLv3 application is license-compatible. (An LGPL build would also be fine, but
GPL is what this action provides and it is not a problem here.)

### Not yet done for Item 1

- The DoD's clean-PC install test (a Windows 10 22H2 VM with nothing
  pre-installed installs and decodes/exports real media) is a runtime check
  that can't run in the compile-only CI — needs a manual VM test.

---

## Item 6 — In-app AI model downloader ✅ (code) / needs manual UI check

The Phase 14 audit fix made `LocalWhisperEngine::download()` and
`VisualModelLoader::download()` real, but nothing exposed them to the UI —
there were no IPC commands. This item added that bridge plus the UI.

### What changed

**`windows/src-tauri/src/lib.rs`**:
- `ModelEngines` managed state holding `Arc<LocalWhisperEngine>` and
  `Arc<VisualModelLoader>` over `%LOCALAPPDATA%/Clawvinci/Models`
  (`clawvinci_search::model_download::default_models_dir()`). Both engines are
  `Send + Sync` (their transcriber/embedder traits require it), so they are
  valid Tauri state and safe to `tokio::spawn`.
- `model_status` command → returns per-model `{ name, installed, sizeMb, state }`
  where `state` is the serializable `LoaderState`
  (`Unknown`/`NotInstalled`/`Downloading(f64)`/`Preparing`/`Ready`/`Failed`).
- `model_download` command → spawns a background task running the engine's real
  `download()` and returns immediately; the UI polls `model_status` for progress.

**`windows/src/index.html`**:
- "Local AI Model Downloads" section in the Settings → Models pane with a row
  per model (name, status text, progress bar, download/retry button).
- JS: `refreshModelStatus()` renders both rows and starts a 1s poll while any
  download is active; button clicks call `model_download`; polling is cleared
  when the settings modal closes.

**`windows/src/app-theme.css`**:
- `.model-dl-row`/`.model-dl-info`/`.model-dl-name`/`.model-dl-status`/
  `.model-dl-progress`/`.model-dl-bar` styles.

### Manual verification needed (not self-certified)

Per the root `AGENTS.md` UI-testing rule, CI only proves this compiles. Test
plan for the user:
1. Launch the app, open Settings → Models → "Local AI Model Downloads".
2. Both models should read "Not installed (~N MB download)".
3. Click Download on Whisper; the button shows "Downloading…" and the bar
   advances; on completion it reads "Installed (~N MB)".
4. Repeat for SigLIP2 (larger — ~340 MB total across three files).
5. Reopen Settings later; installed models should still show "Installed".

---

## CI fix trail

- `ffmpeg-version: "7.1.0"` → rejected ("Requested version is not available").
- `"6.1.0"` → also rejected: setup-ffmpeg@v3 does not index exact patch
  versions on Windows.
- `">=6.1.0"` → accepted; the action documents semver specifiers as the
  Windows-supported form. Final value.

---

## Remaining Phase 15 items (need the user)

- **Item 2 — Code-sign the installer**: requires purchasing an Authenticode
  cert (money + identity verification). Cannot be done by the agent.
- **Item 3 — GPU hardware swapchain**: large, risky rework of the playback
  present path; needs a real GPU and interactive scrub measurement to verify.
- **Item 4 — Real-world media stress testing**: needs real ProRes/VFR/10-bit/
  corrupt-header media and a running app.
- **Item 5 — Canvas interaction polish**: multi-select + snapping; needs
  interactive UI verification.
