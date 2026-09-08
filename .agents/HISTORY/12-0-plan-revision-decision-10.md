# Phase 12.0 — Plan revision: Decision 10 (no accounts/backend/telemetry)

**Status**: 📋 Planning revision only — no code changed in this entry
**Commit**: `c4d9c75` (docs-only)
**Date**: 2026-09-08

## What happened

User decision, made after Phase 11 shipped (all of Phases 0–11 complete,
CI-green): Clawvinci is not a hosted service and must not have an account/
backend/telemetry layer. Every AI capability is BYOK (user's own key,
direct to the provider) or fully local/on-device. Recorded as `PLAN.md`
Decision 10.

This is not a purely forward-looking scope change — grepped the already-
shipped `windows/` tree to check for impact before writing the plan
revision, per RULES.md's "read the actual source, don't infer" discipline:

- **Real defect found**: `windows/src-tauri/crates/clawvinci-search/src/transcription/backend.rs:41`
  (Phase 10, `TranscriptionBackend::default()`) hardcodes
  `endpoint: "https://api.palmier.io"` — a Palmier-operated endpoint
  Clawvinci has no right to call. Confirmed by reading the file directly,
  not inferred from the phase doc.
- **Checked, found clean**: `clawvinci-mcp/src/chat/` only has `anthropic.rs`
  — no `PalmierClient`-equivalent was ever built in Phase 8, despite the
  original Phase 8 doc listing 4 planned providers (Anthropic/OpenAI/BYOK/
  Palmier). Nothing to undo there.
- **Checked, needs verification not rework**: `clawvinci-gen::backend::client::GenerationBackendClient`
  (Phase 11) is trait-based and does not hardcode a Palmier URL
  (`base_url` is a constructor parameter; test/mock URLs are
  `mock.clawvinci.com`). `credits_per_*` fields in `catalog/cost.rs` are
  informational cost estimates, not a wallet — left as-is. What still
  needs checking when Phase 12 starts: does the concrete implementation
  behind that trait call each provider (Seedance/Kling/Nano Banana Pro)
  directly with a per-provider BYOK key, or through one shared backend?

## What changed (docs only)

- `PLAN.md` — added Decision 10; added a "Revision after Phase 11 shipped"
  section documenting the grep findings above.
- `PLAN/12-0-auth-backend-telemetry-updater.md` — fully rewritten. Old
  scope (Clerk auth, Convex backend, Sentry/PostHog) cut. New scope: (1)
  fix the transcription backend — BYOK config + new local on-device
  Whisper-class path (no existing conversion script to retarget, unlike
  `beat_this`/SigLIP2 — this is fresh integration work), (2) verify/rework
  the generation backend as BYOK-direct-to-provider, (3) keep the Tauri
  updater (never depended on accounts).
- `PLAN/10-0-search-transcription-ml.md` — added a revision note pointing
  at Phase 12 for the actual fix, so the doc doesn't read as if
  `backend.rs` still matches what's described there.
- `RULES.md` — added a standing rule: no accounts/Palmier-backend/
  telemetry; fix violations found in already-shipped code rather than
  extend them.
- `Handover.md` — flagged at the top and in the phase-status table, so
  whichever agent picks up Phase 12 sees this before starting.

## For the agent picking up Phase 12

Read `PLAN/12-0-auth-backend-telemetry-updater.md` in full before writing
any code — it has the exact rework list. The transcription backend fix is
section 1 and is the first concrete task (real defect, not speculative).
