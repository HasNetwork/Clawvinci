# Phase 3.0 — Timeline editing core (`clawvinci-timeline`)

Port the editing operations (not the UI) — the mutation layer both the UI
and the MCP agent tools call into, plus the shared undo history. This is
where root `AGENTS.md`'s "Editor mutations and undo" rule lives: "Route UI
and Agent edits through the same domain mutation operations and shared
`EditorUndo` history." That single-source-of-truth requirement is the
whole reason this is its own crate, separate from both Phase 6 (UI) and
Phase 8 (MCP tools) — both of those must call *into* `clawvinci-timeline`,
never re-implement a mutation.

## Source inventory

`Sources/PalmierPro/Editor/` (10,997 LOC across 46 files) plus the two
free-standing mutation engines:

| macOS file(s) | LOC | Rust destination | Notes |
|---|---|---|---|
| `Editor/EditorUndo.swift` | 79 | `clawvinci-timeline::undo` | Wraps AppKit's `UndoManager` with inverse-closure registration + transaction grouping. No AppKit equivalent on Windows — needs a from-scratch undo stack (see below), same *behavior* contract. |
| `Editor/RippleEngine.swift` | 146 | `clawvinci-timeline::ripple` | Ripple insert/delete across tracks, shifting downstream clips. |
| `Editor/OverwriteEngine.swift` | 78 | `clawvinci-timeline::overwrite` | Non-rippling overwrite placement. |
| `Editor/ViewModel/EditorViewModel+ClipMutations.swift` | 807 | `clawvinci-timeline::clip_ops` | Add/move/remove/split/trim/set-properties — the core clip CRUD surface. |
| `Editor/ViewModel/EditorViewModel+Ripple.swift` | 655 | `clawvinci-timeline::ripple` (merge with `RippleEngine`) | |
| `Editor/ViewModel/EditorViewModel+Multicam.swift` | 563 | `clawvinci-timeline::multicam` | Cam switching, sync groups — pairs with `Models/MulticamSource.swift` (Phase 1). |
| `Editor/ViewModel/EditorViewModel+Linking.swift` | 492 | `clawvinci-timeline::linking` | Linked-clip (A/V sync) invariants. |
| `Editor/ViewModel/EditorViewModel+Keyframes.swift` | 461 | `clawvinci-timeline::keyframes` | Keyframe CRUD on top of `Models/Keyframe.swift`'s generic types (Phase 1). |
| `Editor/ViewModel/EditorViewModel+Sync.swift` | 404 | `clawvinci-timeline::sync` | Audio-sync-based alignment (pairs with `Audio/AudioSyncCorrelator.swift`, Phase 9). |
| `Editor/ViewModel/EditorViewModel+SaveAsMedia.swift` | 309 | `clawvinci-timeline::save_as_media` | Depends on Phase 2 encode. |
| `Editor/ViewModel/EditorViewModel+Timelines.swift` | 302 | `clawvinci-timeline::timelines` | Multi-timeline/nested-sequence management. |
| `Editor/ViewModel/EditorViewModel+Folders.swift` | 290 | `clawvinci-timeline::media_library` | Media organization, not clip editing — could also land in a lighter "project" module; decide when porting. |
| `Editor/ViewModel/EditorViewModel+DeadAir.swift` | 289 | `clawvinci-timeline::dead_air` | Silence-based ripple delete — pairs with `Audio/Analysis/SilenceRemovalSettings.swift` (Phase 9). |
| `Editor/ViewModel/EditorViewModel+Nesting.swift` | 223 | `clawvinci-timeline::nesting` | Sequence-in-sequence, uses `Timeline.reachableTimelines` (Phase 1). |
| `Editor/ViewModel/EditorViewModel+Tracks.swift` | 168 | `clawvinci-timeline::tracks` | Track add/remove/reorder/mute/hide. |
| `Editor/ViewModel/EditorViewModel+Clipboard.swift` | 164 | `clawvinci-timeline::clipboard` | Copy/paste clip state. |
| Remaining `EditorViewModel+*.swift` (~15 files) | ~2,000 | mostly Phase 4/6/8/9/11 territory | `+AIEdit`, `+FrameCapture`, `+ProjectSettings`, `+Captions`, `+ChromaKey`, `+AudioEnhance`, `+MediaSwap`, `+Speakers`, `+GeneratedClips`, `+Matte`, `+TimelineMarkers`, `+TimelineRange`, `+PreviewTabs`, `+AITransition`, `+AgentActivity`, `+MediaPanel` — these are thinner adapters over other subsystems (generation, preview, audio, agent). Route each to the crate that owns the underlying subsystem when that phase is reached; don't force them all into `clawvinci-timeline`. |
| `Editor/EditorView.swift`, `EditorWindowController.swift`, `TitleBarView.swift`, `ProjectActivityView.swift`, `Tour/*` | ~1,300 | Phase 6 (UI) | AppKit/SwiftUI — not this phase. |

