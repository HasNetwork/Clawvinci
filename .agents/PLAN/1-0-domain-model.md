# Phase 1.0 — Domain model & project file format

Port `Sources/PalmierPro/Models/*.swift` (3,012 LOC, 21 files, zero Apple
framework imports) to the `clawvinci-model` Rust crate. This is the highest-leverage
phase: the project file format and the in-memory timeline representation are
what every other phase (editing, rendering, export, MCP tools) is built on.
Get the shape right here and everything downstream translates cleanly; get
it wrong and every later phase inherits the mistake.

## Why this phase first (after Phase 0)

- It's the cheapest logic to port (plain `Codable`/`Sendable` value types,
  frame-domain integers, no threading or platform APIs) — see the
  architecture scan in `PLAN.md`.
- The MCP tool layer (`Agent/Tools/`, Phase 8) is defined entirely in terms
  of these types — its 53 tool schemas take/return `Timeline`, `Clip`,
  `Track`, etc. Porting the model first means Phase 8's tool signatures are
  mostly mechanical once this exists.
- The `.palmier` project package format (`ProjectFile` → `project.json` +
  media) is what makes projects portable between the macOS and Windows
  apps at all — worth deciding deliberately, not accidentally.

## Source inventory (what's being translated)

| macOS file | LOC | Rust destination | Notes |
|---|---|---|---|
| `Timeline.swift` | 821 | `clawvinci-model::timeline` | `Timeline`, `Track`, `Clip`, `Transform`, `Crop` — the core types. |
| `TextStyle.swift` | 379 | `clawvinci-model::text_style` | Text rendering style, RGBA. |
| `Keyframe.swift` | 346 | `clawvinci-model::keyframe` | Generic `Keyframe<T>`/`KeyframeTrack<T>` — needs a Rust generics story (see below). |
| `MediaAsset.swift` | 338 | `clawvinci-model::media_asset` | Note: this one is `final class` (reference type) in Swift, not a value type — check why before flattening to a Rust struct; likely identity/caching reasons that need a different Rust pattern (`Arc`?). |
| `VideoLayout.swift` | 125 | `clawvinci-model::layout` | |
| `TextLayout.swift` | 98 | `clawvinci-model::text_layout` | |
| `MediaManifest.swift` | 90 | `clawvinci-model::media_manifest` | |
| `ClipEffectKeyframes.swift` | 90 | `clawvinci-model::keyframe` | |
| `MediaResolver.swift` | 89 | `clawvinci-model::media_resolver` | Note: `@unchecked Sendable` in Swift — flags manual thread-safety reasoning; read carefully before porting. |
| `HueCurves.swift` | 80 | `clawvinci-model::grade` | |
| `Matte.swift` | 76 | `clawvinci-model::matte` | |
| `TextAnimation.swift` | 73 | `clawvinci-model::text_animation` | |
| `MulticamSource.swift` | 64 | `clawvinci-model::multicam` | |
| `TimelineMarker.swift` | 61 | `clawvinci-model::timeline` | |
| `Effect.swift` | 59 | `clawvinci-model::effect` | |
| `ClipType.swift` | 54 | `clawvinci-model::clip_type` | |
| `BlendMode.swift` | 51 | `clawvinci-model::blend_mode` | |
| `GradeCurve.swift` | 48 | `clawvinci-model::grade` | |
| `TextFillMode.swift` | 31 | `clawvinci-model::text_style` | |
| `ProjectFile.swift` | 26 | `clawvinci-model::project_file` | Root of `project.json`; has legacy-format fallback decode — preserve that behavior exactly (see below). |
| `MediaFolder.swift` | 13 | `clawvinci-model::media_manifest` | |

Also read (not ported directly, but governs this phase):
`Project/VideoProject.swift` (742 LOC) and `Project/ProjectPackageCoordinator.swift`
(87 LOC) — these define how the `.palmier` package (project.json + media
files on disk) is read/written/locked. Phase 1 ports the data shapes; the
package I/O coordinator itself (async, off-main-thread, serialized per the
"File I/O and project packages" rule in root `AGENTS.md`) is Phase 3/8
territory once there's a Tauri command layer to hang it off — but the
on-disk *format* decision belongs here, now.

