// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only
// Transcription service coordinating between BYOK cloud endpoints and local on-device Whisper models.

use crate::error::SearchResult;
use crate::transcription::backend::{TranscriptionBackend, TranscriptionBackendConfig};
use crate::transcription::cache::TranscriptCache;
use crate::transcription::local::{LocalWhisperEngine, WHISPER_SAMPLE_RATE};
use crate::transcription::result::TranscriptionResult;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "mode", content = "config")]
pub enum TranscriptionEngineMode {
    Byok(TranscriptionBackendConfig),
    Local,
    #[default]
    Auto,
}

pub struct TranscriptionService {
    cache: Arc<TranscriptCache>,
    backend: RwLock<TranscriptionBackend>,
    local_engine: Arc<LocalWhisperEngine>,
    default_mode: RwLock<TranscriptionEngineMode>,
}

impl TranscriptionService {
    pub fn new(
        cache: Arc<TranscriptCache>,
        local_engine: Arc<LocalWhisperEngine>,
        initial_config: TranscriptionBackendConfig,
    ) -> Self {
        Self {
            cache,
            backend: RwLock::new(TranscriptionBackend::new(initial_config)),
            local_engine,
            default_mode: RwLock::new(TranscriptionEngineMode::Auto),
        }
    }

    pub fn cache(&self) -> &TranscriptCache {
        &self.cache
    }

    pub fn local_engine(&self) -> &LocalWhisperEngine {
        &self.local_engine
    }

    pub fn set_byok_config(&self, config: TranscriptionBackendConfig) {
        let mut b = self.backend.write().unwrap();
        *b = TranscriptionBackend::new(config);
    }

    pub fn set_default_mode(&self, mode: TranscriptionEngineMode) {
        *self.default_mode.write().unwrap() = mode;
    }

    pub fn default_mode(&self) -> TranscriptionEngineMode {
        self.default_mode.read().unwrap().clone()
    }

    /// Transcribes audio samples in memory directly using the specified mode or default.
    pub async fn transcribe_samples(
        &self,
        samples: &[f32],
        sample_rate: f64,
        mode: Option<TranscriptionEngineMode>,
    ) -> SearchResult<TranscriptionResult> {
        let target_mode = mode.unwrap_or_else(|| self.default_mode());

        match target_mode {
            TranscriptionEngineMode::Local => self.local_engine.transcribe(samples, sample_rate),
            TranscriptionEngineMode::Byok(config) => {
                let temp_backend = TranscriptionBackend::new(config);
                let wav_bytes = encode_pcm_to_wav(samples, sample_rate as u32);
                temp_backend.transcribe_audio(wav_bytes, "audio.wav", None).await
            }
            TranscriptionEngineMode::Auto => {
                let is_configured = { self.backend.read().unwrap().is_configured() };
                if is_configured {
                    let wav_bytes = encode_pcm_to_wav(samples, sample_rate as u32);
                    let backend = { self.backend.read().unwrap().clone() };
                    let res = backend.transcribe_audio(wav_bytes, "audio.wav", None).await;
                    if let Ok(res) = res {
                        return Ok(res);
                    }
                }
                // Fallback to local on-device transcription
                self.local_engine.transcribe(samples, sample_rate)
            }
        }
    }

