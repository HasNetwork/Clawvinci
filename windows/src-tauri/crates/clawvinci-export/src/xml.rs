// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only
// Derived from Sources/PalmierPro/Export/XMLExporter.swift (GPLv3).

use crate::error::{ExportError, ExportResult};
use clawvinci_model::clip_type::ClipType;
use clawvinci_model::timeline::{Clip, Timeline, Track};
use std::collections::{HashMap, HashSet};
use std::path::Path;

// MARK: - XML Node Tree

#[derive(Debug, Clone, PartialEq)]
pub struct XMLNode {
    pub name: String,
    pub attributes: Vec<(String, String)>,
    pub text: Option<String>,
    pub children: Vec<XMLNode>,
}

pub fn el(name: &str, children: Vec<XMLNode>) -> XMLNode {
    XMLNode {
        name: name.to_string(),
        attributes: Vec::new(),
        text: None,
        children,
    }
}

pub fn el_attrs(name: &str, attrs: Vec<(&str, &str)>, children: Vec<XMLNode>) -> XMLNode {
    XMLNode {
        name: name.to_string(),
        attributes: attrs
            .into_iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect(),
        text: None,
        children,
    }
}

pub fn leaf_str(name: &str, val: &str) -> XMLNode {
    XMLNode {
        name: name.to_string(),
        attributes: Vec::new(),
        text: Some(val.to_string()),
        children: Vec::new(),
    }
}

pub fn leaf_i64(name: &str, val: i64) -> XMLNode {
    leaf_str(name, &val.to_string())
}

pub fn leaf_bool(name: &str, val: bool) -> XMLNode {
    leaf_str(name, if val { "TRUE" } else { "FALSE" })
}

pub fn render_xml(node: &XMLNode, indent: usize) -> String {
    let pad = " ".repeat(indent);
    let mut attr_str = String::new();
    for (k, v) in &node.attributes {
        attr_str.push_str(&format!(" {}=\"{}\"", k, escape_xml(v)));
    }

    if let Some(text) = &node.text {
        return format!("{}<{}{}>{}</{}>", pad, node.name, attr_str, escape_xml(text), node.name);
    }

    if node.children.is_empty() {
        return format!("{}<{}{}/>", pad, node.name, attr_str);
    }

    let mut inner = Vec::new();
    for child in &node.children {
        inner.push(render_xml(child, indent + 2));
    }
    format!("{}<{}{}>\n{}\n{}</{}>", pad, node.name, attr_str, inner.join("\n"), pad, node.name)
}

pub fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

// MARK: - SMPTE Timecode

pub fn format_smpte_timecode(frame: i64, fps: i32, drop_frame: bool) -> String {
    if fps <= 0 {
        return "00:00:00:00".to_string();
    }
    let mut f = frame.max(0);
    let fps_i64 = fps as i64;
    if drop_frame {
        let drop = ((fps as f64) * 0.066666).round() as i64;
        let per_minute = fps_i64 * 60 - drop;
        let per_10 = per_minute * 10 + drop;
        let d = f / per_10;
        let m = f % per_10;
        let drop_adder = if m > drop {
            drop * ((m - drop) / per_minute)
        } else {
            0
        };
        f += drop * 9 * d + drop_adder;
    }
    let sep = if drop_frame { ";" } else { ":" };
    let ff = f % fps_i64;
    let ss = (f / fps_i64) % 60;
    let mm = (f / (fps_i64 * 60)) % 60;
    let hh = f / (fps_i64 * 3600);
    format!("{:02}{:02}{:02}{:02}", hh, mm, ss, ff)
        .as_bytes()
        .chunks(2)
        .map(|c| std::str::from_utf8(c).unwrap_or("00"))
        .collect::<Vec<_>>()
        .join(sep)
}

fn rate_tags(fps: f64) -> (i32, bool) {
    if (fps - 23.976).abs() < 0.05 {
        (24, true)
    } else if (fps - 29.97).abs() < 0.05 {
        (30, true)
    } else if (fps - 59.94).abs() < 0.05 {
        (60, true)
    } else {
        (fps.round() as i32, false)
    }
}

fn file_url_for_path(path_str: &str) -> String {
    let p = Path::new(path_str);
    let s = p.to_string_lossy().replace('\\', "/");
    if s.starts_with('/') {
        format!("file://localhost{}", s)
    } else {
        format!("file://localhost/{}", s)
    }
}

