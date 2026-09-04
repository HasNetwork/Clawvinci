# Phase 7.0 — Export (`clawvinci-export`)

Batch rendering of a `Timeline` to a delivered file, plus interchange
export (FCPXML/native XML) for round-tripping into other NLEs. Depends on
Phase 2 (encode), Phase 4/5 (the shared `composite_frame` render path —
see Phase 4's "one render path, two consumers" section, which this phase
is the second consumer of), and Phase 1 (model types being exported).

## Source inventory

`Sources/PalmierPro/Export/` (4,161 LOC, 9 files):

| macOS file | LOC | Rust destination | Notes |
|---|---|---|---|
| `FCPXMLExporter.swift` | 1,020 | `clawvinci-export::fcpxml` | Final Cut Pro XML interchange, version-gated (`FCPXMLVersion` enum, default 1.10 for broadest DaVinci Resolve/FCP compatibility). Pure data transform (`Timeline` → XML tree) — no AVFoundation dependency in the transform itself beyond incidental `AppKit`/`CoreText` imports (check what those are actually used for — likely font-name/metrics lookups for text clips — and find the Windows-side equivalent, e.g. a font-metrics crate, rather than assuming they block porting). Highest-value near-term export target since it's the most portable of the export code. |
| `ExportView.swift` | 721 | Phase 6 (UI) | Not this phase — export options UI. |
| `XMLExporter.swift` | 656 | `clawvinci-export::xml` | Palmier's native XML interchange format — same portability profile as FCPXML. |
| `ExportService.swift` | 624 | `clawvinci-export::service` | Orchestrates the actual render-to-file: builds the render plan (reuses Phase 4's builder), drives `composite_frame` per output frame, feeds Phase 2's encoder. `ExportAnalyticsRun`/`ExportAnalyticsContext` — telemetry, defer wiring until Phase 12 but keep the seams (a place to record source/mode/format/resolution/duration) since they're cheap to carry along now. |
| `ExportQueue.swift` | 338 | `clawvinci-export::queue` | Job queue: `ExportJobStatus` (queued/preparing/rendering/canceling/completed/failed/canceled), source (`manual`/`agent`) — the agent-triggered path matters because the MCP `export_project`/`manage_exports` tools (Phase 8) drive this queue directly, same as a UI-triggered export. |
| `HDRVideoExporter.swift` | 291 | `clawvinci-export::hdr` | HDR-specific encode path — highest technical risk in this phase; de-risk after SDR export works, don't block the rest of the phase on it. |
| `ExportTimelineAnalyticsSnapshot.swift` | 237 | `clawvinci-export::analytics` (or fold into Phase 12) | |
| `PalmierProjectExporter.swift` | 165 | `clawvinci-export::project_bundle` | Exports a self-contained project bundle (media + project file) — straightforward given Phase 1's format decision (byte-compatible `.palmier`, Decision 9). |
| `ExportOptions.swift` | 109 | `clawvinci-export::options` | Format/resolution/codec option types — mostly data, ports directly. |

## Design notes

- **Reuse the Phase 4 render path, don't reimplement it.** `ExportService`
  must call the same `composite_frame` function the live preview uses
  (see Phase 4's architecture section) — the biggest risk in this phase
  isn't the encode side, it's someone building a second, subtly different
  compositor for batch export because it was easier to write standalone.
  Guard against this explicitly in code review, not just in this doc.
- **Cancellation**: `ExportJobStatus.canceling` is a real intermediate
  state (not just queued→completed/failed) because a render-in-progress
  needs to be told to stop and clean up mid-frame — matches root
  `AGENTS.md`'s cancellation-propagation rule, and Phase 2's decoder/
  encoder cancellation support is what this depends on.
- **Concurrency**: bound simultaneous export jobs (root `AGENTS.md`:
  "bound concurrent... exports"), and serialize an export against other
  operations touching the same project package (matches
  `ProjectPackageCoordinator`'s serialization rule from root `AGENTS.md`'s
  File I/O section — an export reading media while a save is
  installing new media into the package is a real race to prevent).

## Definition of done for Phase 7

- SDR H.264/HEVC export of a real multi-track, multi-effect timeline,
  verified by actually playing the output file back and comparing against
  the live preview for the same timeline — visual parity is the bar, not
  just "a file was produced."
- FCPXML export that a real installation of DaVinci Resolve (or another
  FCPXML 1.10 consumer) can import without errors — test against the
  actual target application, not just schema validity.
- Export queue: multiple queued jobs process correctly, a mid-render
  cancel actually stops and cleans up, a failed job reports a real error
  (not a silent empty result).
- `.palmier` project bundle export round-trips: export, then re-open the
  exported bundle, and confirm it matches the source project exactly.
- HDR export explicitly scoped in or out of this phase's "done" — don't
  let it silently slip without a decision recorded here once the phase
  starts.
