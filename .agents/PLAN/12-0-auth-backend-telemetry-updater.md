# Phase 12.0 — Auth, backend, telemetry, updater

The cross-cutting platform services: authentication, the Convex backend
client, crash/analytics telemetry, and the app auto-updater. Every other
phase that touches the cloud (Phase 10 transcription, Phase 11
generation, credits/billing) depends on this phase's auth/backend client
existing — sequence it earlier than its phase number suggests once
dependent phases actually need it (see note in Phase 10's doc).

## Source inventory

| Area | LOC | Files | Rust destination | Notes |
|---|---|---|---|---|
| `Account/AccountService.swift` | 472 | 1 | `clawvinci-backend::account` | Clerk auth + credit balance/subscription state — the central auth object other services read (`AccountService.shared.convex` is how `TranscriptionBackend` gets its client, per Phase 10). |
| `Account/AccountPopoverCard.swift`, `CreditSummaryView.swift`, `IdentityViews.swift`, `TopOffField.swift` | 633 | 4 | Phase 6 (UI) | — |
| `Backend/BackendConfig.swift`, `BackendError.swift`, `BackendStorage.swift` | 88 | 3 | `clawvinci-backend::config`, `::error`, `::storage` | Small, mostly config/error-type plumbing — cheap to port, read fully before designing the Rust equivalent since 88 LOC total is little enough to just read start to finish. |
| `Telemetry/Analytics.swift`, `Telemetry.swift` | 512 | 2 | `clawvinci-telemetry` | Sentry + PostHog wrappers (`ProductionTelemetry` trait-gated on macOS, matching `Package.swift`). Both have official Rust SDKs (`sentry` crate) and PostHog has a community Rust SDK or a thin REST-call wrapper is simple enough to hand-roll if the SDK isn't mature — check current state when this phase starts. |
| `App/Updater.swift` | (check on read) | 1 | Tauri's `updater` plugin, not a custom crate | Sparkle → Tauri's official updater plugin is a near-direct swap: both do signed-update-manifest + download + install. Reuse `appcast.xml`'s *concept* (a hosted update manifest) but Tauri's updater has its own manifest format — don't try to make Tauri consume Sparkle's `appcast.xml` directly. |
| `App/AppState.swift`, `AppDelegate.swift`, `AppIdentity.swift`, `AppNotifications.swift`, `MainMenu.swift`, `Changelog.swift`, `UpdateBadgeView.swift`, `main.swift` | ~1,200 (App/ total minus Updater) | 7 | Split: `AppState`/`AppIdentity` → `clawvinci-backend` or a small `clawvinci-app` state crate; `AppDelegate`/`MainMenu`/`main.swift` → Phase 0's Tauri entrypoint (`windows/src-tauri/src/`); `AppNotifications`/`UpdateBadgeView`/`Changelog` → Phase 6 (UI) or Tauri's notification plugin. |

## Auth: Clerk has no first-class Rust/native-Windows SDK

`ClerkKit` (`clerk-ios`) and `ClerkConvex` (`clerk-convex-swift`) are the
macOS auth stack — Clerk's official SDKs target web/mobile, not desktop
Rust. This needs a real decision, not an assumed drop-in replacement:

- Clerk does have a documented backend HTTP API and supports custom
  frontends — a Rust client calling Clerk's REST API directly (session
  creation, token verification) is viable but is meaningfully more
  integration work than the Swift SDK gave for free.
- Alternative: embed Clerk's hosted sign-in flow in a Tauri webview
  (system browser or in-app webview for the OAuth/sign-in dance), then
  hand the resulting session token to the Rust backend for subsequent API
  calls — likely the pragmatic choice, similar to how many desktop apps
  handle OAuth-based auth without a native SDK.
- This is a scoping decision to make deliberately when this phase starts,
  informed by whatever Clerk's current (at implementation time) desktop/
  custom-backend support looks like — don't guess now, the situation may
  have changed.

## Convex client

`convex-swift` (`ConvexMobile`) is the backend RPC client (used directly
in `Transcription/TranscriptionBackend.swift`'s `convex.action(...)`/
`convex.subscribe(...)` calls, Phase 10). Convex publishes an official
Rust client (`convex` crate) — confirm it supports the same
action/subscription pattern used here (real-time subscriptions in
particular — check whether the Rust client supports WebSocket-based
live queries or only request/response) before assuming full parity.

## Definition of done for Phase 12

- `clawvinci-backend` crate compiles, provides: an authenticated session
  (however Clerk integration is resolved above), a Convex client capable
  of the action/subscribe pattern Phase 10's transcription depends on,
  and credit/subscription state queries matching `AccountService`'s role.
- `clawvinci-telemetry` crate wired to Sentry (crash reporting) and
  PostHog (analytics) — behind a build-time flag mirroring the
  `ProductionTelemetry` SPM trait (don't send telemetry from dev builds
  by default, matching the macOS app's existing opt-in-by-build-config
  pattern).
- Tauri updater plugin configured and tested with a real signed update
  (install version N, publish version N+1, confirm the running app
  detects and installs it) — this is the kind of thing worth testing for
  real once, not just trusting the plugin's docs.
- Auth end-to-end: sign in, obtain a session, make an authenticated
  backend call (e.g. fetch credit balance) — proven against the real
  Clerk/Convex backend, not mocked.
