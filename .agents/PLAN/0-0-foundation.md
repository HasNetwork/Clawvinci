# Phase 0.0 — Foundation

Repo layout, toolchain, and build plumbing for the Windows port. Nothing in
this phase touches editing/media logic — it exists so Phase 1+ has
somewhere to land.

## Repo layout decision

The existing repo root is the GPLv3 macOS reference source (`Sources/`,
`Metal/`, `Plugins/`, `models/`, `scripts/`, `mcpb/`, `AGENTS.md`, etc.) and
must stay untouched and buildable as-is — it's the spec we're translating
against, and diffing it against upstream releases later must stay clean.

New Windows-port code goes under a new top-level directory:

```
windows/
  src-tauri/          # Rust: Tauri app shell + core crates (Cargo workspace)
    crates/
      pw-model/        # Phase 1: domain model + project file format
      pw-media/         # Phase 2: FFmpeg decode/encode wrapper
      pw-timeline/       # Phase 3: editing ops + undo history
      pw-render/          # Phase 4-5: compositor, effects, playback
      pw-export/            # Phase 7: FCPXML/XML export, export queue
      pw-mcp/                 # Phase 8: MCP server + tool executor
      pw-audio/                 # Phase 9: beat/VAD/speaker/silence
      pw-search/                  # Phase 10: transcription, embeddings
      pw-gen/                       # Phase 11: generative AI clients
    src/                # Tauri Rust entrypoint, command registration, app state
    tauri.conf.json
    Cargo.toml           # workspace root
  src/                # Phase 6: web UI (timeline, inspector, panels)
    ...
  package.json
README.md             # points here for Windows build instructions
```

Rationale: one crate per phase keeps dependency direction explicit
(`pw-render` depends on `pw-model` + `pw-media`, never the reverse) and lets
each phase land as an independently buildable, independently testable unit,
per AGENTS.md "grow in layers." Crate names picked short; rename before
Phase 1 lands if the branding question (PLAN.md open item) resolves first.

## Toolchain

- Rust: stable channel, edition 2021+. Pin via `rust-toolchain.toml` once
  Phase 1 starts (don't pin speculatively now).
- Node: for the Tauri/web UI side, version pinned via `.nvmrc` when Phase 6
  starts.
- Tauri CLI v2.
- `cargo-nextest` for test running (matches "Verification" rigor in
  `AGENTS.md`'s spirit — fast, isolated test runs).

## CI

GitHub Actions `windows-latest` runner (or self-hosted with a real GPU if
wgpu/D3D12 integration tests need one — decide when Phase 4 needs GPU CI,
not now). Minimum for Phase 0: `cargo build --workspace` and
`cargo clippy --workspace -- -D warnings` on every push, once there's a
workspace to build.

## Licensing carryover

- `windows/` ships under GPLv3 (Decision 4 in `PLAN.md`). Copy the root
  `LICENSE` file into `windows/LICENSE` (or reference the root one — decide
  when the crate boundary is real; a symlink is simplest but Windows
  symlinks need admin/dev-mode, so prefer a plain copy).
- Every new source file that translates or lifts logic from a specific
  macOS source file must carry a header comment naming the source file it's
  derived from, e.g.:
  ```rust
  // Derived from Sources/PalmierPro/Models/Timeline.swift (GPLv3).
  ```
  This is for our own traceability during the port, not a legal requirement
  beyond what GPLv3 already grants — see `RULES.md`.
- Do not carry over the `BINARY_LICENSE.md` terms — those apply only to
  Palmier's proprietary compiled macOS binaries, not to this GPLv3-derived
  work.

## Definition of done for Phase 0

- `windows/src-tauri` is a Cargo workspace that builds (`cargo build
  --workspace`) with the crate skeletons above as empty lib crates (no
  logic yet).
- A minimal Tauri window opens (blank page) on Windows — proves the shell
  boots before any real feature is built on top of it.
- CI runs build + clippy on Windows.
- `windows/README.md` documents build/run steps (mirrors root
  `scripts/dev.sh`'s role for the macOS side).

Do not start Phase 1 domain-model code until this scaffold builds — per
AGENTS.md "start from the smallest version that works end to end."
