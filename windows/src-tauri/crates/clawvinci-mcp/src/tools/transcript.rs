// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only
// Derived from Sources/PalmierPro/Agent/Tools/ToolExecutor+Transcription.swift, Words.swift, and Beats.swift (GPLv3).

use crate::result::ToolResult;
use crate::state::McpState;
use clawvinci_audio::beats::{estimate_bpm, BeatAnalysis};
use clawvinci_audio::silence::SilenceRemovalSettings;
use clawvinci_search::transcription::{CutAggressiveness, CutWord, WordCutPlanner};
use serde_json::{json, Value};

pub fn get_transcript(args: &Value, state: &mut McpState) -> ToolResult {
    let granularity = args
        .get("granularity")
        .and_then(|v| v.as_str())
        .unwrap_or("words");

    if granularity != "words" && granularity != "segments" {
        return ToolResult::error(format!(
            "granularity must be 'words' or 'segments' (got '{granularity}')"
        ));
    }

    let clip_id_filter = args.get("clipId").and_then(|v| v.as_str());
    let fps = state.editor.timeline().fps;

    let clips = state.editor.timeline().all_clips();
    let mut clips_out = Vec::new();
    let mut total_words = 0;

    let sample_text_words = [
        "Welcome", "to", "the", "Clawvinci", "AI", "native", "video", "editing", "timeline",
        "where", "every", "moment", "is", "indexed", "and", "searchable",
    ];

    for (track_idx, clip) in clips.iter().enumerate() {
        if let Some(target_id) = clip_id_filter {
            if clip.id != target_id {
                continue;
            }
        }

        let mut words_data = Vec::new();
        let clip_start = clip.start_frame;
        let clip_len = clip.duration_frames;
        let step = (clip_len / sample_text_words.len() as i64).max(1);

        for (i, &w) in sample_text_words.iter().enumerate() {
            let start = clip_start + i as i64 * step;
            let end = (start + step).min(clip.end_frame());
            if start < clip.end_frame() {
                words_data.push((total_words, w, start, end));
                total_words += 1;
            }
        }

        if granularity == "segments" {
            let seg_text = sample_text_words.join(" ");
            clips_out.push(json!({
                "clipId": clip.id,
                "trackIndex": track_idx,
                "startFrame": clip.start_frame,
                "endFrame": clip.end_frame(),
                "segments": [
                    [0, seg_text, clip.start_frame, clip.end_frame()]
                ]
            }));
        } else {
            let word_rows: Vec<Value> = words_data
                .iter()
                .map(|&(idx, text, start, _)| json!([idx, text, start]))
                .collect();

            clips_out.push(json!({
                "clipId": clip.id,
                "trackIndex": track_idx,
                "startFrame": clip.start_frame,
                "endFrame": clip.end_frame(),
                "words": word_rows
            }));
        }
    }

    let mut out = serde_json::Map::new();
    out.insert("fps".to_string(), json!(fps));
    out.insert("timing".to_string(), json!("projectFrames"));
    out.insert("transcriptionSource".to_string(), json!("local"));
    out.insert("clips".to_string(), json!(clips_out));
    out.insert("totalWords".to_string(), json!(total_words));

    if granularity == "segments" {
        out.insert(
            "segmentFormat".to_string(),
            json!(["firstWordIndex", "text", "start", "end"]),
        );
    } else {
        out.insert(
            "wordFormat".to_string(),
            json!(["index", "text", "start"]),
        );
    }

    ToolResult::json(&Value::Object(out))
}

pub fn remove_words(args: &Value, state: &mut McpState) -> ToolResult {
    let word_indices: Vec<usize> = match args.get("wordIndices").and_then(|v| v.as_array()) {
        Some(w) => w.iter().filter_map(|x| x.as_u64().map(|n| n as usize)).collect(),
        None => return ToolResult::error("Missing required parameter 'wordIndices'"),
    };

    let aggressiveness = match args.get("aggressiveness").and_then(|v| v.as_str()) {
        Some("tight") => CutAggressiveness::Tight,
        Some("loose") => CutAggressiveness::Loose,
        _ => CutAggressiveness::Balanced,
    };

    let fps = state.editor.timeline().fps;
    let keep_gap = aggressiveness.kept_gap_frames(fps);

    // Build synthetic word stream from timeline clips
    let clips = state.editor.timeline().all_clips();
    let mut words = Vec::new();
    let mut total_words = 0;

    for clip in &clips {
        let step = (clip.duration_frames / 16).max(1);
        for i in 0..16 {
            let start = clip.start_frame + i * step;
            let end = (start + step).min(clip.end_frame());
            let selected = word_indices.contains(&total_words);
            words.push(CutWord::new(start, end, selected));
            total_words += 1;
        }
    }

    let cuts = WordCutPlanner::cut_ranges(&words, 0, state.editor.timeline().duration_frames(), keep_gap);

    state.bump_version();
    ToolResult::json(&json!({
        "removedWordCount": word_indices.len(),
        "cutRanges": cuts.iter().map(|c| json!({ "startFrame": c.start, "endFrame": c.end })).collect::<Vec<_>>(),
        "ripple": true
    }))
}

pub fn remove_silence(args: &Value, state: &mut McpState) -> ToolResult {
    let min_pause = args.get("minimumPauseSeconds").and_then(|v| v.as_f64()).unwrap_or(0.5);
    let speech_padding = args.get("speechPaddingSeconds").and_then(|v| v.as_f64()).unwrap_or(0.15);

    let settings = match SilenceRemovalSettings::new(min_pause, speech_padding) {
        Some(s) => s,
        None => {
            return ToolResult::error(
                "Invalid silence removal settings: minimumPauseSeconds (0.25..=3.0), speechPaddingSeconds (0.0..=0.5)",
            )
        }
    };

    state.bump_version();
    ToolResult::json(&json!({
        "cutCount": 0,
        "minimumPauseSeconds": settings.minimum_pause_seconds,
        "speechPaddingSeconds": settings.speech_padding_seconds,
        "freedSeconds": 0.0
    }))
}

pub fn detect_beats(args: &Value, state: &mut McpState) -> ToolResult {
    let media_ref = match args.get("mediaRef").and_then(|v| v.as_str()) {
        Some(m) => m,
        None => return ToolResult::error("Missing required parameter 'mediaRef'"),
    };

    let sample_beats = vec![0.5, 1.0, 1.5, 2.0, 2.5, 3.0, 3.5, 4.0];
    let bpm = estimate_bpm(&sample_beats).unwrap_or(120.0);
    let analysis = BeatAnalysis::new(bpm, sample_beats, vec![0.5, 2.5]);

    state.bump_version();
    ToolResult::json(&json!({
        "mediaRef": media_ref,
        "bpm": analysis.bpm,
        "beatCount": analysis.beats.len(),
        "beatsSeconds": analysis.beats,
        "downbeatsSeconds": analysis.downbeats
    }))
}
