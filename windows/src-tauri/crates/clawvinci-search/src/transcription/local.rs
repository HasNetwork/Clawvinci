// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only
// Local on-device Whisper-class transcription engine.

use crate::error::{SearchError, SearchResult};
use crate::model_download::{download_and_verify, ModelFileDownload};
use crate::transcription::result::{TranscriptionResult, TranscriptionSegment, TranscriptionWord};
use crate::visual::loader::LoaderState;
use serde::{Deserialize, Serialize};
use std::f32::consts::PI;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use tracing::{error, info};
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

pub const WHISPER_SAMPLE_RATE: f64 = 16000.0;
pub const WHISPER_N_MELS: usize = 80;
pub const WHISPER_HOP_LENGTH: usize = 160; // 10ms at 16kHz
pub const WHISPER_N_FFT: usize = 400; // 25ms at 16kHz

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WhisperModelSpec {
    pub name: String,
    pub version: i32,
    pub sample_rate: u32,
    pub n_mels: usize,
    pub is_multilingual: bool,
}

impl Default for WhisperModelSpec {
    fn default() -> Self {
        Self {
            name: "whisper-tiny.en".to_string(),
            version: 1,
            sample_rate: 16000,
            n_mels: 80,
            is_multilingual: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhisperModelFileSpec {
    pub name: String,
    pub sha256: String,
    pub bytes: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhisperModelManifest {
    pub model: String,
    pub model_file: WhisperModelFileSpec,
}

impl Default for WhisperModelManifest {
    fn default() -> Self {
        Self {
            model: "whisper-tiny.en".to_string(),
            model_file: WhisperModelFileSpec {
                name: "ggml-tiny.en.bin".to_string(),
                sha256: "921e4cf8b289a9a4c072e1b1e2c411e1ae2e3535a5b91ee48490e668d043b454"
                    .to_string(),
                bytes: 77_691_713,
                url: Some(
                    "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-tiny.en.bin"
                        .to_string(),
                ),
            },
        }
    }
}

/// Mel-spectrogram feature extractor for Whisper models.
pub struct WhisperAudioPreprocessor;

impl WhisperAudioPreprocessor {
    /// Resamples or adjusts mono PCM audio to 16,000 Hz.
    pub fn resample_to_16k(samples: &[f32], sample_rate: f64) -> Vec<f32> {
        if (sample_rate - WHISPER_SAMPLE_RATE).abs() < 1.0 || samples.is_empty() {
            return samples.to_vec();
        }

        let ratio = WHISPER_SAMPLE_RATE / sample_rate;
        let new_len = (samples.len() as f64 * ratio).round() as usize;
        let mut out = Vec::with_capacity(new_len);

        for i in 0..new_len {
            let src_idx = i as f64 / ratio;
            let idx0 = (src_idx.floor() as usize).min(samples.len() - 1);
            let idx1 = (idx0 + 1).min(samples.len() - 1);
            let frac = (src_idx - idx0 as f64) as f32;
            let val = samples[idx0] * (1.0 - frac) + samples[idx1] * frac;
            out.push(val);
        }

        out
    }

    /// Computes 80-channel log Mel-filterbank energies from 16kHz audio samples.
    pub fn log_mel_spectrogram(samples: &[f32]) -> Vec<Vec<f32>> {
        if samples.len() < WHISPER_N_FFT {
            return vec![vec![0.0; WHISPER_N_MELS]];
        }

        let n_frames = (samples.len() - WHISPER_N_FFT) / WHISPER_HOP_LENGTH + 1;
        let mut mel_spectrogram = Vec::with_capacity(n_frames);

        // Precompute Hann window
        let mut window = vec![0.0f32; WHISPER_N_FFT];
        for (i, w) in window.iter_mut().enumerate() {
            *w = 0.5 * (1.0 - (2.0 * PI * i as f32 / WHISPER_N_FFT as f32).cos());
        }

        for frame_idx in 0..n_frames {
            let start = frame_idx * WHISPER_HOP_LENGTH;
            let frame = &samples[start..start + WHISPER_N_FFT];

            // Windowed power spectrum approximation
            let mut mel_energies = vec![0.0f32; WHISPER_N_MELS];
            for (m, energy) in mel_energies.iter_mut().enumerate() {
                let bin_start = (m * WHISPER_N_FFT) / (WHISPER_N_MELS * 2);
                let bin_end = ((m + 2) * WHISPER_N_FFT) / (WHISPER_N_MELS * 2);
                let mut sum = 0.0f32;
                for k in bin_start..bin_end.min(WHISPER_N_FFT) {
                    let s = frame[k] * window[k];
                    sum += s * s;
                }
                *energy = (sum + 1e-6).log10();
            }

            mel_spectrogram.push(mel_energies);
        }

        mel_spectrogram
    }
}

pub trait LocalWhisperTranscriber: Send + Sync {
    fn spec(&self) -> &WhisperModelSpec;
    fn transcribe(&self, samples: &[f32], sample_rate: f64) -> SearchResult<TranscriptionResult>;
}

// ---------------------------------------------------------------------------
// Real whisper-rs transcriber using GGML models
// ---------------------------------------------------------------------------

pub struct WhisperRsTranscriber {
    spec: WhisperModelSpec,
    ctx: WhisperContext,
}

impl WhisperRsTranscriber {
    pub fn load(spec: WhisperModelSpec, model_path: &Path) -> SearchResult<Self> {
        let path_str = model_path
            .to_str()
            .ok_or_else(|| SearchError::ModelNotReady("Invalid model path encoding".into()))?;

        let ctx = WhisperContext::new_with_params(path_str, WhisperContextParameters::default())
            .map_err(|e| {
                SearchError::LocalTranscriptionFailed(format!("Failed to load Whisper model: {e}"))
            })?;

        Ok(Self { spec, ctx })
    }
}

impl LocalWhisperTranscriber for WhisperRsTranscriber {
    fn spec(&self) -> &WhisperModelSpec {
        &self.spec
    }

    fn transcribe(&self, samples: &[f32], sample_rate: f64) -> SearchResult<TranscriptionResult> {
        if samples.is_empty() || sample_rate <= 0.0 {
            return Ok(TranscriptionResult::new(
                "",
                Some("en".to_string()),
                vec![],
                vec![],
            ));
        }

        let resampled = WhisperAudioPreprocessor::resample_to_16k(samples, sample_rate);

        let mut state = self.ctx.create_state().map_err(|e| {
            SearchError::LocalTranscriptionFailed(format!("Failed to create whisper state: {e}"))
        })?;

        let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
        params.set_language(Some("en"));
        params.set_print_special(false);
        params.set_print_progress(false);
        params.set_print_realtime(false);
        params.set_print_timestamps(false);
        params.set_token_timestamps(true);

        state.full(params, &resampled).map_err(|e| {
            SearchError::LocalTranscriptionFailed(format!("Whisper inference failed: {e}"))
        })?;

        let n_segments = state.full_n_segments().map_err(|e| {
            SearchError::LocalTranscriptionFailed(format!("Failed to get segments: {e}"))
        })?;

        let mut words = Vec::new();
        let mut segments = Vec::new();
        let mut full_text_parts = Vec::new();

        for i in 0..n_segments {
            let text = state.full_get_segment_text_lossy(i).map_err(|e| {
                SearchError::LocalTranscriptionFailed(format!("Failed to get segment text: {e}"))
            })?;
            let t0 = state.full_get_segment_t0(i).map_err(|e| {
                SearchError::LocalTranscriptionFailed(format!("Failed to get t0: {e}"))
            })? as f64
                / 100.0;
            let t1 = state.full_get_segment_t1(i).map_err(|e| {
                SearchError::LocalTranscriptionFailed(format!("Failed to get t1: {e}"))
            })? as f64
                / 100.0;

            let text = text.trim().to_string();
            if text.is_empty() {
                continue;
            }

            let n_tokens = state.full_n_tokens(i).map_err(|e| {
                SearchError::LocalTranscriptionFailed(format!("Failed to get token count: {e}"))
            })?;

            for j in 0..n_tokens {
                let token_text = state
                    .full_get_token_text(i, j)
                    .unwrap_or_default()
                    .trim()
                    .to_string();
                if token_text.is_empty() || token_text.starts_with('[') {
                    continue;
                }

                let token_data = state.full_get_token_data(i, j).map_err(|e| {
                    SearchError::LocalTranscriptionFailed(format!(
                        "Failed to get token data: {e}"
                    ))
                })?;

                words.push(TranscriptionWord::new(
                    &token_text,
                    Some(token_data.t0 as f64 / 100.0),
                    Some(token_data.t1 as f64 / 100.0),
                ));
            }

            segments.push(TranscriptionSegment::new(&text, t0, t1));
            full_text_parts.push(text);
        }

        let full_text = full_text_parts.join(" ");
        Ok(TranscriptionResult::new(
            full_text,
            Some("en".to_string()),
            words,
            segments,
        ))
    }
}

// ---------------------------------------------------------------------------
// Test-only deterministic transcriber (gated behind test-mocks feature)
// ---------------------------------------------------------------------------

/// Deterministic, fast, offline local transcriber for test environments only.
/// Uses voice activity energy detection to segment words with frame-accurate timestamps.
#[cfg(any(test, feature = "test-mocks"))]
pub struct DeterministicLocalTranscriber {
    spec: WhisperModelSpec,
}

#[cfg(any(test, feature = "test-mocks"))]
impl DeterministicLocalTranscriber {
    pub fn new(spec: WhisperModelSpec) -> Self {
        Self { spec }
    }
}

#[cfg(any(test, feature = "test-mocks"))]
impl Default for DeterministicLocalTranscriber {
    fn default() -> Self {
        Self::new(WhisperModelSpec::default())
    }
}

#[cfg(any(test, feature = "test-mocks"))]
impl LocalWhisperTranscriber for DeterministicLocalTranscriber {
    fn spec(&self) -> &WhisperModelSpec {
        &self.spec
    }

    fn transcribe(&self, samples: &[f32], sample_rate: f64) -> SearchResult<TranscriptionResult> {
        if samples.is_empty() || sample_rate <= 0.0 {
            return Ok(TranscriptionResult::new(
                "",
                Some("en".to_string()),
                vec![],
                vec![],
            ));
        }

        let resampled = WhisperAudioPreprocessor::resample_to_16k(samples, sample_rate);
        let frame_size = 320; // 20ms at 16kHz
        let frame_count = resampled.len() / frame_size;

        if frame_count == 0 {
            return Ok(TranscriptionResult::new(
                "",
                Some("en".to_string()),
                vec![],
                vec![],
            ));
        }

        // Voice activity energy analysis
        let mut frame_energies = Vec::with_capacity(frame_count);
        for f in 0..frame_count {
            let chunk = &resampled[f * frame_size..(f + 1) * frame_size];
            let energy: f32 = chunk.iter().map(|s| s * s).sum::<f32>() / chunk.len() as f32;
            frame_energies.push(energy);
        }

        let mean_energy: f32 = frame_energies.iter().sum::<f32>() / frame_energies.len() as f32;
        let threshold = (mean_energy * 0.4).max(1e-5);

        // Detect contiguous speech segments
        let mut speech_regions: Vec<(f64, f64)> = Vec::new();
        let mut in_speech = false;
        let mut start_sec = 0.0;

        for (f, &e) in frame_energies.iter().enumerate() {
            let t = f as f64 * 0.02; // 20ms
            if e > threshold && !in_speech {
                in_speech = true;
                start_sec = t;
            } else if e <= threshold && in_speech {
                in_speech = false;
                if t - start_sec >= 0.1 {
                    speech_regions.push((start_sec, t));
                }
            }
        }

        if in_speech {
            let end_sec = frame_count as f64 * 0.02;
            speech_regions.push((start_sec, end_sec));
        }

        let words_bank = [
            "the", "video", "editor", "timeline", "audio", "track", "cut", "transition",
            "render", "export", "scene", "color", "grade", "agent", "marker", "clip",
        ];

        let mut words = Vec::new();
        let mut segments = Vec::new();
        let mut full_text_parts = Vec::new();

        for (seg_idx, (start, end)) in speech_regions.iter().enumerate() {
            let dur = end - start;
            let word_count = ((dur / 0.4).round() as usize).clamp(1, 8);
            let mut seg_words = Vec::new();

            let word_dur = dur / word_count as f64;
            for w in 0..word_count {
                let w_start = start + w as f64 * word_dur;
                let w_end = w_start + word_dur * 0.85;
                let word_str = words_bank[(seg_idx * 3 + w) % words_bank.len()];

                words.push(TranscriptionWord::new(word_str, Some(w_start), Some(w_end)));
                seg_words.push(word_str);
            }

            let segment_text = seg_words.join(" ");
            segments.push(TranscriptionSegment::new(&segment_text, *start, *end));
            full_text_parts.push(segment_text);
        }

        let full_text = full_text_parts.join(". ");
        Ok(TranscriptionResult::new(
            full_text,
            Some("en".to_string()),
            words,
            segments,
        ))
    }
}

// ---------------------------------------------------------------------------
// Local Whisper engine (model lifecycle management)
// ---------------------------------------------------------------------------

/// Local Whisper on-device execution engine.
pub struct LocalWhisperEngine {
    models_dir: PathBuf,
    manifest: WhisperModelManifest,
    state: RwLock<LoaderState>,
    transcriber: RwLock<Option<Arc<dyn LocalWhisperTranscriber>>>,
}

impl LocalWhisperEngine {
    pub fn new(models_dir: impl AsRef<Path>) -> Self {
        let manifest = WhisperModelManifest::default();
        Self {
            models_dir: models_dir.as_ref().to_path_buf(),
            manifest,
            state: RwLock::new(LoaderState::Unknown),
            transcriber: RwLock::new(None),
        }
    }

    pub fn manifest(&self) -> &WhisperModelManifest {
        &self.manifest
    }

    pub fn state(&self) -> LoaderState {
        self.state.read().unwrap().clone()
    }

    pub fn is_ready(&self) -> bool {
        matches!(self.state(), LoaderState::Ready)
    }

    pub fn is_installed(&self) -> bool {
        let model_path = self.models_dir.join(&self.manifest.model_file.name);
        model_path.exists()
    }

    /// Initializes the on-device transcription engine with real whisper-rs inference.
    pub fn prepare(&self) {
        let mut s = self.state.write().unwrap();
        *s = LoaderState::Preparing;
        drop(s);

        if !self.is_installed() {
            let mut s = self.state.write().unwrap();
            *s = LoaderState::NotInstalled;
            info!(
                "Whisper model not installed at {}",
                self.models_dir.display()
            );
            return;
        }

        let spec = WhisperModelSpec {
            name: self.manifest.model.clone(),
            version: 1,
            sample_rate: 16000,
            n_mels: 80,
            is_multilingual: false,
        };

        let model_path = self.models_dir.join(&self.manifest.model_file.name);

        match WhisperRsTranscriber::load(spec, &model_path) {
            Ok(transcriber) => {
                info!("Whisper-rs transcriber loaded successfully");
                *self.transcriber.write().unwrap() = Some(Arc::new(transcriber));
                *self.state.write().unwrap() = LoaderState::Ready;
            }
            Err(e) => {
                error!("Failed to load Whisper model: {e}");
                *self.state.write().unwrap() =
                    LoaderState::Failed(format!("Whisper load failed: {e}"));
            }
        }
    }

    /// Transcribes audio samples on-device.
    pub fn transcribe(
        &self,
        samples: &[f32],
        sample_rate: f64,
    ) -> SearchResult<TranscriptionResult> {
        let transcriber = self.transcriber.read().unwrap().clone();
        match transcriber {
            Some(t) => t.transcribe(samples, sample_rate),
            None => Err(SearchError::ModelNotReady(
                "Local Whisper engine not ready — model may not be installed".into(),
            )),
        }
    }

    /// Test-only: prepare the engine with a deterministic mock transcriber.
    #[cfg(any(test, feature = "test-mocks"))]
    pub fn prepare_with_mock(&self) {
        let spec = WhisperModelSpec::default();
        let transcriber = Arc::new(DeterministicLocalTranscriber::new(spec));
        *self.transcriber.write().unwrap() = Some(transcriber);
        *self.state.write().unwrap() = LoaderState::Ready;
    }

    /// Downloads the GGML model file for on-device Whisper inference.
    pub async fn download(&self) {
        {
            let s = self.state.read().unwrap();
            match *s {
                LoaderState::Downloading(_) | LoaderState::Preparing | LoaderState::Ready => return,
                _ => {}
            }
        }

        *self.state.write().unwrap() = LoaderState::Downloading(0.0);

        let url = match &self.manifest.model_file.url {
            Some(u) => u.clone(),
            None => {
                *self.state.write().unwrap() = LoaderState::Failed(format!(
                    "No download URL for {}",
                    self.manifest.model_file.name
                ));
                return;
            }
        };

        let spec = ModelFileDownload {
            url,
            dest: self.models_dir.join(&self.manifest.model_file.name),
            expected_sha256: self.manifest.model_file.sha256.clone(),
            expected_bytes: self.manifest.model_file.bytes,
        };

        let state_ref = &self.state;
        let result = download_and_verify(&spec, |progress| {
            *state_ref.write().unwrap() = LoaderState::Downloading(progress);
        })
        .await;

        match result {
            Ok(()) => {
                info!("Whisper model downloaded successfully");
                self.prepare();
            }
            Err(e) => {
                error!("Whisper model download failed: {e}");
                *self.state.write().unwrap() =
                    LoaderState::Failed(format!("Download failed: {e}"));
            }
        }
    }
}
