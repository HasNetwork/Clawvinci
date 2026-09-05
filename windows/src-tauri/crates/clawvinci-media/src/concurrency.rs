// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only
// Derived from root AGENTS.md concurrency invariants (GPLv3).

use crate::error::{MediaError, MediaResult};
use std::future::Future;
use std::sync::Arc;
use tokio::sync::Semaphore;
use tokio_util::sync::CancellationToken;

#[derive(Clone)]
pub struct MediaConcurrencyLimiter {
    probe_gate: Arc<Semaphore>,
    decode_gate: Arc<Semaphore>,
    waveform_gate: Arc<Semaphore>,
    encode_gate: Arc<Semaphore>,
}

impl Default for MediaConcurrencyLimiter {
    fn default() -> Self {
        Self::new(8, 4, 2, 1)
    }
}

impl MediaConcurrencyLimiter {
    pub fn new(
        max_concurrent_probes: usize,
        max_concurrent_decodes: usize,
        max_concurrent_waveforms: usize,
        max_concurrent_encodes: usize,
    ) -> Self {
        Self {
            probe_gate: Arc::new(Semaphore::new(max_concurrent_probes.max(1))),
            decode_gate: Arc::new(Semaphore::new(max_concurrent_decodes.max(1))),
            waveform_gate: Arc::new(Semaphore::new(max_concurrent_waveforms.max(1))),
            encode_gate: Arc::new(Semaphore::new(max_concurrent_encodes.max(1))),
        }
    }

    pub async fn run_probe<F, T>(&self, token: &CancellationToken, task: F) -> MediaResult<T>
    where
        F: Future<Output = MediaResult<T>>,
    {
        self.run_gated(&self.probe_gate, token, task).await
    }

    pub async fn run_decode<F, T>(&self, token: &CancellationToken, task: F) -> MediaResult<T>
    where
        F: Future<Output = MediaResult<T>>,
    {
        self.run_gated(&self.decode_gate, token, task).await
    }

    pub async fn run_waveform<F, T>(&self, token: &CancellationToken, task: F) -> MediaResult<T>
    where
        F: Future<Output = MediaResult<T>>,
    {
        self.run_gated(&self.waveform_gate, token, task).await
    }

    pub async fn run_encode<F, T>(&self, token: &CancellationToken, task: F) -> MediaResult<T>
    where
        F: Future<Output = MediaResult<T>>,
    {
        self.run_gated(&self.encode_gate, token, task).await
    }

    async fn run_gated<F, T>(
        &self,
        gate: &Semaphore,
        token: &CancellationToken,
        task: F,
    ) -> MediaResult<T>
    where
        F: Future<Output = MediaResult<T>>,
    {
        tokio::select! {
            _ = token.cancelled() => Err(MediaError::Cancelled),
            permit = gate.acquire() => {
                let _permit = permit.map_err(|_| MediaError::Cancelled)?;
                tokio::select! {
                    _ = token.cancelled() => Err(MediaError::Cancelled),
                    result = task => result,
                }
            }
        }
    }
}
