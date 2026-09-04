# Phase 8.0 — MCP agent layer (`clawvinci-mcp`)

Ports the HTTP MCP server and the 53-tool executor — the product's stated
identity ("The video editor built for AI"). This is `Agent/` minus the
chat UI: `Agent/Tools/` (16,684 LOC total for all of `Agent/`, tools
specifically detailed below), `Agent/MCP/`, and `Agent/Clients/`
(LLM provider clients for the in-app agent, distinct from the MCP *server*
that external agents connect to — don't conflate the two).

## Two distinct things named "agent" here — keep them separate

1. **MCP server** (`Agent/MCP/MCPHTTPServer.swift`, 312 LOC): exposes
   `http://127.0.0.1:19789/mcp` so *external* agents (Claude Code, Codex,
   Cursor, Claude Desktop) can drive the editor. This is what
   `Agent/Tools/` implements the tool surface for.
2. **In-app agent chat** (`Agent/Chat/`, `Agent/Clients/`, `Agent/Panel/`):
   a chat panel *inside* the app that itself calls LLM providers directly
   (`Clients/AnthropicProvider.swift`, `OpenAIProvider.swift`,
   `BYOKClient.swift`, `PalmierClient.swift`) and uses the same tool
   executor to act on the timeline. `Agent/Panel/` is UI (Phase 6).
   `Agent/Chat/` (`AgentService.swift`, `ChatSessionStore.swift`,
   `AgentStreamPresentationBuffer.swift`, `AgentMentionContext.swift`) is
   the orchestration logic — portable, belongs in this phase.

Both consume the same `ToolExecutor` — this phase's core deliverable is
that executor, callable from both the MCP HTTP server and the in-app chat
orchestrator.

## Source inventory: `Agent/Tools/` (53 tools, 27 files, fully portable —
zero Apple-framework imports per the architecture scan)

`ToolDefinitions.swift` (the `ToolName` enum, 53 cases) groups into:

| Group | Tool count (approx) | Backing `ToolExecutor+*.swift` | Rust destination |
|---|---|---|---|
| Projects/Timelines | 9 (`manage_project`, `get_timeline`, `inspect_timeline`, `create_timeline`, `set_active_timeline`, `manage_markers`, `set_project_settings`, `export_project`, `manage_exports`) | `+Projects`, `+Timeline`, `+InspectTimeline`, `+Markers`, `+ProjectSettings`, `+Export` | `clawvinci-mcp::tools::project`, `::timeline` |
| Media library | 6 (`get_media`, `inspect_media`, `search_media`, `import_media`, `capture_frame`, `organize_media`) | `+Media`, `+Search`, `+Import`, `+CaptureFrame`, `+Organize` | `clawvinci-mcp::tools::media` |
| Clips | 14 (`manage_tracks`, `manage_clip_links`, `add_clips`, `insert_clips`, `move_clips`, `remove_clips`, `split_clips`, `ripple_delete_ranges`, `swap_clip_media`, `set_clip_properties`, `copy_clip_settings`, `set_keyframes`, `apply_layout`, `sync_clips`, `undo`) | `+Clips`, `+Layout`, `+Sync` | `clawvinci-mcp::tools::clips` — thin wrappers calling straight into Phase 3's `clawvinci-timeline` ops. |
| Multicam | 3 (`manage_multicam`, `change_cam`, `get_multicam`) | `+Multicam` | `clawvinci-mcp::tools::multicam` |
| Transcript | 4 (`get_transcript`, `remove_words`, `remove_silence`, `detect_beats`) | `+Transcription`, `+Words`, `+Beats` | `clawvinci-mcp::tools::transcript` — calls into Phase 9 (audio) and Phase 10 (transcription). |
| Text & captions | 3 (`add_texts`, `update_text`, `add_captions`) | `+Texts`, `+Captions` | `clawvinci-mcp::tools::text` |
| Color & effects | remaining tools (`apply_color`, `applyEffect`-family — check `ToolDefinitions.swift`'s tail for the exact list, cut off in the initial scan) | `+Color`, `+Effect`, `+Denoise` | `clawvinci-mcp::tools::effects` — calls into Phase 5. |
| Generation | — | `+Generate` | `clawvinci-mcp::tools::generate` — calls into Phase 11. |
| Feedback/activity/internal | — | `+Feedback`, `+AgentActivity`, `+MutationDelta`, `+ShortId`, `+Skills` | `clawvinci-mcp::support` |

`ToolExecutor.swift` (the dispatcher) and `ToolResult.swift` (response
envelope) are the two files to read first when this phase starts — they
define the calling convention every tool follows. `AgentInstructions.swift`
is the system-prompt/instructions text given to agents — port the content
verbatim (it's product-defined behavior, not implementation), adapting
only environment-specific details (paths, port number if changed).

## MCP protocol layer

`swift-sdk` (the `modelcontextprotocol/swift-sdk` package) implements the
MCP protocol on the Swift side. The Rust ecosystem has an official-adjacent
MCP SDK (check current state when this phase starts — the ecosystem moves
fast; don't assume a specific crate name now). `MCPHTTPServer.swift` (312
LOC) wires that SDK to an HTTP listener on port 19789 and registers the 53
tools; `MCPService.swift` (139 LOC) and `MCPClientInfo.swift` (44 LOC) are
supporting types. Port the *protocol behavior* (tool schemas, the HTTP
transport, the port number — `README.md` documents
`http://127.0.0.1:19789/mcp` as a stable integration point external tools
already point at, so don't change it without a strong reason) using
whatever the best-maintained Rust MCP crate is at implementation time.

## Definition of done for Phase 8

- `clawvinci-mcp` crate compiles, exposing an HTTP MCP server on
  `127.0.0.1:19789/mcp` with all 53 tools registered and schema-correct
  (verify against the real tool schemas the Swift `ToolDefinitions.swift`
  + each `ToolExecutor+*.swift` file define — the JSON schema for each
  tool's arguments is part of the MCP contract external agents rely on,
  so it must match, not just "have a plausible shape").
- Connect a real MCP client (Claude Code, per `README.md`'s own
  documented setup: `claude mcp add --transport http clawvinci
  http://127.0.0.1:19789/mcp`) against a running instance and drive a
  basic edit end-to-end (e.g. `add_clips` + `split_clips` + `undo`) —
  this is the actual acceptance test, not a unit test in isolation.
- Every tool that mutates the timeline goes through `clawvinci-timeline`'s
  undo system (Phase 3) — an agent edit and a UI edit of the same
  operation must be indistinguishable in the undo stack, per root
  `AGENTS.md`'s "Route UI and Agent edits through the same domain
  mutation operations and shared undo history."
- `undo` tool works correctly against agent-originated edits.
- In-app chat orchestration (`AgentService`-equivalent) works against at
  least one LLM provider (Anthropic, matching `AnthropicProvider.swift`)
  end-to-end, calling the same tool executor as the MCP server.