// MARK: - XMEML Exporter

pub struct XMLExporter;

impl XMLExporter {
    pub fn render(
        timeline: &Timeline,
        media_paths: &HashMap<String, String>,
    ) -> String {
        let root = build_xmeml_tree(timeline, media_paths);
        format!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<!DOCTYPE xmeml>\n{}",
            render_xml(&root, 0)
        )
    }

    pub fn export_to_file(
        timeline: &Timeline,
        media_paths: &HashMap<String, String>,
        output_path: &Path,
    ) -> ExportResult<()> {
        let xml_str = Self::render(timeline, media_paths);
        std::fs::write(output_path, xml_str).map_err(|e| {
            ExportError::XmlEncodingFailed(format!("Failed to write XMEML file: {}", e))
        })?;
        Ok(())
    }
}

fn build_xmeml_tree(
    timeline: &Timeline,
    media_paths: &HashMap<String, String>,
) -> XMLNode {
    let seq_node = build_sequence_node(timeline, "sequence-1", media_paths);
    el_attrs("xmeml", vec![("version", "4")], vec![seq_node])
}

fn build_sequence_node(
    timeline: &Timeline,
    seq_id: &str,
    media_paths: &HashMap<String, String>,
) -> XMLNode {
    let (timebase, ntsc) = rate_tags(timeline.fps as f64);
    let total_frames = timeline.total_frames();

    // Video tracks in XMEML are bottom-to-top, whereas timeline model is top-to-bottom.
    let mut video_tracks: Vec<&Track> = timeline
        .tracks
        .iter()
        .filter(|t| t.track_type != ClipType::Audio)
        .collect();
    video_tracks.reverse();

    let audio_tracks: Vec<&Track> = timeline
        .tracks
        .iter()
        .filter(|t| t.track_type == ClipType::Audio)
        .collect();

    let mut emitted_files = HashSet::new();

    let mut video_nodes = Vec::new();
    video_nodes.push(build_video_format_node(timeline.width as i64, timeline.height as i64, timebase, ntsc));
    for track in video_tracks {
        video_nodes.push(build_track_node(track, false, timebase, ntsc, media_paths, &mut emitted_files));
    }

    let mut audio_nodes = Vec::new();
    audio_nodes.push(leaf_i64("numOutputChannels", 2));
    audio_nodes.push(build_audio_format_node());
    audio_nodes.push(build_audio_outputs_node());
    for track in audio_tracks {
        audio_nodes.push(build_track_node(track, true, timebase, ntsc, media_paths, &mut emitted_files));
    }

    let rate_node = el("rate", vec![
        leaf_i64("timebase", timebase as i64),
        leaf_bool("ntsc", ntsc),
    ]);

    let timecode_node = el("timecode", vec![
        rate_node.clone(),
        leaf_str("string", "00:00:00:00"),
        leaf_i64("frame", 0),
        leaf_str("source", "source"),
        leaf_str("displayformat", "NDF"),
    ]);

    el_attrs(
        "sequence",
        vec![("id", seq_id)],
        vec![
            leaf_str("name", &timeline.name),
            leaf_i64("duration", total_frames),
            rate_node,
            timecode_node,
            el("media", vec![
                el("video", video_nodes),
                el("audio", audio_nodes),
            ]),
        ],
    )
}

fn build_video_format_node(width: i64, height: i64, timebase: i32, ntsc: bool) -> XMLNode {
    el("format", vec![
        el("samplecharacteristics", vec![
            leaf_i64("width", width),
            leaf_i64("height", height),
            leaf_bool("anamorphic", false),
            leaf_str("pixelaspectratio", "square"),
            leaf_str("fielddominance", "none"),
            el("rate", vec![
                leaf_i64("timebase", timebase as i64),
                leaf_bool("ntsc", ntsc),
            ]),
        ]),
    ])
}

fn build_audio_format_node() -> XMLNode {
    el("format", vec![
        el("samplecharacteristics", vec![
            leaf_i64("samplerate", 48000),
            leaf_i64("depth", 16),
        ]),
    ])
}

