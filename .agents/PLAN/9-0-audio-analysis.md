# Phase 9.0 — Audio analysis (`clawvinci-audio`)

Beat detection, voice activity, speaker identity, silence removal, audio
metering, waveform/envelope extraction, and multi-source sync correlation.
Depends on Phase 2 (audio decode) and feeds Phase 3 (`+Sync`, `+DeadAir`
editor extensions) and Phase 8 (`detect_beats`, `remove_silence` MCP
tools).

## Source inventory

`Sources/PalmierPro/Audio/` (2,120 LOC, 13 files):

| macOS file | LOC | Rust destination | On-device model? |
|---|---|---|---|
| `AudioSyncCorrelator.swift` | 318 | `clawvinci-audio::sync` | No — cross-correlation on decoded audio, pure DSP. |
| `Beats/BeatDetector.swift` | 264 | `clawvinci-audio::beats` | **Yes** — runs the bundled `beat_this` model (see below). |
| `AudioMeterView.swift` | 247 | Phase 6 (UI) | — |
| `Analysis/SpeakerIdentity.swift` | 185 | `clawvinci-audio::speaker` | Yes — `import Speech`/`SpeechVAD`, gated behind the `BundledSpeech` trait on macOS. |
| `Analysis/VoiceActivity.swift` | 163 | `clawvinci-audio::vad` | Yes — same `SpeechVAD` dependency. |
| `Analysis/SpeechMaskStore.swift` | 158 | `clawvinci-audio::speech_mask` | No — storage/caching of VAD output. |
| `Analysis/SilenceRemovalSettings.swift` | 157 | `clawvinci-audio::silence` | No — thresholds/config, pure data + logic. |
| `AudioMeter.swift` | 139 | `clawvinci-audio::meter` | No — real-time level metering, DSP. |
| `Beats/BeatStore.swift` | 131 | `clawvinci-audio::beats` | No — caching of detected beats per media asset. |
| `AudioEnhancer.swift` | 126 | `clawvinci-audio::enhance` | Yes — `import SpeechEnhancement`, denoise. |
| `AudioTrackReader.swift` | 92 | `clawvinci-audio::track_reader` | No — thin wrapper over Phase 2 decode for audio-only reads. |
| `AudioEnvelope.swift` | 78 | `clawvinci-audio::envelope` | No — peak/RMS envelope math. |
| `WaveformExtractor.swift` | 62 | Already covered in Phase 2 (`clawvinci-media`) — decide during Phase 2/9 boundary-setting which crate actually owns this; logically it's "decode audio + compute envelope," which spans both. Don't duplicate it in both crates. |

## The on-device model question (BeatThis, speech models)

Three of the on-device models (`BeatDetector`, `SpeakerIdentity`,
`VoiceActivity`, `AudioEnhancer` — 4 files) depend on either a bundled
CoreML model or Apple's `Speech`/`SpeechVAD`/`SpeechEnhancement`
frameworks, gated behind the macOS `BundledSpeech` SPM trait
(`Package.swift`: "Include on-device speech models and MLX"). Windows has
no equivalent OS-level speech framework or MLX runtime. Two separate
sub-problems, handled differently:

- **Beat detection** (`models/beat_this/`): the repo already documents
  the *build pipeline* — a PyTorch checkpoint (torch.hub) traced with
  `torch.jit`, converted to CoreML via `coremltools`
  (`models/beat_this/convert.py`). **This is directly reusable**: the
  same traced PyTorch model exports to ONNX instead of CoreML (`torch.onnx.export`
  on the same traced module, or `coremltools`' conversion input reused
  with an ONNX target). Read `convert.py` when this phase starts and add
  an ONNX export path alongside (or replacing, for the Windows build) the
  CoreML one — this is a small, well-scoped task, not a research problem.
  Run the resulting ONNX model via `onnxruntime` (Rust bindings), with
  DirectML as the Windows execution provider for GPU accel.
- **Speech VAD / speaker ID / enhancement** (Apple's `Speech`,
  `SpeechVAD`, `SpeechEnhancement` via the `speech-swift` package): these
  are Apple system frameworks/models, not something the repo bundles or
  builds — there is no equivalent artifact to convert. This needs either
  (a) a different open-source model for the same task (VAD: e.g. Silero
  VAD, ONNX-exportable, widely used; speaker ID/diarization: a separate
  model choice; speech enhancement: e.g. a demucs-derived or similar
  model), evaluated for output-quality parity, or (b) explicitly scoping
  this feature out of parity for now. This is a product/scope decision,
  not an engineering one — flag it back to the user when this phase
  starts, don't silently substitute a model without sign-off (matches the
  project's "surface conflicts" rule).

## Definition of done for Phase 9

- `clawvinci-audio` crate compiles against `clawvinci-media` (Phase 2).
- Beat detection: ONNX-exported `beat_this` model runs on Windows,
  producing beat/downbeat frames matching the macOS CoreML output within
  the same 1-frame tolerance the model's own conversion pipeline already
  gates on (`convert.py`'s existing parity check is the reference bar —
  reuse its fixture).
- Silence removal, audio metering, sync correlation, waveform envelope:
  pure-DSP paths, no model dependency, straightforward ports with
  numerical-equivalence tests against known input/output pairs.
- VAD/speaker-ID/enhancement: explicit decision recorded (which
  replacement model, or explicitly deferred) before claiming this phase
  done — "silently missing" is not an acceptable end state per the
  project's "fail loud" rule.
