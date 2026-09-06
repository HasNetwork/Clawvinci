// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only
// Derived from Sources/PalmierProTests/PreviewTests/CompositionBuilderTests.swift and VideoEngineTests.swift (GPLv3).

use clawvinci_media::decode::VideoFrame;
use clawvinci_model::blend_mode::BlendMode;
use clawvinci_model::clip_type::ClipType;
use clawvinci_model::timeline::{Clip, Timeline, Track, Transform};
use clawvinci_render::audio::{AudioClock, ScrubAudioEngine};
use clawvinci_render::compositor::composite_frame;
use clawvinci_render::engine::{PlaybackEngine, PlaybackStatus, SeekMode};
use clawvinci_render::plan::{CompositionBuilder, LayerSource};
use std::collections::HashMap;

fn create_test_timeline() -> Timeline {
    let mut timeline = Timeline::new(30, 1920, 1080);

    let mut track1 = Track {
        id: "track-v1".into(),
        track_type: ClipType::Video,
        name: Some("Video 1".into()),
        muted: false,
        hidden: false,
        sync_locked: true,
        display_height: 48.0,
        clips: Vec::new(),
    };

    let mut clip1 = Clip::new("media-video-1", 0, 100);
    clip1.id = "clip-1".into();
    clip1.trim_start_frame = 10;
    clip1.speed = 1.0;
    clip1.opacity = 1.0;
    track1.clips.push(clip1);

    timeline.tracks.push(track1);
    timeline
}

#[test]
fn test_frame_plan_empty_timeline() {
    let timeline = Timeline::new(30, 1280, 720);
    let plan = CompositionBuilder::build_frame_plan(&timeline, 0).expect("build plan");

    assert_eq!(plan.render_width, 1280);
    assert_eq!(plan.render_height, 720);
    assert_eq!(plan.fps, 30);
    assert_eq!(plan.frame_index, 0);
    assert!(plan.layers.is_empty());
    assert!(plan.audio_mix.is_none());
}

#[test]
fn test_frame_plan_single_clip_speed_and_trim() {
    let mut timeline = Timeline::new(30, 1920, 1080);
    let mut track = Track {
        id: "track-v1".into(),
        track_type: ClipType::Video,
        name: Some("Video 1".into()),
        muted: false,
        hidden: false,
        sync_locked: true,
        display_height: 48.0,
        clips: Vec::new(),
    };

    let mut clip = Clip::new("media-source-a", 10, 50);
    clip.id = "clip-a".into();
    clip.trim_start_frame = 5;
    clip.speed = 2.0; // 2x playback speed
    track.clips.push(clip);
    timeline.tracks.push(track);

    // Frame 20 is relative frame 10 (20 - 10).
    // With trim=5 and speed=2.0, source_frame = 5 + (10 * 2.0) = 25.
    let plan = CompositionBuilder::build_frame_plan(&timeline, 20).expect("build plan");
    assert_eq!(plan.layers.len(), 1);
    let layer = &plan.layers[0];
    assert_eq!(layer.clip_id, "clip-a");

    match &layer.source {
        LayerSource::Video {
            media_ref,
            source_frame,
            source_time_seconds,
        } => {
            assert_eq!(media_ref, "media-source-a");
            assert_eq!(*source_frame, 25);
            let expected_time = 25.0 / 30.0;
            assert!((source_time_seconds - expected_time).abs() < 1e-4);
        }
        _ => panic!("Expected Video layer source"),
    }
}

#[test]
fn test_frame_plan_multi_track_layer_order() {
    let mut timeline = Timeline::new(30, 1920, 1080);

    let mut track1 = Track {
        id: "v1".into(),
        track_type: ClipType::Video,
        name: Some("Bottom Track".into()),
        muted: false,
        hidden: false,
        sync_locked: true,
        display_height: 48.0,
        clips: vec![Clip::new("bottom-media", 0, 60)],
    };
    track1.clips[0].id = "bottom-clip".into();

    let mut track2 = Track {
        id: "v2".into(),
        track_type: ClipType::Video,
        name: Some("Top Track".into()),
        muted: false,
        hidden: false,
        sync_locked: true,
        display_height: 48.0,
        clips: vec![Clip::new("top-media", 0, 60)],
    };
    track2.clips[0].id = "top-clip".into();

    timeline.tracks.push(track1);
    timeline.tracks.push(track2);

    let plan = CompositionBuilder::build_frame_plan(&timeline, 15).expect("build plan");
    assert_eq!(plan.layers.len(), 2);
    // V1 must be layer 0 (bottom), V2 must be layer 1 (top)
    assert_eq!(plan.layers[0].clip_id, "bottom-clip");
    assert_eq!(plan.layers[1].clip_id, "top-clip");
}

