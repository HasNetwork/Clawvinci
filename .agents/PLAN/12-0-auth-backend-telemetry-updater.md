# Phase 12.0 — BYOK/local cleanup + updater

**Rewritten under `PLAN.md` Decision 10** (added after Phase 11 shipped):
Clawvinci is not a hosted service. There is no Palmier account, subscription,
credit wallet, or backend to authenticate against, and no product telemetry
to send anywhere. Every AI capability is either **BYOK** (the user supplies
their own API key/endpoint directly to a provider) or **local** (on-device
model, no network call at all). This phase's original scope — Clerk auth,
a Convex/Palmier backend client, Sentry/PostHog telemetry — is cut
entirely, not deferred. Do not build any of it.

What's left is smaller than the phase number suggests: one real rework item
inherited from Phase 10, a smaller verification item on Phase 11, and the
updater (which never depended on accounts in the first place).

## 1. Rework: transcription backend calls Palmier's own infrastructure

`windows/src-tauri/crates/clawvinci-search/src/transcription/backend.rs`
(built in Phase 10, direct port of `TranscriptionBackend.swift`) currently
defaults to:

```rust
endpoint: "https://api.palmier.io".to_string(),
```

Clawvinci has no relationship with that service and must not call it. This
is a real defect against Decision 10, not a hypothetical — fix it as this
phase's first task:

- Replace `TranscriptionBackendConfig` with a BYOK config: user supplies
  their own endpoint + API key (default target: an OpenAI-Whisper-API-
  compatible endpoint, since that's the most common BYOK transcription
  surface — but keep the endpoint field user-editable, don't hardcode a
  new default that's just a different third party standing in for
  Palmier). No endpoint configured = transcription unavailable via this
  path, not a silent fallback to some other hosted default.
- Add a **local on-device path**: a Whisper-class model run via ONNX
  Runtime + DirectML, matching the pattern Phase 9 (`beat_this`) and
  Phase 10 (SigLIP2) already established for on-device ML — except unlike
  those two, there's no existing PyTorch→CoreML conversion script in
  `models/` to retarget, since Apple's `Speech` framework (used for local
  transcription fallback on macOS) has no bundled model to export. This
  means sourcing an actual open Whisper checkpoint (e.g. `whisper.cpp`'s
  supported ONNX exports, or a Hugging Face Whisper checkpoint via
  `optimum`) and integrating it fresh — budget real time for this, it's
  not a straight port.
- User picks BYOK vs. local per-project or in settings (Phase 13); both
  paths feed the same `TranscriptionResult`/`TranscriptCache` types Phase
  10 already built — this is a backend swap, not a data-model change.
- `get_transcript`/`remove_words`/`remove_silence` MCP tools (Phase 8) are
  unaffected — they consume `TranscriptCache`, not the backend directly.

## 2. Verify: generation backend is BYOK-direct, not Palmier-proxied

`clawvinci-gen::backend::client::GenerationBackendClient` (Phase 11) is
already a trait, not a hardcoded client — good, it doesn't have Phase 10's
problem of a baked-in Palmier URL. Confirm before closing this phase:

- The concrete implementation(s) behind that trait call each provider
  (Seedance, Kling, Nano Banana Pro, etc.) **directly** with a
  user-supplied API key for that specific provider — not through any
  single unified "backend" that would imply a middleman service.
- `CostEstimator`/`credits_per_*` fields (`catalog/cost.rs`) stay — they're
  informational cost *estimates* shown to the user before they spend their
  own provider credits, not a wallet balance. No change needed there.
- If the current implementation assumes one shared backend for all
  providers, split it into one BYOK client per provider, matching how
  `catalog/models.rs` already models providers as distinct entries.

## 3. Cut entirely — do not implement

- Clerk auth / any sign-in flow / session tokens.
- Convex client, or any other Palmier-operated backend service.
- Credit balance, subscription state, account settings.
- Sentry crash reporting, PostHog analytics, or any other telemetry —
  nothing phones home.
- `Settings/AccountPane`-equivalent UI (Phase 13's settings polish should
  drop this pane, not stub it).

## 4. Keep: updater

Sparkle → Tauri's official `updater` plugin, as originally planned — this
never depended on accounts. Signed update manifest, download, install.
Reuse `appcast.xml`'s *concept* (a hosted manifest), not its format —
Tauri's updater has its own manifest shape.

## Definition of done for Phase 12

- `grep`-clean: zero references to `api.palmier.io`, `convex`, `clerk`,
  `sentry`, `posthog` (case-insensitive) anywhere under `windows/`.
- Transcription: both BYOK (real call against a user-configured
  Whisper-API-compatible endpoint) and local (real ONNX Whisper-class
  model run) paths verified against real audio, not just type-level
  plumbing.
- Generation: confirmed (or reworked) to call provider APIs directly with
  per-provider BYOK keys — no shared backend in between.
- Tauri updater plugin configured and tested with a real signed update
  (install version N, publish version N+1, confirm the running app
  detects and installs it).
- No account/auth/telemetry code exists anywhere in the `windows/` tree.
