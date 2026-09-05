// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only

use clawvinci_media::*;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::time::{sleep, Duration};

#[tokio::test]
async fn test_concurrency_limiter_throttles() {
    let limiter = MediaConcurrencyLimiter::new(2, 2, 2, 2);
    let active_count = Arc::new(AtomicUsize::new(0));
    let max_observed = Arc::new(AtomicUsize::new(0));

    let mut handles = Vec::new();

    for _ in 0..6 {
        let lim = limiter.clone();
        let act = active_count.clone();
        let max_obs = max_observed.clone();
        let token = CancellationToken::new();

        handles.push(tokio::spawn(async move {
            lim.run_probe(&token, async {
                let current = act.fetch_add(1, Ordering::SeqCst) + 1;
                max_obs.fetch_max(current, Ordering::SeqCst);
                sleep(Duration::from_millis(50)).await;
                act.fetch_sub(1, Ordering::SeqCst);
                Ok(())
            })
            .await
        }));
    }

    for h in handles {
        let res = h.await.expect("task join failed");
        assert!(res.is_ok());
    }

    assert!(max_observed.load(Ordering::SeqCst) <= 2);
}

#[tokio::test]
async fn test_concurrency_limiter_immediate_cancellation() {
    let limiter = MediaConcurrencyLimiter::new(1, 1, 1, 1);
    let token = CancellationToken::new();
    token.cancel(); // Pre-cancelled

    let result: MediaResult<()> = limiter
        .run_decode(&token, async {
            panic!("Should not execute task when token is cancelled");
        })
        .await;

    assert!(result.is_err());
    match result.unwrap_err() {
        MediaError::Cancelled => {}
        other => panic!("Expected MediaError::Cancelled, got: {:?}", other),
    }
}

#[tokio::test]
async fn test_concurrency_limiter_mid_flight_cancellation() {
    let limiter = MediaConcurrencyLimiter::new(1, 1, 1, 1);
    let token = CancellationToken::new();
    let token_clone = token.clone();

    // Spawn a background task to cancel the token after 20ms
    tokio::spawn(async move {
        sleep(Duration::from_millis(20)).await;
        token_clone.cancel();
    });

    let result: MediaResult<()> = limiter
        .run_decode(&token, async {
            sleep(Duration::from_millis(500)).await;
            Ok(())
        })
        .await;

    assert!(result.is_err());
    match result.unwrap_err() {
        MediaError::Cancelled => {}
        other => panic!("Expected MediaError::Cancelled, got: {:?}", other),
    }
}
