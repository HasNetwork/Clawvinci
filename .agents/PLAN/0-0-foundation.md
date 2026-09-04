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
      clawvinci-model/        # Phase 1: domain model + project file format
      clawvinci-media/        # Phase 2: FFmpeg decode/encode wrapper
      clawvinci-timeline/     # Phase 3: editing ops + undo history
      clawvinci-render/       # Phase 4-5: compositor, effects, playback
      clawvinci-export/       # Phase 7: FCPXML/XML export, export queue
      clawvinci-mcp/          # Phase 8: MCP server + tool executor
      clawvinci-audio/        # Phase 9: beat/VAD/speaker/silence
      clawvinci-search/       # Phase 10: transcription, embeddings
      clawvinci-gen/          # Phase 11: generative AI clients
    src/                # Tauri Rust entrypoint, command registration, app state
    tauri.conf.json     # productName: "Clawvinci"
    Cargo.toml           # workspace root
  src/                # Phase 6: web UI (timeline, inspector, panels)
    ...
  package.json
README.md             # points here for Windows build instructions
```

Rationale: one crate per phase keeps dependency direction explicit
(`clawvinci-render` depends on `clawvinci-model` + `clawvinci-media`, never
the reverse) and lets each phase land as an independently buildable,
independently testable unit, per AGENTS.md "grow in layers." Crate prefix
is the product name (Decision 5, `PLAN.md`) — resolved, no longer a
placeholder.

## Platform targets

- **OS floor: Windows 10 22H2.** Not Windows 11-only. Concretely:
  - Tauri v2 on Windows requires the WebView2 runtime — 22H2 ships it
    in-box, but bundle the Evergreen Bootstrapper anyway so a stale/removed
    runtime doesn't break install. Don't assume WebView2 is always present.
  - No Win11-only APIs (e.g. newer DirectStorage/DirectX features gated to
    Win11) without a runtime capability check and a documented fallback.
  - CI/dev-machine testing should include a Windows 10 22H2 target, not
    just whatever the primary dev machine runs.
- **GPU backend: DX12 primary via wgpu, Vulkan fallback.** `clawvinci-render`
  (Phase 4-5) must request wgpu backends in that order and actually
  exercise the fallback path (e.g. via `WGPU_BACKEND=vulkan` in a CI job or
  test), not just leave it theoretically supported by wgpu's backend enum.
  This matters directly because of the Win10 floor — DX12 support on Win10
  varies more by driver/hardware age than on Win11, so Vulkan fallback is
  the actual safety net for older GPUs, not a nice-to-have.

## Toolchain

- Rust: stable channel, edition 2021+. Pin via `rust-toolchain.toml` once
  Phase 1 starts (don't pin speculatively now).
- Node: for the Tauri/web UI side, version pinned via `.nvmrc` when Phase 6
  starts.
- Tauri CLI v2.
- `cargo-nextest` for test running (matches "Verification" rigor in
  `AGENTS.md`'s spirit — fast, isolated test runs).

## CI

Default to GitHub Actions `windows-latest` runners. Switch a given job to
self-hosted only when a concrete need shows up that hosted runners can't
satisfy cheaply — e.g. a real GPU for wgpu/D3D12 integration tests
(Phase 4+), or native library versions (FFmpeg build deps) that are
awkward to provision fresh on every hosted run. Decide per-job when that
need is concrete; don't provision self-hosted infrastructure speculatively
now.

Minimum for Phase 0: `cargo build --workspace` and
`cargo clippy --workspace -- -D warnings` on every push, on
`windows-latest`, once there's a workspace to build.

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
- A minimal Tauri window opens (blank page) on Windows 10 22H2, titled
  "Clawvinci" — proves the shell boots on the actual floor OS before any
  real feature is built on top of it.
- CI runs build + clippy on `windows-latest`.
- `windows/README.md` documents build/run steps (mirrors root
  `scripts/dev.sh`'s role for the macOS side).

Do not start Phase 1 domain-model code until this scaffold builds — per
AGENTS.md "start from the smallest version that works end to end."
