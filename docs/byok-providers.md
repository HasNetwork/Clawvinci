# BYOK (Bring Your Own Key) & Privacy Guide

Clawvinci adheres to **Decision 10**:
> **Zero telemetry, no accounts, no subscriptions, no cloud proxy backends.**
> All AI capabilities operate strictly via user-provided API keys (BYOK) connecting directly to official provider APIs, or entirely on-device (local DSP and ONNX models).

---

## Configuring Provider API Keys

You can enter your API keys inside Clawvinci by opening **Settings (Gear Icon) -> Models (BYOK)** or by saving them in `%APPDATA%/Clawvinci/settings.json`.

Keys are stored locally on your machine in encrypted/restricted user application data and are **never** transmitted to any middleman server.

### Supported Providers

| Provider | Supported Capabilities | How to Obtain Key |
|---|---|---|
| **OpenAI** | Speech-to-Text (Whisper API), Text & Image Generation | [platform.openai.com/api-keys](https://platform.openai.com/api-keys) |
| **Kling AI** | High-fidelity Video Generation & Re-timing | [klingai.com](https://klingai.com) |
| **Seedance / Bytedance** | Fast AI Video Generation | [seedance.ai](https://seedance.ai) |
| **Fal.ai** | Fast Diffusion & Flux Image Models | [fal.ai/dashboard/keys](https://fal.ai/dashboard/keys) |
| **ElevenLabs** | Voice Synthesis & Dubbing | [elevenlabs.io](https://elevenlabs.io) |
| **Suno** | AI Music & Soundtrack Generation | [suno.com](https://suno.com) |

---

## Local On-Device AI Options

If you prefer complete offline privacy with zero network calls:
- **Speech Transcription**: Set Whisper Mode to `Local`. Clawvinci uses an offline speech energy and Voice Activity Detection (VAD) engine with local ONNX model support.
- **Visual Search**: Visual search uses on-device SigLIP2 vision transformer embeddings stored in the self-contained `.palmier` bundle (`PALMEMB1` vector store).
- **Audio Analysis**: Beat detection, tempo estimation, and silence removal execute 100% locally via deterministic DSP algorithms in `clawvinci-audio`.
