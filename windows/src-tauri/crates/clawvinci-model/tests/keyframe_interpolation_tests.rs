// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only

use clawvinci_model::*;

#[test]
fn test_keyframe_linear_interpolation() {
    let mut track = KeyframeTrack::default();
    track.upsert(Keyframe::new(0, 0.0, Interpolation::Linear));
    track.upsert(Keyframe::new(100, 100.0, Interpolation::Linear));

    assert_eq!(track.sample(-10, -1.0), 0.0);
    assert_eq!(track.sample(0, -1.0), 0.0);
    assert_eq!(track.sample(25, -1.0), 25.0);
    assert_eq!(track.sample(50, -1.0), 50.0);
    assert_eq!(track.sample(75, -1.0), 75.0);
    assert_eq!(track.sample(100, -1.0), 100.0);
    assert_eq!(track.sample(150, -1.0), 100.0);
}

#[test]
fn test_keyframe_hold_interpolation() {
    let mut track = KeyframeTrack::default();
    track.upsert(Keyframe::new(0, 10.0, Interpolation::Hold));
    track.upsert(Keyframe::new(50, 20.0, Interpolation::Hold));
    track.upsert(Keyframe::new(100, 30.0, Interpolation::Hold));

    assert_eq!(track.sample(0, 0.0), 10.0);
    assert_eq!(track.sample(25, 0.0), 10.0);
    assert_eq!(track.sample(49, 0.0), 10.0);
    assert_eq!(track.sample(50, 0.0), 20.0);
    assert_eq!(track.sample(99, 0.0), 20.0);
    assert_eq!(track.sample(100, 0.0), 30.0);
}

#[test]
fn test_keyframe_smooth_interpolation() {
    let mut track = KeyframeTrack::default();
    track.upsert(Keyframe::new(0, 0.0, Interpolation::Smooth));
    track.upsert(Keyframe::new(100, 1.0, Interpolation::Smooth));

    let mid = track.sample(50, 0.0);
    // At t=0.5, smoothstep(0.5) = 0.5 * 0.5 * (3 - 1) = 0.5
    assert!((mid - 0.5).abs() < 1e-6);

    let quarter = track.sample(25, 0.0);
    // At t=0.25, smoothstep(0.25) = 0.0625 * 2.5 = 0.15625 < 0.25 (easing in)
    assert!((quarter - 0.15625).abs() < 1e-6);
}

#[test]
fn test_anim_pair_and_crop_interpolation() {
    let mut pos_track = KeyframeTrack::default();
    pos_track.upsert(Keyframe::new(0, AnimPair::new(0.0, 10.0), Interpolation::Linear));
    pos_track.upsert(Keyframe::new(10, AnimPair::new(10.0, 20.0), Interpolation::Linear));

    let sample = pos_track.sample(5, AnimPair::new(0.0, 0.0));
    assert_eq!(sample.a, 5.0);
    assert_eq!(sample.b, 15.0);

    let mut crop_track = KeyframeTrack::default();
    crop_track.upsert(Keyframe::new(0, Crop::new(0.0, 0.0, 0.0, 0.0), Interpolation::Linear));
    crop_track.upsert(Keyframe::new(100, Crop::new(0.2, 0.1, 0.2, 0.1), Interpolation::Linear));

    let crop_mid = crop_track.sample(50, Crop::default());
    assert!((crop_mid.left - 0.1).abs() < 1e-6);
    assert!((crop_mid.top - 0.05).abs() < 1e-6);
}