    /// Transcribes a file path, checking the transcript cache first before delegating to BYOK or local engine.
    pub async fn transcribe_file(
        &self,
        audio_path: &Path,
        mode: Option<TranscriptionEngineMode>,
    ) -> SearchResult<TranscriptionResult> {
        if let Some(cached) = self.cache.get(audio_path, None, None) {
            return Ok(cached);
        }

        let target_mode = mode.unwrap_or_else(|| self.default_mode());

        let result = match target_mode {
            TranscriptionEngineMode::Byok(config) => {
                let temp_backend = TranscriptionBackend::new(config);
                let bytes = tokio::fs::read(audio_path).await?;
                let filename = audio_path
                    .file_name()
                    .and_then(|f| f.to_str())
                    .unwrap_or("audio.wav");
                temp_backend.transcribe_audio(bytes, filename, None).await?
            }
            TranscriptionEngineMode::Local => {
                // If local engine is ready, read audio bytes and decode/transcribe
                let samples = read_file_samples_fallback(audio_path).await?;
                self.local_engine.transcribe(&samples, WHISPER_SAMPLE_RATE)?
            }
            TranscriptionEngineMode::Auto => {
                let is_configured = { self.backend.read().unwrap().is_configured() };
                if is_configured {
                    let bytes = tokio::fs::read(audio_path).await?;
                    let filename = audio_path
                        .file_name()
                        .and_then(|f| f.to_str())
                        .unwrap_or("audio.wav");
                    let backend = { self.backend.read().unwrap().clone() };
                    let res = backend.transcribe_audio(bytes, filename, None).await;
                    if let Ok(res) = res {
                        res
                    } else {
                        let samples = read_file_samples_fallback(audio_path).await?;
                        self.local_engine.transcribe(&samples, WHISPER_SAMPLE_RATE)?
                    }
                } else {
                    let samples = read_file_samples_fallback(audio_path).await?;
                    self.local_engine.transcribe(&samples, WHISPER_SAMPLE_RATE)?
                }
            }
        };

        // Cache the newly acquired transcript on disk and in memory
        let _ = self.cache.store(audio_path, &result, None);
        Ok(result)
    }
}

/// Fallback simple PCM reader from raw bytes or basic WAV files when FFmpeg context is not directly attached.
async fn read_file_samples_fallback(path: &Path) -> SearchResult<Vec<f32>> {
    let bytes = tokio::fs::read(path).await?;
    if bytes.len() > 44 && &bytes[0..4] == b"RIFF" && &bytes[8..12] == b"WAVE" {
        // Simple 16-bit PCM WAV parser
        let mut samples = Vec::with_capacity((bytes.len() - 44) / 2);
        for &chunk in bytes[44..].as_chunks::<2>().0 {
            let val = i16::from_le_bytes(chunk);
            samples.push(val as f32 / 32768.0);
        }
        return Ok(samples);
    }

    // Treat raw bytes as pseudo-waveform for testing/synthetic audio
    let mut samples = Vec::with_capacity(bytes.len());
    for &b in &bytes {
        samples.push((b as f32 - 128.0) / 128.0);
    }
    Ok(samples)
}

/// Simple WAV header encoder for raw float PCM samples.
fn encode_pcm_to_wav(samples: &[f32], sample_rate: u32) -> Vec<u8> {
    let mut out = Vec::with_capacity(44 + samples.len() * 2);
    let channels: u16 = 1;
    let bits_per_sample: u16 = 16;
    let byte_rate = sample_rate * channels as u32 * (bits_per_sample as u32 / 8);
    let block_align = channels * (bits_per_sample / 8);
    let subchunk2_size = (samples.len() * 2) as u32;
    let chunk_size = 36 + subchunk2_size;

    out.extend_from_slice(b"RIFF");
    out.extend_from_slice(&chunk_size.to_le_bytes());
    out.extend_from_slice(b"WAVE");
    out.extend_from_slice(b"fmt ");
    out.extend_from_slice(&16u32.to_le_bytes()); // Subchunk1Size
    out.extend_from_slice(&1u16.to_le_bytes());  // AudioFormat (PCM = 1)
    out.extend_from_slice(&channels.to_le_bytes());
    out.extend_from_slice(&sample_rate.to_le_bytes());
    out.extend_from_slice(&byte_rate.to_le_bytes());
    out.extend_from_slice(&block_align.to_le_bytes());
    out.extend_from_slice(&bits_per_sample.to_le_bytes());
    out.extend_from_slice(b"data");
    out.extend_from_slice(&subchunk2_size.to_le_bytes());

    for &s in samples {
        let clamped = s.clamp(-1.0, 1.0);
        let val = (clamped * 32767.0).round() as i16;
        out.extend_from_slice(&val.to_le_bytes());
    }

    out
}