#[test]
fn test_frame_plan_gap_handling() {
    let mut timeline = Timeline::new(30, 1920, 1080);
    let mut track = Track {
        id: "v1".into(),
        track_type: ClipType::Video,
        name: None,
        muted: false,
        hidden: false,
        sync_locked: true,
        display_height: 48.0,
        clips: vec![
            Clip::new("media-1", 0, 20),
            Clip::new("media-2", 40, 20),
        ],
    };
    track.clips[0].id = "clip-1".into();
    track.clips[1].id = "clip-2".into();
    timeline.tracks.push(track);

    // Frame 10 is inside clip 1
    let plan1 = CompositionBuilder::build_frame_plan(&timeline, 10).expect("plan1");
    assert_eq!(plan1.layers.len(), 1);
    assert_eq!(plan1.layers[0].clip_id, "clip-1");

    // Frame 30 is in the gap (20..40) -> empty layer plan (black background)
    let plan_gap = CompositionBuilder::build_frame_plan(&timeline, 30).expect("plan_gap");
    assert!(plan_gap.layers.is_empty());

    // Frame 45 is inside clip 2
    let plan2 = CompositionBuilder::build_frame_plan(&timeline, 45).expect("plan2");
    assert_eq!(plan2.layers.len(), 1);
    assert_eq!(plan2.layers[0].clip_id, "clip-2");
}

#[test]
fn test_frame_plan_nested_sequence() {
    let mut nested_timeline = Timeline::new(30, 1920, 1080);
    let mut nested_track = Track {
        id: "nested-track".into(),
        track_type: ClipType::Video,
        name: None,
        muted: false,
        hidden: false,
        sync_locked: true,
        display_height: 48.0,
        clips: vec![Clip::new("nested-child-media", 0, 100)],
    };
    nested_track.clips[0].id = "nested-child-clip".into();
    nested_timeline.tracks.push(nested_track);

    let mut main_timeline = Timeline::new(30, 1920, 1080);
    let mut main_track = Track {
        id: "main-track".into(),
        track_type: ClipType::Video,
        name: None,
        muted: false,
        hidden: false,
        sync_locked: true,
        display_height: 48.0,
        clips: Vec::new(),
    };

    let mut nest_carrier = Clip::new("nest-sequence-1", 0, 100);
    nest_carrier.id = "nest-carrier-clip".into();
    nest_carrier.media_type = ClipType::Sequence;
    main_track.clips.push(nest_carrier);
    main_timeline.tracks.push(main_track);

    let plan = CompositionBuilder::build_frame_plan_with_resolvers(
        &main_timeline,
        10,
        |_| None,
        |id| {
            if id == "nest-sequence-1" {
                Some(&nested_timeline)
            } else {
                None
            }
        },
    )
    .expect("build plan with nest");

    assert_eq!(plan.layers.len(), 1);
    match &plan.layers[0].source {
        LayerSource::Nested {
            timeline_id,
            sub_plan,
        } => {
            assert_eq!(timeline_id, "nest-sequence-1");
            assert_eq!(sub_plan.frame_index, 10);
            assert_eq!(sub_plan.layers.len(), 1);
            assert_eq!(sub_plan.layers[0].clip_id, "nested-child-clip");
        }
        _ => panic!("Expected Nested layer source"),
    }
}

