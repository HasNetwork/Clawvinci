# RULES.md — coder agent rules for the Windows port

These apply on top of the user's global engineering rules (forwarded to you
separately). Project-specific rules only — explanations of *why* live in
`PLAN.md` and the phase docs, not here.

## Scope discipline

- **Never edit anything outside `windows/`.** The repo root
  (`Sources/`, `Metal/`, `Plugins/`, `models/`, `scripts/`, `mcpb/`,
  `AGENTS.md`, `README.md`, `Package.swift`, etc.) is the read-only GPLv3
  macOS reference. Read it freely; write to it never.
- Follow `PLAN.md`'s phase order. Don't start Phase N+1 code before Phase
  N's "Definition of done" is met — later phases depend on earlier ones'
  shapes being settled, and reordering compounds rework.
- If a phase doc doesn't exist yet for the work you're about to do, write
  it first (see Rule 13 in the global rules) — don't improvise
  undocumented structure.

## Licensing

- Every new file ships GPLv3 (project-wide decision, see `PLAN.md`
  Decision 4).
- When a Rust file is a direct translation of a specific Swift source file
  (not just "inspired by the general approach"), add a header comment
  naming that source file, e.g. `// Derived from
  Sources/PalmierPro/Models/Timeline.swift (GPLv3).` This is for our own
  traceability, not a legal requirement beyond GPLv3's own terms.
- Do not introduce a dependency whose license is incompatible with GPLv3
  distribution (e.g. a proprietary SDK with a no-redistribution clause) without
  flagging it — GPLv3 has copyleft implications for what you can link
  against.
- Don't use "Palmier Pro" branding/name/logo in anything user-facing yet —
  see the open naming question in `PLAN.md`. Use the working name.

## Translating from the Swift reference

- Read the actual source file before porting it — don't infer its
  behavior from the filename or from this plan's summary. The plan docs
  describe *what* and *why*; the Swift file is the *how*.
- Preserve behavior, not syntax. A Swift `final class` used for reference
  semantics (e.g. `MediaAsset`), an `@unchecked Sendable` with a manual
  thread-safety argument (e.g. `MediaResolver`), or a `Codable` fallback
  path (e.g. `ProjectFile.decode`'s legacy handling) all encode a decision
  someone made for a reason. Understand the reason before you decide
  whether Rust needs the equivalent pattern or a different one — don't
  reflexively flatten everything to a plain struct.
- Where the Swift source's `CodingKeys` or field names determine an
  on-disk JSON shape, match them exactly unless the phase doc says the new
  format is intentionally divergent (see Phase 1's format decision).
- Root `AGENTS.md`'s engineering invariants (concurrency, file I/O safety,
  performance, undo semantics, correctness/edge cases) describe required
  *behavior*, translated below for this stack — they are not Swift-specific
  advice to discard.

## Rust/Tauri equivalents of the macOS invariants

Restating root `AGENTS.md`'s rules for this stack — same requirements,
different mechanism:

- **Main-actor discipline → IPC boundary discipline.** The Swift rule "keep
  observable UI state on the main actor, cross with immutable values" maps
  to: UI state lives in the frontend (or a Tauri-managed state struct); all
  blocking work (file I/O, decode, render, model inference) runs in Tokio
  tasks or `spawn_blocking`, never on a Tauri command handler that the UI
  thread waits on synchronously. Cross the IPC boundary with plain
  serializable values, not shared mutable state.
- **File I/O off the main thread → async all the way.** Every filesystem
  operation (read, write, enumerate, metadata, copy, move, delete) runs via
  `tokio::fs` or `spawn_blocking`, never synchronously in a command handler.
  Assume slow/removable/network volumes. Stage output outside the live
  project package; atomic rename to install. Serialize operations that
  target the same package (mirrors `ProjectPackageCoordinator`).
- **CMTime / frame-domain time → keep integers.** Preserve frame-domain
  integer time (matching `Timeline`/`Clip`'s `Int` frame fields) as long as
  possible; convert to floating point only at UI/external-format
  boundaries (same rule the Swift code follows, ported directly since the
  model layer is a straight translation).
- **Cancellation propagation.** Decode, render, inference, export, and
  generation loops must check for cancellation between chunks — use
  `tokio_util::sync::CancellationToken` or equivalent, propagated the same
  way the Swift code propagates `Task` cancellation.
- **Bounded concurrency.** Cap concurrent decoders/readers/exports/
  inference/thumbnail/waveform work — don't let a batch operation spawn
  unbounded tasks.
- **Undo.** UI edits and MCP agent edits route through the same mutation
  operations and a shared undo history (mirrors "Editor mutations and
  undo"). One user-visible intent = one undo entry. Validate before opening
  an undo group; failed/no-op operations create no entry. No cumulative
  rounding drift on undo/redo.
- **Performance-sensitive paths.** No file I/O, logging, JSON
  encode/decode, decoder/reader setup, or GPU pipeline setup inside a
  per-frame/per-sample/per-item hot path. Reuse decoders, render contexts,
  and audio graphs across frames; don't rebuild per interaction. Lazy-load
  thumbnails/waveforms/transcripts/models.
- **Errors surfaced, not swallowed.** No `.unwrap_or_default()`-as-silent-
  failure or empty-result-on-error for anything the user asked for. Return
  `Result` and let the caller decide how to present it — matches "surface
  user-requested file failures."

## Rust idioms for this project

- No `unwrap()`/`expect()`/`panic!` in any code path reachable from a Tauri
  command, file I/O, network call, or media decode — these must return
  `Result`. `unwrap()` is fine only in tests or in code proven unreachable
  by a prior `match`/type-level guarantee.
- Prefer `thiserror` for library-crate error types, matching each crate's
  own failure domain, rather than one giant workspace-wide error enum.
- Async runtime: Tokio, consistently, across all crates — don't mix in a
  second async runtime.

## Rule 2 applied here specifically

- Don't build a phase's abstraction ahead of the phase that needs it (e.g.
  don't design the effects-pipeline trait system while doing Phase 1
  domain models). Each phase doc's "Definition of done" is the scope
  boundary for that phase's work.
- Don't detail phase docs for phases that aren't next — write a phase doc
  when that phase is about to start, informed by what the previous phases
  actually produced, not speculatively now.

## Verification

- Every phase's "Definition of done" in its `PLAN/*.md` doc is the
  acceptance bar — don't mark a subphase complete in `HISTORY.md` until
  it's met.
- `cargo test` (or `cargo nextest run`) and `cargo clippy --workspace -- -D
  warnings` must both pass before a subphase is considered done.
- State explicitly in the `HISTORY.md` log entry if anything was skipped,
  deferred, or only partially verified — per the global "fail loud" rule.