`EditorViewModel.swift` itself (781 LOC) is the `@MainActor` coordinator
these extensions hang off of — its role (owning current `Timeline` state,
publishing changes, dispatching to the extensions above) becomes
`clawvinci-timeline`'s top-level state/API surface, minus the
`@MainActor`/`@Observable` SwiftUI-specific plumbing (Phase 6's job to
wire observability into the frontend, not this crate's).

## The undo system — no AppKit `UndoManager` on Windows

`EditorUndo.swift` wraps `UndoManager`'s inverse-closure registration
(`registerUndo(withTarget:handler:)`) with transaction grouping so nested
mutation calls coalesce into one undo entry. Windows has no equivalent
system service. Design a Rust undo stack that preserves the *behavior*
`EditorUndo` guarantees, not its mechanism:

- A `Transaction` type: `begin(name) -> TransactionGuard`, and mutations
  push inverse operations (either closures capturing pre-mutation state,
  or command objects with an `apply`/`undo` pair) onto the active
  transaction. Nested calls within one transaction coalesce — matches
  "one coherent user intent should produce one undoable action... nested
  implementation work must coalesce into the outer user action."
- Failed/cancelled/no-op mutations must not push an empty transaction —
  validate before `begin`, matching root `AGENTS.md` exactly.
- Decide `Closure`-based (`Box<dyn FnOnce(&mut Timeline)>`, simple but
  loses replayability/serialization) vs. `Command`-enum-based (each
  mutation is a serializable variant with `apply`/`invert`, more
  boilerplate but replayable, debuggable, and — notably — reusable as the
  wire format for MCP agent-edit descriptions in Phase 8, e.g. the
  `undo` MCP tool and the "mutation delta" concept in
  `Agent/Tools/ToolExecutor+MutationDelta.swift`). **Lean Command-enum**:
  the reuse against Phase 8 is a real, near-term payoff, not a speculative
  one — the source app already has an agent `undo` tool that must exist on
  day one, and 53 MCP tools that each describe "what changed" back to the
  agent.
- Undo/redo correctness bar from root `AGENTS.md`: "must restore exact
  state without cumulative frame rounding, derived-state drift, orphaned
  linked clips, or stale selection." Since Phase 1's model types are
  frame-domain integers already, exact undo is mostly "did you serialize
  the whole affected substructure, not a lossy diff" — test this
  explicitly, not just "undo runs without crashing."

## Definition of done for Phase 3

- `clawvinci-timeline` crate compiles against `clawvinci-model` (Phase 1).
- Core clip ops implemented: add, move, remove, split, trim, set
  properties, ripple insert/delete, overwrite — each going through the
  transaction/undo system, each with an undo-then-redo round-trip test
  asserting exact `Timeline` equality (not just "didn't panic").
- Track ops: add/remove/reorder/mute/hide/rename (with `TrackName`
  validation ported from `Models/Timeline.swift`'s `TrackName` enum).
- One coherent operation from the UI or an agent tool call produces
  exactly one undo entry, verified with a test that performs a
  multi-step mutation (e.g. ripple-delete touching several tracks) and
  checks the undo stack depth.
- No `unwrap()`/`panic!` on invalid mutation input (e.g. out-of-range
  frame, nonexistent clip id) — returns `Result::Err`.