#[test]
fn test_composite_frame_black_background() {
    let timeline = Timeline::new(30, 64, 64);
    let plan = CompositionBuilder::build_frame_plan(&timeline, 0).expect("build plan");

    let sources = HashMap::new();
    let frame = composite_frame(&plan, &sources).expect("composite empty frame");

    assert_eq!(frame.width, 64);
    assert_eq!(frame.height, 64);
    assert_eq!(frame.data.len(), 64 * 64 * 4);

    // Verify all pixels are opaque black (0, 0, 0, 255)
    for pixel in frame.data.chunks_exact(4) {
        assert_eq!(pixel, &[0, 0, 0, 255]);
    }
}

#[test]
fn test_composite_frame_layer_opacity_and_transforms() {
    let mut timeline = Timeline::new(30, 100, 100);
    let mut track = Track {
        id: "v1".into(),
        track_type: ClipType::Video,
        name: None,
        muted: false,
        hidden: false,
        sync_locked: true,
        display_height: 48.0,
        clips: Vec::new(),
    };

    let mut clip = Clip::new("red-source", 0, 30);
    clip.id = "red-clip".into();
    clip.opacity = 0.5; // 50% opacity
    clip.transform = Transform {
        center_x: 0.5,
        center_y: 0.5,
        width: 1.0,
        height: 1.0,
        rotation: 0.0,
        rotation_x: 0.0,
        rotation_y: 0.0,
        flip_horizontal: false,
        flip_vertical: false,
    };
    track.clips.push(clip);
    timeline.tracks.push(track);

    let plan = CompositionBuilder::build_frame_plan(&timeline, 0).expect("plan");

    // Pure red source frame: R=255, G=0, B=0, A=255
    let red_frame = VideoFrame {
        width: 100,
        height: 100,
        frame_index: 0,
        data: vec![255, 0, 0, 255]
            .into_iter()
            .cycle()
            .take(100 * 100 * 4)
            .collect(),
    };

    let mut sources = HashMap::new();
    sources.insert("red-clip".to_string(), red_frame);

    let composed = composite_frame(&plan, &sources).expect("composite");

    // Center pixel should be red blended with black at 50% opacity:
    // R ~= 127..128, G = 0, B = 0, A = 255
    let center_pixel = &composed.data[(50 * 100 + 50) * 4..][..4];
    assert!(
        center_pixel[0] >= 120 && center_pixel[0] <= 135,
        "Expected R around 127, got {}",
        center_pixel[0]
    );
    assert_eq!(center_pixel[1], 0);
    assert_eq!(center_pixel[2], 0);
    assert_eq!(center_pixel[3], 255);
}

#[test]
fn test_composite_frame_blend_modes() {
    let mut timeline = Timeline::new(30, 20, 20);

    // Track 1: Solid White background
    let mut track1 = Track {
        id: "v1".into(),
        track_type: ClipType::Video,
        name: None,
        muted: false,
        hidden: false,
        sync_locked: true,
        display_height: 48.0,
        clips: vec![Clip::new("white-media", 0, 10)],
    };
    track1.clips[0].id = "white-clip".into();

    // Track 2: Solid Green with Multiply blend mode
    let mut track2 = Track {
        id: "v2".into(),
        track_type: ClipType::Video,
        name: None,
        muted: false,
        hidden: false,
        sync_locked: true,
        display_height: 48.0,
        clips: vec![Clip::new("green-media", 0, 10)],
    };
    track2.clips[0].id = "green-clip".into();
    track2.clips[0].blend_mode = Some(BlendMode::Multiply);

    timeline.tracks.push(track1);
    timeline.tracks.push(track2);

    let plan = CompositionBuilder::build_frame_plan(&timeline, 0).expect("plan");

    let white_frame = VideoFrame {
        width: 20,
        height: 20,
        frame_index: 0,
        data: vec![255, 255, 255, 255]
            .into_iter()
            .cycle()
            .take(20 * 20 * 4)
            .collect(),
    };
    let green_frame = VideoFrame {
        width: 20,
        height: 20,
        frame_index: 0,
        data: vec![0, 255, 0, 255]
            .into_iter()
            .cycle()
            .take(20 * 20 * 4)
            .collect(),
    };

    let mut sources = HashMap::new();
    sources.insert("white-clip".into(), white_frame);
    sources.insert("green-clip".into(), green_frame);

    let composed = composite_frame(&plan, &sources).expect("composite");

    // White multiplied with Green should yield pure Green: (0, 255, 0, 255)
    let p = &composed.data[..4];
    assert_eq!(p[0], 0);
    assert_eq!(p[1], 255);
    assert_eq!(p[2], 0);
    assert_eq!(p[3], 255);
}

