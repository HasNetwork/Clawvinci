# Phase 13.0 — Polish: Localization, Settings, Home/Onboarding, Help & Windows MCP

## Overview
Phase 13.0 completes the final polish layer for Clawvinci, delivering a cohesive, native Windows desktop experience:
- Client-side 28-locale localization system ported from Palmier Pro's original string inventory.
- Complete settings system with persistent JSON storage on Windows `%APPDATA%`, cache inspection and purging, and direct BYOK provider configuration.
- Home Screen (Project Hub) with Recent Projects list, New Project creation, and Sample Projects.
- First-run 4-step onboarding wizard.
- Windows-accurate Help system with Windows keybindings (`Ctrl`/`Alt`), 1-click copy MCP setup guides for Claude Desktop, Cursor, Claude Code, and Codex, and an interactive 5-step guided tour.

---

## Key Deliverables

### 1. Localization Subsystem (`windows/src/locales/catalog.js` & `windows/src/i18n.js`)
- Extracted 28 language catalogs from `Sources/PalmierPro/Resources/Localization/*.lproj/Localizable.strings`:
  - English (`en`), Spanish (`es`), French (`fr`), German (`de`), Japanese (`ja`), Simplified Chinese (`zh-Hans`), Traditional Chinese (`zh-Hant`), Arabic (`ar`), Bengali (`bn`), Czech (`cs`), Danish (`da`), Finnish (`fi`), Hebrew (`he`), Hindi (`hi`), Indonesian (`id`), Italian (`it`), Korean (`ko`), Norwegian (`nb`), Dutch (`nl`), Polish (`pl`), Portuguese Brazil (`pt-BR`), Portuguese Portugal (`pt-PT`), Russian (`ru`), Swedish (`sv`), Thai (`th`), Turkish (`tr`), Ukrainian (`uk`), Vietnamese (`vi`).
- Built `windows/src/i18n.js` providing:
  - `t(key, fallback)` translation lookup with English fallback.
  - `setLanguage(langCode)` with reactive event dispatch (`clawvinci-locale-changed`).
  - `updateDomTranslations()` updating all `[data-i18n]`, `[data-i18n-title]`, and `[data-i18n-placeholder]` DOM elements.
  - Native display names in the General Settings language picker.

### 2. Settings & Storage Backend (`windows/src-tauri/src/settings.rs`)
- Defined domain structs with full CamelCase serde mapping:
  - `AppSettings`: language, default frame rate, theme, track colors, BYOK keys map, Whisper settings, recent projects, onboarding completion status.
  - `TrackColorsDto`: custom hex colors for video, audio, text, and image tracks.
  - `WhisperSettingsDto`: transcription mode (`auto`, `local`, `byok`), model size (`tiny`, `base`, `small`), device (`cpu`, `directml`).
  - `StorageInfoDto`: cache sizes for thumbnails, waveforms, transcripts, and total disk usage.
  - `RecentProjectDto`: project ID, display name, local path, timestamp, duration.
- Windows file paths:
  - Config: `%APPDATA%/Clawvinci/settings.json` (fallback `%USERPROFILE%/.clawvinci/settings.json`).
  - Cache: `%LOCALAPPDATA%/Clawvinci/Cache` (fallback `%USERPROFILE%/.clawvinci/cache`).
- Tauri IPC Commands registered in `lib.rs`:
  - `settings_get`, `settings_update`
  - `storage_get_info`, `storage_clear_cache`
  - `project_recent_list`, `project_create`

### 3. Settings UI (`windows/src/index.html`)
- Sidebar-tabbed settings modal matching Palmier Pro aesthetic:
  - **General**: Language selector (28 languages), default frame rate (24, 25, 29.97, 30, 59.94, 60 fps), software update checker.
  - **Models (BYOK)**: Direct API key inputs with password-masking toggles for OpenAI, Kling, Seedance, Fal, ElevenLabs, Suno; on-device Whisper model configuration.
  - **Agent & MCP**: Live MCP status display on port 19789 (53 active tools), in-app chat API key, quick link to Windows MCP setup guide.
  - **Appearance**: Live color swatches for video, audio, text, and image clips mapped directly to CSS root variables.
  - **Storage**: Real-time cache metric reporting with granular and bulk cache clearing.
- *Decision 10 Compliance*: The legacy macOS `AccountPane` (Clerk auth and subscription credits) is completely excluded.

### 4. Home Hub & First-Run Onboarding
- **Project Hub (`#home-modal-backdrop`)**:
  - Welcome hero banner with quick actions.
  - Sample project templates (Cinematic Action Trailer, Podcast Split & Cutdown, Vertical Social Reel).
  - Recent projects table showing project paths, duration, and 1-click opening.
  - "New Project" creation dialog.
- **Onboarding Wizard (`#onboarding-modal-backdrop`)**:
  - Step 1: Welcome & core AI-native editing overview.
  - Step 2: Editing experience selector (Pro NLE, Creator, AI Engineer).
  - Step 3: Speech transcription & generation setup (Local Whisper vs BYOK).
  - Step 4: Ready to edit with timeline initialization.

### 5. Help, Windows Shortcuts & MCP Setup Guide
- **Shortcuts Pane**:
  - Windows-native bindings (`Ctrl` instead of `Cmd`, `Alt` instead of `Opt`):
  - Playback: Space, Arrow keys, J/K/L shuttle.
  - Editing: S / C / Ctrl+K (Split), Delete / Backspace (Ripple Delete), Q / W (Trim).
  - History: Ctrl+Z (Undo), Ctrl+Y / Ctrl+Shift+Z (Redo).
- **Windows MCP Setup Guide**:
  - Windows paths with 1-click clipboard copy:
  - Claude Desktop: `%APPDATA%\Claude\claude_desktop_config.json`
  - Cursor IDE: `%USERPROFILE%\.cursor\mcp.json`
  - Claude Code CLI: `claude mcp add --transport http clawvinci http://127.0.0.1:19789/mcp`
  - Codex CLI: `codex mcp add clawvinci --url http://127.0.0.1:19789/mcp`
- **Interactive Tour (`#tour-modal-backdrop`)**:
  - 5-step guided spotlight tour highlighting the Media Panel, Viewport, Timeline, Inspector, and MCP Server.
- **Feedback**:
  - Form connecting directly to GitHub Issues (`https://github.com/HasNetwork/Clawvinci/issues`).

---

## Verification
- Automated unit tests in `windows/src-tauri/tests/polish_tests.rs`:
  - `test_settings_default_and_roundtrip`: verifies default values and JSON serialization.
  - `test_track_colors_and_whisper_defaults`: verifies default color palette and Whisper configuration.
  - `test_byok_keys_storage`: verifies storing and retrieving multiple external provider keys.
  - `test_recent_projects_manipulation`: verifies project list ordering and deduplication.
  - `test_storage_info_and_cache_clear`: verifies cache path resolution and safe directory purging.
- **GitHub Actions CI Verification**:
  - Run ID: `34219245633` (commit `d41711f`)
  - Status: **100% Green (Success)**
  - Job: `Build, Lint & Test (Windows)` passed in 6m48s
  - `Cargo Check Workspace`: Passed
  - `Cargo Clippy`: Passed cleanly with zero warnings (`-D warnings`)
  - `Cargo Test`: Passed (including all unit tests in `polish_tests.rs`)
  - `Build Tauri Desktop Executable`: Successfully produced Windows release binary
  - Uploaded Artifact: `clawvinci-windows-x64`