## Translation approach

- **Types → `serde`.** Swift `Codable` struct ⇒ Rust `struct` with
  `#[derive(Serialize, Deserialize, Clone, PartialEq)]`. Match field names
  and `CodingKeys` exactly so the JSON shape is unchanged — this is what
  lets a `.palmier` project (or a compatible new format) round-trip.
- **`Identifiable` (`id: String = UUID().uuidString`)** ⇒ a Rust
  `pub id: String` field defaulted via a `fn default_id() -> String` helper
  passed to `#[serde(default = "...")]`. Don't reach for a wrapper `Id<T>`
  newtype unless a real type-confusion bug shows up — Rule 2, no speculative
  abstraction.
- **`Keyframe<Value: Codable & Sendable & Equatable>`** — Swift's
  constrained generic maps to a Rust generic with matching trait bounds
  (`Serialize + DeserializeOwned + Clone + PartialEq`). The
  `KeyframeInterpolatable` protocol (with `Double` and `AnimPair`/`Crop`
  conformances) maps to a Rust trait with the same two impls. Straightforward,
  no design risk here.
- **`enum MediaSource: Codable, Sendable, Equatable`** (associated-value
  enum) ⇒ Rust enum with `#[serde(tag = "...", content = "...")]` or
  untagged, matched to whatever the actual JSON encoding of the Swift enum
  is — **check the real encoded JSON shape** (write a throwaway Swift
  `JSONEncoder` test or read `Codable` synthesis rules) before picking the
  serde representation; don't guess.
- **Legacy decode fallback** (`ProjectFile.decode`: try `ProjectFile`, on
  failure try bare `Timeline` and wrap it) — port this exact two-step
  fallback. This is a compatibility behavior worth preserving even though
  root `AGENTS.md` says "do not preserve backward compatibility" for the
  macOS app's own internal evolution — that rule governs the *macOS app's*
  code, not our obligation to open old project files. State this
  explicitly as an intentional deviation (Rule 7: surface conflicts, pick
  one, explain why).
- **`CGFloat` leakage**: `Track.displayHeight: CGFloat` — the only Apple
  numeric type found in the portable set (3 files flagged in the scan).
  Map to `f64`. Check the other two flagged files for the same before
  starting.

## Project file format decision — resolved

**Byte-identical `.palmier` compatibility** (Decision 9, `PLAN.md`). Same
`project.json` shape, same package layout, as the macOS app — projects must
move between platforms. This constrains every field name/shape to match
Swift's `Codable` output exactly, including the `ProjectFile.decode`
legacy-fallback edge case (bare-`Timeline`-shaped old projects). No
`.palmierwin`-style divergent format — don't introduce one later without
flagging why, per the project's "surface conflicts, don't average them"
rule.

Practical implication: before implementing each type's serde derive,
capture real JSON output from the actual Swift `Codable` encoder (e.g. a
throwaway `JSONEncoder` call in a Swift REPL/test, or output from the
running macOS app) rather than inferring the shape from Swift syntax alone
— `Codable` synthesis has enum/optional-encoding rules that are easy to
get subtly wrong by inspection.

## Definition of done for Phase 1

- `clawvinci-model` crate compiles, with the full type set above.
- Round-trip test: for each type, serialize → deserialize → assert
  equality (Rule 9: tests encode intent — the intent here is "this Rust
  type is a faithful re-encoding of the Swift `Codable` contract," so the
  test fixtures should be JSON *captured from the real Swift app* where
  feasible, not hand-written Rust-side JSON that just confirms Rust's own
  serde round-trips itself).
- `ProjectFile::decode`'s legacy-fallback behavior has an explicit test
  using a bare-`Timeline`-shaped JSON fixture.
- No `unwrap()`/`panic!` in decode paths — malformed project JSON must
  return a `Result::Err`, matching root `AGENTS.md`'s "surface user-requested
  file failures" principle applied to project loading.