fn build_audio_outputs_node() -> XMLNode {
    el("outputs", vec![
        el("group", vec![
            leaf_i64("index", 1),
            leaf_i64("numchannels", 2),
            leaf_i64("downmix", 0),
            el("channel", vec![leaf_i64("index", 1)]),
            el("channel", vec![leaf_i64("index", 2)]),
        ]),
    ])
}

fn build_track_node(
    track: &Track,
    is_audio: bool,
    timebase: i32,
    ntsc: bool,
    media_paths: &HashMap<String, String>,
    emitted_files: &mut HashSet<String>,
) -> XMLNode {
    let enabled = if is_audio { !track.muted } else { !track.hidden };
    let mut children = vec![
        leaf_bool("enabled", enabled),
        leaf_bool("locked", false),
    ];

    for clip in &track.clips {
        children.push(build_clipitem_node(clip, is_audio, timebase, ntsc, media_paths, emitted_files));
    }

    el("track", children)
}

fn build_clipitem_node(
    clip: &Clip,
    is_audio: bool,
    timebase: i32,
    ntsc: bool,
    media_paths: &HashMap<String, String>,
    emitted_files: &mut HashSet<String>,
) -> XMLNode {
    let clip_id = format!("clipitem-{}", clip.id);
    let name = &clip.media_ref;
    let rate_node = el("rate", vec![
        leaf_i64("timebase", timebase as i64),
        leaf_bool("ntsc", ntsc),
    ]);

    let mut children = vec![
        leaf_str("name", name),
        leaf_bool("enabled", true),
        leaf_i64("duration", clip.duration_frames),
        rate_node.clone(),
        leaf_i64("start", clip.start_frame),
        leaf_i64("end", clip.end_frame()),
        leaf_i64("in", clip.trim_start_frame),
        leaf_i64("out", clip.trim_start_frame + clip.duration_frames),
    ];

    // File reference
    children.push(build_file_node(clip, is_audio, timebase, ntsc, media_paths, emitted_files));

    // Filters (Basic Motion, Opacity, Crop, Volume, Speed)
    if is_audio {
        let volume_db = if clip.volume > 0.0001 {
            20.0 * clip.volume.log10()
        } else {
            -60.0
        };
        children.push(build_audio_levels_filter(volume_db));
    } else {
        // Basic Motion filter (scale, rotation, center)
        let scale_percent = clip.transform.width * 100.0;
        let rot_ccw = -clip.transform.rotation;
        let center_x = (clip.transform.center_x - 0.5) * 2.0;
        let center_y = -(clip.transform.center_y - 0.5) * 2.0; // invert Y for FCP coordinate space
        children.push(build_basic_motion_filter(scale_percent, rot_ccw, center_x, center_y));

        // Opacity filter
        if (clip.opacity - 1.0).abs() > 0.001 {
            children.push(build_opacity_filter(clip.opacity as f32 * 100.0));
        }

        // Crop filter
        if !clip.crop.is_identity() {
            children.push(build_crop_filter(&clip.crop));
        }
    }

    // Speed / Time Remap filter if non-1.0
    if (clip.speed - 1.0).abs() > 0.001 && clip.speed > 0.0 {
        children.push(build_time_remap_filter(clip.speed));
    }

    el_attrs("clipitem", vec![("id", &clip_id)], children)
}

fn build_file_node(
    clip: &Clip,
    is_audio: bool,
    timebase: i32,
    ntsc: bool,
    media_paths: &HashMap<String, String>,
    emitted_files: &mut HashSet<String>,
) -> XMLNode {
    let file_id = format!("file-{}", clip.media_ref);
    let full_path = media_paths.get(&clip.media_ref).cloned().unwrap_or_else(|| clip.media_ref.clone());
    let path_url = file_url_for_path(&full_path);

    if emitted_files.contains(&clip.media_ref) {
        // Shared file reference
        return el_attrs("file", vec![("id", &file_id)], Vec::new());
    }

    emitted_files.insert(clip.media_ref.clone());

    let rate_node = el("rate", vec![
        leaf_i64("timebase", timebase as i64),
        leaf_bool("ntsc", ntsc),
    ]);

    let media_node = if is_audio {
        el("media", vec![
            el("audio", vec![
                el("samplecharacteristics", vec![
                    leaf_i64("samplerate", 48000),
                    leaf_i64("depth", 16),
                ]),
                el("channelcount", vec![leaf_i64("numchannels", 2)]),
            ]),
        ])
    } else {
        el("media", vec![
            el("video", vec![
                el("samplecharacteristics", vec![
                    leaf_i64("width", 1920),
                    leaf_i64("height", 1080),
                    rate_node.clone(),
                ]),
            ]),
        ])
    };

    el_attrs(
        "file",
        vec![("id", &file_id)],
        vec![
            leaf_str("name", &clip.media_ref),
            leaf_str("pathurl", &path_url),
            rate_node,
            leaf_i64("duration", clip.duration_frames.max(100)),
            media_node,
        ],
    )
}

