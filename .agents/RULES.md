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
- Ship under the "Clawvinci" name (Decision 5, `PLAN.md`). Never use
  "Palmier Pro" branding/name/logo anywhere user-facing — it's proprietary
  to Palmier, Inc. even though this source is GPLv3.
- **No accounts, no Palmier-operated backend, no telemetry** (Decision 10,
  `PLAN.md`). Never add Clerk/auth code, never call `api.palmier.io` or any
  other Palmier/Convex-hosted endpoint, never add Sentry/PostHog or any
  other phone-home telemetry. Every AI capability is BYOK (user's own key,
  direct to the provider) or fully local/on-device — no proxy backend in
  between. If you find code that violates this (e.g. a hardcoded Palmier
  URL from before Decision 10), fix it, don't extend it.

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
  boundary for that phase's work — this applies to *code*, not to the
  planning docs themselves (all 13 phases are already detailed in
  `PLAN/`, at the user's request, so writing ahead doesn't apply to
  planning going forward).
- When a phase's actual implementation surfaces something its doc got
  wrong or didn't anticipate (a Rust crate that doesn't exist as assumed,
  a Swift behavior that reads differently once you're actually porting
  it), update that phase's `PLAN/*.md` doc to match reality before or
  alongside implementing — don't let the doc silently drift out of sync
  with the code. Note the correction's reasoning in the `HISTORY.md` log
  entry for that subphase.
- If a phase's doc turns out to need splitting into subphases once work
  starts (e.g. Phase 6 UI shell is large enough that "6.1 design tokens,"
  "6.2 timeline canvas," "6.3 inspector" might make more sense than one
  monolithic 6.0), that split itself follows Rule 2 — do it when the
  phase is actually starting and the natural seams are concrete, not
  speculatively now.

## Verification

- **A mock, stub, or placeholder reachable from a non-test code path is
  never a valid Definition-of-done, regardless of CI status.** CI passing
  only proves compilation and whatever tests were written — it proves
  nothing about whether a `Mock*`/`Deterministic*`/simulated
  implementation is what a real user's call actually hits at runtime. If a
  mock exists to keep surrounding orchestration testable during
  development, that's fine, but the phase is not done until every
  production code path is swapped to the real implementation, and a test
  exists that would fail if a mock were still wired in. (See
  `.agents/HISTORY/14-0-mock-audit-findings.md` for exactly this failure
  mode occurring in Phases 10–12: three "complete" subsystems were
  self-reported done while a mock was still reachable in production.)
- Every phase's "Definition of done" in its `PLAN/*.md` doc is the
  acceptance bar — don't mark a subphase complete in `HISTORY.md` until
  it's met.
- **No Local Tool Installation / Runs**: Do NOT install or run local `cargo`,
  `rustc`, `npm`, or native build/test CLI tools on the user's host machine.
  The host environment is strictly for agent file editing and git operations.
- **GitHub CI is the Verification Environment**: All compilation, unit/integration
  tests (`cargo test --workspace`), and linter runs (`cargo clippy --workspace -- -D warnings`)
  are executed remotely on GitHub Actions CI (`.github/workflows/ci.yml`). Code must be
  written carefully with high precision to pass CI cleanly on the first run.
- State explicitly in the `HISTORY.md` log entry if anything was skipped,
  deferred, or only partially verified — per the global "fail loud" rule.
