// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only

use clawvinci_media::*;
use std::path::Path;

#[test]
fn test_probe_json_parsing_success() {
    let sample_json = r#"{
        "format": {
            "format_name": "mov,mp4,m4a,3gp,3g2,mj2",
            "duration": "12.456000",
            "size": "45239120",
            "bit_rate": "29055311"
        },
        "streams": [
            {
                "index": 0,
                "codec_type": "video",
                "codec_name": "h264",
                "width": 3840,
                "height": 2160,
                "r_frame_rate": "30000/1001",
                "nb_frames": "374",
                "pix_fmt": "yuv420p",
                "color_space": "bt709",
                "color_primaries": "bt709",
                "color_transfer": "bt709",
                "side_data_list": [
                    {
                        "rotation": -90
                    }
                ]
            },
            {
                "index": 1,
                "codec_type": "audio",
                "codec_name": "aac",
                "sample_rate": "48000",
                "channels": 2,
                "channel_layout": "stereo",
                "bit_rate": "320000"
            }
        ]
    }"#;

    let probe = parse_probe_json(Path::new("dummy.mp4"), sample_json)
        .expect("Failed to parse probe JSON");

    assert_eq!(probe.format_name, "mov,mp4,m4a,3gp,3g2,mj2");
    assert!((probe.duration_seconds - 12.456).abs() < 1e-4);
    assert_eq!(probe.size_bytes, 45239120);
    assert_eq!(probe.bitrate, 29055311);

    // Video stream
    let video = probe.primary_video().expect("expected video stream");
    assert_eq!(video.index, 0);
    assert_eq!(video.codec_name, "h264");
    assert_eq!(video.width, 3840);
    assert_eq!(video.height, 2160);
    assert_eq!(video.fps_numerator, 30000);
    assert_eq!(video.fps_denominator, 1001);
    assert!((video.fps - 29.970029).abs() < 1e-4);
    assert_eq!(video.duration_frames, 374);
    assert_eq!(video.pixel_format, "yuv420p");
    assert_eq!(video.color_space.as_deref(), Some("bt709"));
    assert_eq!(video.rotation, -90);

    // Audio stream
    let audio = probe.primary_audio().expect("expected audio stream");
    assert_eq!(audio.index, 1);
    assert_eq!(audio.codec_name, "aac");
    assert_eq!(audio.sample_rate, 48000);
    assert_eq!(audio.channels, 2);
    assert_eq!(audio.channel_layout, "stereo");
    assert_eq!(audio.bitrate, 320000);
}

#[tokio::test]
async fn test_probe_missing_file_error() {
    let ctx = FfmpegContext::with_paths("ffmpeg".into(), "ffprobe".into());
    let missing_path = Path::new("non_existent_media_file_12345.mp4");

    let result = probe_media(&ctx, missing_path).await;
    assert!(result.is_err());
    match result.unwrap_err() {
        MediaError::InvalidInput(msg) => {
            assert!(msg.contains("Media file not found"));
        }
        other => panic!("Expected InvalidInput error, got: {:?}", other),
    }
}
