# Phase 6.0 — UI Shell (Tauri / Web Frontend)

**Status**: ✅ Complete  
**Date**: 2026-09-06  

## What was built

Phase 6.0 implements the complete UI Shell for Clawvinci, connecting the Tauri v2 web frontend with the underlying `clawvinci-*` Rust crates using the **Premium Dark Design System**:

1. **Design Token System (`windows/src/app-theme.css`):**
   - Mapped all design tokens from `Sources/PalmierPro/UI/AppTheme.swift` and `TimelineClipColors.swift` to CSS custom properties.
   - Strictly conforms to the **Premium Dark Design System** (monochromatic dark grays, no emojis, light weights for large metrics, tabular figures for timecode, desaturated NLE clip palette, refined borders and scrollbars).

2. **Tauri IPC Command Surface (`windows/src-tauri/src/lib.rs`):**
   - Introduced an integrated `AppState` that pairs `TimelineEditor` (undo/redo, clip ops, track ops) with `PlaybackEngine` (frame planning, clock sync, composition rendering) and the media pool.
   - Added 16 new IPC commands:
     - `timeline_get`: Retrieves current `Timeline` state.
     - `timeline_split_clip`: Splits clip at target frame using `clip_ops::split_clip`.
     - `timeline_trim_clip`: Trims clip head or tail with `ripple::TrimEdge`.
     - `timeline_move_clips`: Repositions clips across tracks and time.
     - `timeline_remove_clips`: Removes selected clips.
     - `timeline_ripple_delete`: Ripple deletes clips and closes gaps.
     - `timeline_insert_track`: Inserts new video/audio/text tracks.
     - `timeline_undo` & `timeline_redo`: Shared undo/redo history.
     - `timeline_history_status`: Returns `{ can_undo, can_redo, undo_name, redo_name }`.
     - `timeline_update_clip_transform`: Updates position, scale, rotation, and opacity with live preview invalidation.
     - `timeline_update_clip_effect`: Updates effect parameters (exposure, contrast, highlights, shadows, vignette, blur).
     - `timeline_update_clip_text`: Updates text content and font size.
     - `media_list` & `media_import`: Asset management with `clawvinci_media::probe_media`.

3. **Interactive Timeline Canvas (`windows/src/timeline.js`):**
   - High-performance HTML5 canvas rendering tracks, clips, type badges, duration labels, and time ruler.
   - Interactive playhead with exact frame snap and scrubbing.
   - Click to select clips, drag to move, edge handles to trim head/tail.
   - Zoom in/out via slider or `Ctrl` + mouse wheel.
   - Integrated keyboard shortcuts (`Space` for play/pause, `S`/`C` for split, `Delete`/`Backspace` for ripple delete, `Ctrl+Z` for undo, `Ctrl+Y` for redo).

4. **Inspector Property Editor (`windows/src/inspector.js`):**
   - Tabbed inspector:
     - **Adjust Tab**: Transform (position, scale, rotation, opacity), Tone controls (exposure, contrast, highlights, shadows), Stylize (vignette, blur).
     - **Audio Tab**: Volume slider with decibel mapping.
     - **Text Tab**: Real-time text content editor and font size slider.
   - Live debounced updates reflected immediately in the preview composite.

5. **Media Panel Asset Browser (`windows/src/media_panel.js`):**
   - Asset grid displaying imported media with thumbnail placeholders, duration, resolution, and framerate.
   - Live search filter input.
   - File import prompt and double-click to add to timeline.

6. **Professional 3-Pane Layout (`windows/src/index.html`):**
   - Unified workspace layout: Top bar with branding, history, and transport controls; Top split pane (Media Panel, Viewport, Inspector); Bottom split pane (Timeline Canvas & Track Headers); Status footer with live metrics.
