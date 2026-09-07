// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only
// Derived from Sources/PalmierPro/Export/FCPXMLExporter.swift (GPLv3).

use crate::error::{ExportError, ExportResult};
use crate::options::{FCPXMLTarget, FCPXMLVersion};
use crate::xml::{render_xml, XMLNode};
use clawvinci_model::clip_type::ClipType;
use clawvinci_model::timeline::{Clip, Timeline, Track};
use std::collections::HashMap;
use std::path::Path;

pub struct FCPXMLExporter;

impl FCPXMLExporter {
    pub fn render(
        timeline: &Timeline,
        media_paths: &HashMap<String, String>,
        version: FCPXMLVersion,
        target: FCPXMLTarget,
    ) -> String {
        let root = build_fcpxml_tree(timeline, media_paths, version, target);
        format!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<!DOCTYPE fcpxml>\n{}",
            render_xml(&root, 0)
        )
    }

    pub fn export_to_file(
        timeline: &Timeline,
        media_paths: &HashMap<String, String>,
        version: FCPXMLVersion,
        target: FCPXMLTarget,
        output_path: &Path,
    ) -> ExportResult<()> {
        let xml_str = Self::render(timeline, media_paths, version, target);
        std::fs::write(output_path, xml_str).map_err(|e| {
            ExportError::XmlEncodingFailed(format!("Failed to write FCPXML file: {}", e))
        })?;
        Ok(())
    }
}

pub fn frame_duration_str(fps: i32) -> &'static str {
    match fps {
        24 => "1/24s",
        25 => "1/25s",
        30 => "1/30s",
        50 => "1/50s",
        60 => "1/60s",
        _ => "1/30s",
    }
}

