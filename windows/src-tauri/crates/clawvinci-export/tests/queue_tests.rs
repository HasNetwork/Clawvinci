// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only

use clawvinci_export::queue::{ExportJobSource, ExportJobStatus, ExportQueue};
use std::path::PathBuf;

#[test]
fn test_export_queue_lifecycle() {
    let mut queue = ExportQueue::new();
    let dest_1 = PathBuf::from("C:/export/render1.mp4");
    let dest_2 = PathBuf::from("C:/export/render2.mp4");

    let sub1 = queue
        .enqueue(
            "proj-1".to_string(),
            dest_1.clone(),
            ExportJobSource::Manual,
            vec![],
        )
        .unwrap();

    assert_eq!(sub1.queue_position, 1);
    assert!(sub1.started);

    // Collision rejection on active destination
    let err_collision = queue.enqueue(
        "proj-1".to_string(),
        dest_1.clone(),
        ExportJobSource::Manual,
        vec![],
    );
    assert!(err_collision.is_err());

    // Enqueue second job
    let sub2 = queue
        .enqueue(
            "proj-1".to_string(),
            dest_2.clone(),
            ExportJobSource::Manual,
            vec![],
        )
        .unwrap();
    assert_eq!(sub2.queue_position, 2);

    // Transition first job to preparing
    let (active_id, token) = queue.next_waiting_job().unwrap();
    assert_eq!(active_id, sub1.job_id);
    assert!(!token.is_cancelled());

    // Progress update
    queue.update_progress(active_id, 0.45);
    assert_eq!(queue.get_job(active_id).unwrap().progress, 0.45);
    assert_eq!(
        queue.get_job(active_id).unwrap().status,
        ExportJobStatus::Rendering
    );

    // Finish first job
    queue.finish_job(active_id, ExportJobStatus::Completed, None, vec![]);
    assert_eq!(queue.get_job(active_id).unwrap().progress, 1.0);
    assert_eq!(
        queue.get_job(active_id).unwrap().status,
        ExportJobStatus::Completed
    );

    // Next waiting job
    let (active_id_2, token_2) = queue.next_waiting_job().unwrap();
    assert_eq!(active_id_2, sub2.job_id);

    // Cancel running job
    assert!(queue.cancel(active_id_2));
    assert!(token_2.is_cancelled());
    assert_eq!(
        queue.get_job(active_id_2).unwrap().status,
        ExportJobStatus::Canceling
    );

    queue.finish_job(
        active_id_2,
        ExportJobStatus::Canceled,
        Some("Cancelled by user".to_string()),
        vec![],
    );
    assert_eq!(
        queue.get_job(active_id_2).unwrap().status,
        ExportJobStatus::Canceled
    );

    // Clear finished jobs
    queue.clear_finished(None);
    assert!(queue.jobs().is_empty());
}