#[test]
fn test_playback_engine_state_transitions() {
    let timeline = create_test_timeline();
    let mut engine = PlaybackEngine::new(timeline);

    assert_eq!(engine.status(), PlaybackStatus::Stopped);
    assert_eq!(engine.current_frame(), 0);

    engine.play();
    assert_eq!(engine.status(), PlaybackStatus::Playing);

    engine.pause();
    assert_eq!(engine.status(), PlaybackStatus::Paused);

    engine.toggle_play();
    assert_eq!(engine.status(), PlaybackStatus::Playing);

    let plan = engine.seek(40, SeekMode::Exact).expect("seek");
    assert_eq!(engine.status(), PlaybackStatus::Paused);
    assert_eq!(engine.current_frame(), 40);
    assert_eq!(plan.frame_index, 40);

    let (step_plan, grain) = engine.step(1).expect("step");
    assert_eq!(engine.current_frame(), 41);
    assert_eq!(step_plan.frame_index, 41);
    assert!(!grain.is_empty());
}

#[test]
fn test_playback_engine_generation_counter_stale_rejection() {
    let timeline = create_test_timeline();
    let mut engine = PlaybackEngine::new(timeline.clone());

    let gen1 = engine.current_generation();

    // Mid-render edit arrives: user modifies timeline or adds a clip
    let mut updated_timeline = timeline;
    updated_timeline.tracks[0].clips[0].duration_frames = 200;
    let gen2 = engine.set_timeline(updated_timeline);

    assert!(gen2 > gen1);
    assert!(engine.is_stale_generation(gen1));
    assert!(!engine.is_stale_generation(gen2));

    let dummy_frame = VideoFrame {
        width: 100,
        height: 100,
        frame_index: 0,
        data: vec![0; 100 * 100 * 4],
    };

    // Frame rendered for stale generation gen1 must be rejected
    let committed_stale = engine.commit_rendered_frame(gen1, dummy_frame.clone());
    assert!(!committed_stale);
    assert!(engine.last_committed_frame().is_none());

    // Frame rendered for current generation gen2 must be committed
    let committed_fresh = engine.commit_rendered_frame(gen2, dummy_frame);
    assert!(committed_fresh);
    assert!(engine.last_committed_frame().is_some());
}

#[test]
fn test_scrub_audio_zipper_synthesis() {
    let step_forward = ScrubAudioEngine::synthesize_step_grain(true, 48_000);
    assert_eq!(step_forward.len(), ScrubAudioEngine::GRAIN_SAMPLES);
    assert!(step_forward.iter().any(|&s| s.abs() > 0.01));

    let step_backward = ScrubAudioEngine::synthesize_step_grain(false, 48_000);
    assert_eq!(step_backward.len(), ScrubAudioEngine::GRAIN_SAMPLES);
    assert!(step_backward.iter().any(|&s| s.abs() > 0.01));

    let scrub_burst = ScrubAudioEngine::synthesize_scrub_burst(5, 48_000);
    assert_eq!(scrub_burst.len(), ScrubAudioEngine::GRAIN_SAMPLES);
    assert!(scrub_burst.iter().any(|&s| s.abs() > 0.01));
}

#[test]
fn test_audio_clock_sync() {
    let mut clock = AudioClock::new();
    assert!(!clock.is_running());
    assert_eq!(clock.current_frame(30), 0);

    clock.start(1.0);
    assert!(clock.is_running());
    assert!(clock.current_time_seconds() >= 1.0);
    assert!(clock.current_frame(30) >= 30);

    clock.seek(2.0);
    assert!(clock.current_time_seconds() >= 2.0);
    assert!(clock.current_frame(30) >= 60);

    clock.pause();
    assert!(!clock.is_running());
    let paused_time = clock.current_time_seconds();
    std::thread::sleep(std::time::Duration::from_millis(10));
    assert_eq!(clock.current_time_seconds(), paused_time);
}