fn build_basic_motion_filter(scale: f64, rotation: f64, center_x: f64, center_y: f64) -> XMLNode {
    el("filter", vec![
        el("effect", vec![
            leaf_str("name", "Basic Motion"),
            leaf_str("effectid", "basic"),
            leaf_str("effectcategory", "motion"),
            leaf_str("effecttype", "motion"),
            leaf_str("mediatype", "video"),
            build_parameter("scale", "Scale", "0", "1000", &format!("{:.2}", scale)),
            build_parameter("rotation", "Rotation", "-86400", "86400", &format!("{:.2}", rotation)),
            build_center_parameter(center_x, center_y),
        ]),
    ])
}

fn build_opacity_filter(opacity_percent: f32) -> XMLNode {
    el("filter", vec![
        el("effect", vec![
            leaf_str("name", "Opacity"),
            leaf_str("effectid", "opacity"),
            leaf_str("effectcategory", "motion"),
            leaf_str("effecttype", "motion"),
            leaf_str("mediatype", "video"),
            build_parameter("opacity", "Opacity", "0", "100", &format!("{:.1}", opacity_percent)),
        ]),
    ])
}

fn build_crop_filter(crop: &clawvinci_model::timeline::Crop) -> XMLNode {
    el("filter", vec![
        el("effect", vec![
            leaf_str("name", "Crop"),
            leaf_str("effectid", "crop"),
            leaf_str("effectcategory", "motion"),
            leaf_str("effecttype", "motion"),
            leaf_str("mediatype", "video"),
            build_parameter("left", "Left", "0", "100", &format!("{:.2}", crop.left * 100.0)),
            build_parameter("top", "Top", "0", "100", &format!("{:.2}", crop.top * 100.0)),
            build_parameter("right", "Right", "0", "100", &format!("{:.2}", crop.right * 100.0)),
            build_parameter("bottom", "Bottom", "0", "100", &format!("{:.2}", crop.bottom * 100.0)),
        ]),
    ])
}

fn build_audio_levels_filter(volume_db: f64) -> XMLNode {
    el("filter", vec![
        el("effect", vec![
            leaf_str("name", "Audio Levels"),
            leaf_str("effectid", "audiolevels"),
            leaf_str("effectcategory", "audio"),
            leaf_str("effecttype", "audio"),
            leaf_str("mediatype", "audio"),
            build_parameter("level", "Level", "-60", "12", &format!("{:.2}", volume_db)),
        ]),
    ])
}

fn build_time_remap_filter(speed: f64) -> XMLNode {
    el("filter", vec![
        el("effect", vec![
            leaf_str("name", "Time Remap"),
            leaf_str("effectid", "timeremap"),
            leaf_str("effectcategory", "motion"),
            leaf_str("effecttype", "motion"),
            leaf_str("mediatype", "video"),
            build_parameter("speed", "Speed", "0.01", "100", &format!("{:.2}", speed * 100.0)),
        ]),
    ])
}

fn build_parameter(id: &str, name: &str, min: &str, max: &str, value: &str) -> XMLNode {
    el("parameter", vec![
        leaf_str("parameterid", id),
        leaf_str("name", name),
        leaf_str("valuemin", min),
        leaf_str("valuemax", max),
        leaf_str("value", value),
    ])
}

fn build_center_parameter(x: f64, y: f64) -> XMLNode {
    el("parameter", vec![
        leaf_str("parameterid", "center"),
        leaf_str("name", "Center"),
        el("value", vec![
            leaf_str("horiz", &format!("{:.5}", x)),
            leaf_str("vert", &format!("{:.5}", y)),
        ]),
    ])
}
