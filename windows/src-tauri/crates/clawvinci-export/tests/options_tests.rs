// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only

use clawvinci_export::options::{ExportFormat, ExportResolution, VideoCodec};

#[test]
fn test_render_size_standards() {
    let res_1080p = ExportResolution::R1080p;
    assert_eq!(res_1080p.render_size(1920, 1080), (1920, 1080));

    let res_720p = ExportResolution::R720p;
    assert_eq!(res_720p.render_size(1920, 1080), (1280, 720));

    let res_4k = ExportResolution::R4k;
    assert_eq!(res_4k.render_size(1920, 1080), (3840, 2160));

    let res_match = ExportResolution::MatchTimeline;
    assert_eq!(res_match.render_size(1920, 1080), (1920, 1080));
}

#[test]
fn test_render_size_vertical_video() {
    // 9:16 vertical video (1080x1920)
    let res_720p = ExportResolution::R720p;
    assert_eq!(res_720p.render_size(1080, 1920), (720, 1280));
}

#[test]
fn test_render_size_even_dimensions_enforced() {
    let custom = ExportResolution::Custom { width: 853, height: 479 };
    assert_eq!(custom.render_size(1920, 1080), (852, 478));
}

#[test]
fn test_format_extensions_and_codecs() {
    assert_eq!(ExportFormat::H264.file_extension(), "mp4");
    assert_eq!(ExportFormat::H265.file_extension(), "mp4");
    assert_eq!(ExportFormat::ProRes.file_extension(), "mov");
    assert_eq!(ExportFormat::HevcHdr.file_extension(), "mov");
    assert_eq!(ExportFormat::Xml.file_extension(), "xml");
    assert_eq!(ExportFormat::Fcpxml.file_extension(), "fcpxml");

    assert!(!ExportFormat::H264.is_hdr());
    assert!(ExportFormat::HevcHdr.is_hdr());

    assert_eq!(VideoCodec::H264.container_label(), "MPEG-4 (.mp4)");
    assert_eq!(VideoCodec::ProRes.container_label(), "QuickTime (.mov)");
}