pub fn format_fcpxml_time(frames: i64, fps: i32) -> String {
    let f = frames.max(0);
    let fps_val = fps.max(1);
    format!("{}/{}s", f, fps_val)
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

fn build_fcpxml_tree(
    timeline: &Timeline,
    media_paths: &HashMap<String, String>,
    version: FCPXMLVersion,
    target: FCPXMLTarget,
) -> XMLNode {
    let mut resources = Vec::new();
    let format_id = "r1";
    let format_dur = frame_duration_str(timeline.fps);

    // Format resource
    resources.push(XMLNode {
        name: "format".to_string(),
        attributes: vec![
            ("id".to_string(), format_id.to_string()),
            ("name".to_string(), format!("FFVideoFormat{}p{}", timeline.height, timeline.fps)),
            ("frameDuration".to_string(), format_dur.to_string()),
            ("width".to_string(), timeline.width.to_string()),
            ("height".to_string(), timeline.height.to_string()),
        ],
        text: None,
        children: Vec::new(),
    });

    // Asset resources
    let mut asset_id_map: HashMap<String, String> = HashMap::new();
    let mut next_asset_id = 2;

    for track in &timeline.tracks {
        for clip in &track.clips {
            if clip.media_type == ClipType::Text {
                continue;
            }
            if !asset_id_map.contains_key(&clip.media_ref) {
                let id_str = format!("r{}", next_asset_id);
                next_asset_id += 1;
                asset_id_map.insert(clip.media_ref.clone(), id_str.clone());

                let full_path = media_paths
                    .get(&clip.media_ref)
                    .cloned()
                    .unwrap_or_else(|| clip.media_ref.clone());
                let file_url = file_url_for_path(&full_path);
                let dur_str = format_fcpxml_time(clip.duration_frames.max(300), timeline.fps);

                let is_audio = track.track_type == ClipType::Audio;
                let mut asset_attrs = vec![
                    ("id".to_string(), id_str),
                    ("name".to_string(), clip.media_ref.clone()),
                    ("start".to_string(), "0s".to_string()),
                    ("duration".to_string(), dur_str),
                    ("format".to_string(), format_id.to_string()),
                ];
                if is_audio {
                    asset_attrs.push(("hasAudio".to_string(), "1".to_string()));
                    asset_attrs.push(("audioSources".to_string(), "1".to_string()));
                    asset_attrs.push(("audioChannels".to_string(), "2".to_string()));
                } else {
                    asset_attrs.push(("hasVideo".to_string(), "1".to_string()));
                }

                let media_rep = XMLNode {
                    name: "media-rep".to_string(),
                    attributes: vec![
                        ("kind".to_string(), "original-media".to_string()),
                        ("src".to_string(), file_url),
                    ],
                    text: None,
                    children: Vec::new(),
                };

                resources.push(XMLNode {
                    name: "asset".to_string(),
                    attributes: asset_attrs,
                    text: None,
                    children: vec![media_rep],
                });
            }
        }
    }

    // Effect resources for titles/text
    let effect_id = format!("r{}", next_asset_id);
    resources.push(XMLNode {
        name: "effect".to_string(),
        attributes: vec![
            ("id".to_string(), effect_id.clone()),
            ("name".to_string(), "Basic Title".to_string()),
            ("uid".to_string(), ".../Titles.localized/Bumper:Glow.localized/Basic Title.localized/Basic Title.moti".to_string()),
        ],
        text: None,
        children: Vec::new(),
    });

    let resources_node = XMLNode {
        name: "resources".to_string(),
        attributes: Vec::new(),
        text: None,
        children: resources,
    };

    // Build spine elements
    let spine_children = build_spine_elements(timeline, &asset_id_map, &effect_id, target);

    let total_dur_str = format_fcpxml_time(timeline.duration(), timeline.fps);

    let sequence_node = XMLNode {
        name: "sequence".to_string(),
        attributes: vec![
            ("format".to_string(), format_id.to_string()),
            ("duration".to_string(), total_dur_str),
            ("tcStart".to_string(), "0s".to_string()),
            ("tcFormat".to_string(), "NDF".to_string()),
        ],
        text: None,
        children: vec![XMLNode {
            name: "spine".to_string(),
            attributes: Vec::new(),
            text: None,
            children: spine_children,
        }],
    };

    let project_node = XMLNode {
        name: "project".to_string(),
        attributes: vec![("name".to_string(), timeline.name.clone())],
        text: None,
        children: vec![sequence_node],
    };

    let event_node = XMLNode {
        name: "event".to_string(),
        attributes: vec![("name".to_string(), "Clawvinci Export".to_string())],
        text: None,
        children: vec![project_node],
    };

    let library_node = XMLNode {
        name: "library".to_string(),
        attributes: Vec::new(),
        text: None,
        children: vec![event_node],
    };

    XMLNode {
        name: "fcpxml".to_string(),
        attributes: vec![("version".to_string(), version.version_string().to_string())],
        text: None,
        children: vec![resources_node, library_node],
    }
}

fn build_spine_elements(
    timeline: &Timeline,
    asset_id_map: &HashMap<String, String>,
    title_effect_id: &str,
    target: FCPXMLTarget,
) -> Vec<XMLNode> {
    let mut elements = Vec::new();
    let total_duration = timeline.duration();
    let fps = timeline.fps;

    if timeline.tracks.is_empty() {
        elements.push(XMLNode {
            name: "gap".to_string(),
            attributes: vec![
                ("name".to_string(), "Gap".to_string()),
                ("offset".to_string(), "0s".to_string()),
                ("duration".to_string(), format_fcpxml_time(total_duration.max(30), fps)),
            ],
            text: None,
            children: Vec::new(),
        });
        return elements;
    }

    // Lane 0 is primary track
    let primary_track = &timeline.tracks[0];
    let mut current_frame: i64 = 0;

    for clip in &primary_track.clips {
        // Gap before clip
        if clip.start_frame > current_frame {
            let gap_len = clip.start_frame - current_frame;
            elements.push(XMLNode {
                name: "gap".to_string(),
                attributes: vec![
                    ("name".to_string(), "Gap".to_string()),
                    ("offset".to_string(), format_fcpxml_time(current_frame, fps)),
                    ("duration".to_string(), format_fcpxml_time(gap_len, fps)),
                ],
                text: None,
                children: Vec::new(),
            });
            current_frame = clip.start_frame;
        }

        let clip_node = build_fcpxml_clip_node(clip, primary_track, 0, asset_id_map, title_effect_id, fps, target);
        elements.push(clip_node);
        current_frame = clip.end_frame();
    }

    // Trailing gap on primary track if needed
    if current_frame < total_duration {
        let gap_len = total_duration - current_frame;
        elements.push(XMLNode {
            name: "gap".to_string(),
            attributes: vec![
                ("name".to_string(), "Gap".to_string()),
                ("offset".to_string(), format_fcpxml_time(current_frame, fps)),
                ("duration".to_string(), format_fcpxml_time(gap_len, fps)),
            ],
            text: None,
            children: Vec::new(),
        });
    }

    // Add connected clips from secondary tracks (lanes 1, 2, 3...)
    for (lane_idx, track) in timeline.tracks.iter().enumerate().skip(1) {
        let lane_number = if track.track_type == ClipType::Audio {
            -(lane_idx as i32)
        } else {
            lane_idx as i32
        };

        for clip in &track.clips {
            let connected_clip = build_fcpxml_clip_node(
                clip,
                track,
                lane_number,
                asset_id_map,
                title_effect_id,
                fps,
                target,
            );
            elements.push(connected_clip);
        }
    }

    elements
}

fn build_fcpxml_clip_node(
    clip: &Clip,
    track: &Track,
    lane: i32,
    asset_id_map: &HashMap<String, String>,
    title_effect_id: &str,
    fps: i32,
    target: FCPXMLTarget,
) -> XMLNode {
    let offset_str = format_fcpxml_time(clip.start_frame, fps);
    let dur_str = format_fcpxml_time(clip.duration_frames, fps);
    let in_str = format_fcpxml_time(clip.trim_start_frame, fps);
    let clip_name = &clip.media_ref;

    if clip.media_type == ClipType::Text {
        let mut title_attrs = vec![
            ("ref".to_string(), title_effect_id.to_string()),
            ("name".to_string(), clip_name.clone()),
            ("offset".to_string(), offset_str),
            ("duration".to_string(), dur_str),
            ("start".to_string(), in_str),
        ];
        if lane != 0 {
            title_attrs.push(("lane".to_string(), lane.to_string()));
        }

        let content = clip.text_content.clone().unwrap_or_default();
        let font_size = clip.text_style.as_ref().map(|s| s.font_size).unwrap_or(72.0);
        let font_family = clip.text_style.as_ref().map(|s| s.font_family.clone()).unwrap_or_else(|| "Inter".to_string());

        let text_style_id = format!("ts-{}", clip.id);
        let text_node = XMLNode {
            name: "text".to_string(),
            attributes: Vec::new(),
            text: None,
            children: vec![XMLNode {
                name: "text-style".to_string(),
                attributes: vec![("ref".to_string(), text_style_id.clone())],
                text: Some(content),
                children: Vec::new(),
            }],
        };

        let style_def = XMLNode {
            name: "text-style-def".to_string(),
            attributes: vec![("id".to_string(), text_style_id)],
            text: None,
            children: vec![XMLNode {
                name: "text-style".to_string(),
                attributes: vec![
                    ("font".to_string(), font_family),
                    ("fontSize".to_string(), format!("{:.0}", font_size)),
                    ("fontColor".to_string(), "1 1 1 1".to_string()),
                    ("alignment".to_string(), "center".to_string()),
                ],
                text: None,
                children: Vec::new(),
            }],
        };

        return XMLNode {
            name: "title".to_string(),
            attributes: title_attrs,
            text: None,
            children: vec![text_node, style_def],
        };
    }

    let asset_ref = asset_id_map
        .get(&clip.media_ref)
        .cloned()
        .unwrap_or_else(|| "r1".to_string());

    let mut clip_attrs = vec![
        ("ref".to_string(), asset_ref),
        ("name".to_string(), clip_name.clone()),
        ("offset".to_string(), offset_str),
        ("duration".to_string(), dur_str),
        ("start".to_string(), in_str),
    ];
    if lane != 0 {
        clip_attrs.push(("lane".to_string(), lane.to_string()));
    }

    let mut children = Vec::new();

    let (pos_x, pos_y) = match target {
        FCPXMLTarget::Resolve => {
            let px = (clip.transform.center_x - 0.5) * 100.0;
            let py = (clip.transform.center_y - 0.5) * 100.0;
            (px, py)
        }
        FCPXMLTarget::FinalCutPro => {
            ((clip.transform.center_x - 0.5) * 100.0, (clip.transform.center_y - 0.5) * 100.0)
        }
    };

    let scale_x = clip.transform.width;
    let scale_y = clip.transform.height;
    let rot_deg = -clip.transform.rotation;

    if pos_x.abs() > 0.001 || pos_y.abs() > 0.001 || (scale_x - 1.0).abs() > 0.001 || rot_deg.abs() > 0.001 {
        children.push(XMLNode {
            name: "adjust-transform".to_string(),
            attributes: vec![
                ("position".to_string(), format!("{:.2} {:.2}", pos_x, pos_y)),
                ("scale".to_string(), format!("{:.4} {:.4}", scale_x, scale_y)),
                ("rotation".to_string(), format!("{:.2}", rot_deg)),
            ],
            text: None,
            children: Vec::new(),
        });
    }

    if track.track_type == ClipType::Audio || clip.volume < 0.999 {
        let db = if clip.volume > 0.0001 {
            20.0 * clip.volume.log10()
        } else {
            -60.0
        };
        children.push(XMLNode {
            name: "adjust-volume".to_string(),
            attributes: vec![("amount".to_string(), format!("{:.1}dB", db))],
            text: None,
            children: Vec::new(),
        });
    }

    if (clip.speed - 1.0).abs() > 0.001 && clip.speed > 0.0 {
        let media_dur_frames = (clip.duration_frames as f64 * clip.speed).round() as i64;
        let media_dur_str = format_fcpxml_time(media_dur_frames, fps);
        children.push(XMLNode {
            name: "timeMap".to_string(),
            attributes: Vec::new(),
            text: None,
            children: vec![
                XMLNode {
                    name: "timept".to_string(),
                    attributes: vec![("value".to_string(), "0s".to_string()), ("time".to_string(), "0s".to_string())],
                    text: None,
                    children: Vec::new(),
                },
                XMLNode {
                    name: "timept".to_string(),
                    attributes: vec![
                        ("value".to_string(), media_dur_str),
                        ("time".to_string(), format_fcpxml_time(clip.duration_frames, fps)),
                    ],
                    text: None,
                    children: Vec::new(),
                },
            ],
        });
    }

    XMLNode {
        name: "asset-clip".to_string(),
        attributes: clip_attrs,
        text: None,
        children,
    }
}
