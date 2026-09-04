# Phase 13.0 — Polish: localization, settings, home/onboarding, help

The last phase by design — everything here either wraps a subsystem that
must already exist (Settings panes configure things Phases 1-12 built) or
is genuinely low-risk, conventional UI work. Nothing in this phase blocks
anything else; nothing else blocks starting this phase once its target
subsystem exists, so pieces of it can land opportunistically alongside
later Phase 6-12 work rather than strictly waiting for Phase 12 to close
first.

## Source inventory

| Area | LOC | Files | Notes |
|---|---|---|---|
| `Localization/` | 216 | 3 | `AppLanguage.swift`, `AppLocalization.swift`, `UILocalization.swift` — small. `README.md` lists 15 supported languages (`docs/readme/README.*.md`); `Package.swift`'s `defaultLocalization: "en"` and `.process("Resources/Localization")` show string-catalog-based localization. Web i18n equivalent: a standard JSON-per-locale catalog (e.g. `i18next` or similar) — port the *string keys and translated values*, not the Swift string-catalog mechanism. Check `Sources/PalmierPro/Resources/Localization` for the actual catalog format when this phase starts; extracting existing translated strings (rather than re-translating from scratch) is real, reusable value here. |
| `Settings/` | 2,818 | 15 | `SettingsView.swift` + panes: `AccountPane`, `AgentPane`, `AppearancePane`, `GeneralPane`, `ModelsPane`, `NotificationsPane`, `PrivacyPane`, `StoragePane`, `TimelineColorsPane`, plus `Settings/Skill/*` (5 files: skill management UI — pairs with `Agent/Skills/` from Phase 8, which is portable logic already covered there; this is just the UI). Each pane is a conventional settings-form UI over state owned by the phase that pane configures (Account → Phase 12, Agent/Models → Phase 8/11, Appearance/TimelineColors → Phase 6's design tokens, Storage → Phase 2/project package). |
| `Home/` | 1,461 | 9 | `HomeView.swift`, `MyProjectsSection.swift`, `ProjectCard.swift`, `SampleProjectsStrip.swift`, `UpdateOverlay.swift`, `Onboarding/*` (4 files: `OnboardingModels`, `OnboardingOverlay`, `OnboardingSteps`, `OnboardingStore`) — the landing/project-picker screen and first-run onboarding flow. `Project/SampleProjectService.swift` (173 LOC, from the earlier `Project/` scan) backs `SampleProjectsStrip` — check whether sample project assets are bundled or downloaded before assuming this ports trivially. |
| `Help/` | 975 | 5 | `HelpView.swift`, `ShortcutsPane.swift`, `FeedbackView.swift` + `FeedbackScreenshot.swift`, `MCPInstructionsPane.swift`. `MCPInstructionsPane.swift` (322 LOC) is worth calling out specifically: it's the in-app UI documented in `README.md` as `Help → MCP Instructions → Install in Cursor/Claude Desktop` — this needs Windows-appropriate install instructions (different config file paths, the Claude Desktop `.mcpb` extension install flow) once Phase 8's MCP server naming/port is finalized. |
| `Editor/Tour/` | (small, from Phase 3's scan: `EditorSplitViewController+Tour.swift`, `TourController.swift`, `TourOverlay.swift`) | 3 | Guided first-use tour overlay — depends on Phase 6's timeline/panel UI existing, standard product-tour UI pattern. |

## Definition of done for Phase 13

- Localization catalog ported with at least the same language set as the
  macOS app (or an explicit, flagged decision to launch with a subset and
  add the rest later — don't silently ship English-only if the macOS app
  supports 15 languages, without recording that as a deliberate scope
  cut).
- Every Settings pane wired to its real backing state (not stub UI) —
  each pane's "done" is really a checkpoint that its owning phase's Rust
  API surface is complete enough to configure end-to-end.
- Home screen: project list, create/open project, sample projects (if
  bundled/downloadable assets are confirmed available), first-run
  onboarding flow.
- Help: shortcuts reference, feedback submission, and MCP setup
  instructions accurate for Windows (correct config paths for Claude
  Code/Codex/Cursor/Claude Desktop on Windows, not copy-pasted macOS
  paths).
- `mcpb/` (the bundled Claude Desktop extension, `manifest.json` +
  `server`) — check whether the `.mcpb` format itself is
  platform-neutral (it's a zip-based extension format, likely portable
  as-is with just the server binary swapped for a Windows build) before
  assuming this needs a rewrite; may be nearly free once Phase 8's MCP
  server exists as a Windows binary.
