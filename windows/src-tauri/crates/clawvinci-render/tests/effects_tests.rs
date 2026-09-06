// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only

use clawvinci_model::clip_type::ClipType;
use clawvinci_model::effect::Effect;
use clawvinci_model::grade::{GradeCurve, HueCurves};
use clawvinci_model::text_animation::{AnimationPreset, TextAnimation};
use clawvinci_model::text_style::{Rgba, TextStyle};
use clawvinci_model::timeline::{Clip, Timeline, Track};
use clawvinci_render::compositor::composite_frame;
use clawvinci_render::effects::lut::interpolate_tetrahedral;
use clawvinci_render::effects::lut_loader::LutLoader;
use clawvinci_render::effects::types::ResolvedEffectParams;
use clawvinci_render::effects::EffectRegistry;
use clawvinci_render::plan::CompositionBuilder;
use clawvinci_render::text::{TextAnimator, TextRenderer};
use std::collections::HashMap;

#[test]
fn test_canonical_order_and_registry() {
    let order = EffectRegistry::canonical_order();
    assert_eq!(order.len(), 21);
    assert_eq!(order[0], "color.exposure");
    assert_eq!(order[1], "color.contrast");
    assert_eq!(order[2], "color.highlightsShadows");
    assert_eq!(order[3], "color.blacksWhites");
    assert_eq!(order[4], "color.temperature");
    assert_eq!(order[5], "color.vibrance");
    assert_eq!(order[6], "color.saturation");
    assert_eq!(order[7], "color.wheels");
    assert_eq!(order[8], "color.curves");
    assert_eq!(order[9], "color.hueCurves");
    assert_eq!(order[10], "color.lut");
    assert_eq!(order[11], "detail.clarity");
    assert_eq!(order[12], "key.chroma");
    assert_eq!(order[13], "blur.gaussian");
    assert_eq!(order[14], "blur.sharpen");
    assert_eq!(order[15], "blur.noiseReduction");
    assert_eq!(order[16], "blur.motion");
    assert_eq!(order[17], "stylize.invert");
    assert_eq!(order[18], "stylize.grain");
    assert_eq!(order[19], "stylize.vignette");
    assert_eq!(order[20], "stylize.glow");

    for id in order {
        let desc = EffectRegistry::descriptor(id);
        assert!(desc.is_some(), "missing descriptor for {}", id);
        assert_eq!(desc.unwrap().id, *id);
    }
}

#[test]
fn test_insert_index_ordering() {
    let effects = vec![
        Effect::new("color.exposure"),
        Effect::new("color.wheels"),
        Effect::new("stylize.vignette"),
    ];

    // Contrast should insert after exposure (index 1)
    let idx_contrast = EffectRegistry::insert_index(&effects, "color.contrast");
    assert_eq!(idx_contrast, 1);

    // Glow should insert after vignette (index 3)
    let idx_glow = EffectRegistry::insert_index(&effects, "stylize.glow");
    assert_eq!(idx_glow, 3);
}

#[test]
fn test_levels_math() {
    let desc = EffectRegistry::descriptor("color.blacksWhites").unwrap();
    let mut pixels = [128u8, 128, 128, 255]; // 0.50196
    let mut values = HashMap::new();
    values.insert("blacks".to_string(), 0.2);
    values.insert("whites".to_string(), 0.3);
    let params = ResolvedEffectParams::new(values, HashMap::new(), 0, 1.0);

    (desc.apply)(&mut pixels, 1, 1, &params);

    // Hand calculation:
    // r = 128 / 255 = 0.5019608
    // bp = -0.2 * 0.4 = -0.08
    // wp = 1.0 - 0.3 * 0.4 = 0.88
    // denom = 0.88 - (-0.08) = 0.96
    // out = (0.5019608 - (-0.08)) / 0.96 = 0.5819608 / 0.96 = 0.606209
    // byte = round(0.606209 * 255) = 155
    assert_eq!(pixels[0], 155);
    assert_eq!(pixels[1], 155);
    assert_eq!(pixels[2], 155);
    assert_eq!(pixels[3], 255);
}

#[test]
fn test_highlights_shadows_math() {
    let desc = EffectRegistry::descriptor("color.highlightsShadows").unwrap();
    let mut pixels = [204u8, 204, 204, 255]; // 0.8
    let mut values = HashMap::new();
    values.insert("highlights".to_string(), 0.5);
    values.insert("shadows".to_string(), 0.0);
    let params = ResolvedEffectParams::new(values, HashMap::new(), 0, 1.0);

    (desc.apply)(&mut pixels, 1, 1, &params);

    // y = 0.8; hi = 0.512; lo = 0.008; dY = (0.5 * 0.512) * 0.5 = 0.128
    // out = 0.8 + 0.128 = 0.928 -> round(0.928 * 255) = 237
    assert_eq!(pixels[0], 237);
    assert_eq!(pixels[1], 237);
    assert_eq!(pixels[2], 237);
}

#[test]
fn test_invert() {
    let desc = EffectRegistry::descriptor("stylize.invert").unwrap();
    let mut pixels = [100u8, 150, 200, 255];
    let params = ResolvedEffectParams::default();

    (desc.apply)(&mut pixels, 1, 1, &params);

    assert_eq!(pixels[0], 155);
    assert_eq!(pixels[1], 105);
    assert_eq!(pixels[2], 55);
    assert_eq!(pixels[3], 255);
}

#[test]
fn test_saturation_grayscale() {
    let desc = EffectRegistry::descriptor("color.saturation").unwrap();
    let mut pixels = [255u8, 0, 0, 255]; // Pure red
    let mut values = HashMap::new();
    values.insert("amount".to_string(), 0.0); // Desaturate completely
    let params = ResolvedEffectParams::new(values, HashMap::new(), 0, 1.0);

    (desc.apply)(&mut pixels, 1, 1, &params);

    // Rec.709 luma for pure red is 0.2126 * 255 = 54
    assert_eq!(pixels[0], 54);
    assert_eq!(pixels[1], 54);
    assert_eq!(pixels[2], 54);
}

#[test]
fn test_chroma_key() {
    let desc = EffectRegistry::descriptor("key.chroma").unwrap();
    // Green pixel
    let mut pixels = [20u8, 220, 30, 255];
    let mut values = HashMap::new();
    values.insert("keyHue".to_string(), 0.333); // Green hue
    values.insert("tolerance".to_string(), 0.8);
    values.insert("softness".to_string(), 0.1);
    values.insert("spill".to_string(), 0.5);
    let params = ResolvedEffectParams::new(values, HashMap::new(), 0, 1.0);

    (desc.apply)(&mut pixels, 1, 1, &params);

    // Green pixel keyed out -> alpha near 0
    assert!(pixels[3] < 30, "expected keyed out alpha, got {}", pixels[3]);
}

#[test]
fn test_edge_rounding() {
    use clawvinci_render::effects::apply_edge_rounding;

    let mut pixels = vec![255u8; 100 * 100 * 4];
    apply_edge_rounding(&mut pixels, 100, 100, 0.5, 0.0);

    // Center pixel (50, 50) must retain alpha 255
    let center_idx = (50 * 100 + 50) * 4;
    assert_eq!(pixels[center_idx + 3], 255);

    // Top-left corner (0, 0) must be clipped to alpha 0
    let corner_idx = 0;
    assert_eq!(pixels[corner_idx + 3], 0);
}

#[test]
fn test_vignette() {
    let desc = EffectRegistry::descriptor("stylize.vignette").unwrap();
    let mut pixels = vec![200u8; 100 * 100 * 4];
    let mut values = HashMap::new();
    values.insert("amount".to_string(), -0.8); // Darken edges
    values.insert("midpoint".to_string(), 0.3);
    values.insert("feather".to_string(), 0.3);
    let params = ResolvedEffectParams::new(values, HashMap::new(), 0, 1.0);

    (desc.apply)(&mut pixels, 100, 100, &params);

    // Center pixel unchanged
    let center_idx = (50 * 100 + 50) * 4;
    assert_eq!(pixels[center_idx], 200);

    // Corner pixel significantly darkened
    let corner_idx = 0;
    assert!(pixels[corner_idx] < 120, "expected darkened corner, got {}", pixels[corner_idx]);
}

#[test]
fn test_grain_midtones() {
    let desc = EffectRegistry::descriptor("stylize.grain").unwrap();
    // Test mid-tone pixel (luma = 0.5) vs pure black pixel (luma = 0.0)
    let mut pixels = [128u8, 128, 128, 255, 0, 0, 0, 255];
    let mut values = HashMap::new();
    values.insert("amount".to_string(), 1.0);
    let params = ResolvedEffectParams::new(values, HashMap::new(), 42, 1.0);

    (desc.apply)(&mut pixels, 2, 1, &params);

    // Mid-tone pixel should have noise added (shifted from 128)
    assert_ne!(pixels[0], 128);

    // Pure black pixel has lumaMask = 4 * 0 * 1 = 0, so remains exactly 0
    assert_eq!(pixels[4], 0);
    assert_eq!(pixels[5], 0);
    assert_eq!(pixels[6], 0);
}

#[test]
fn test_lut_parsing_and_tetrahedral_interpolation() {
    let cube_text = r#"
# Sample 2x2x2 identity cube
TITLE "Test Identity"
LUT_3D_SIZE 2
DOMAIN_MIN 0.0 0.0 0.0
DOMAIN_MAX 1.0 1.0 1.0
0.0 0.0 0.0
1.0 0.0 0.0
0.0 1.0 0.0
1.0 1.0 0.0
0.0 0.0 1.0
1.0 0.0 1.0
0.0 1.0 1.0
1.0 1.0 1.0
"#;

    let lut = LutLoader::parse(cube_text).expect("failed to parse .cube LUT");
    assert_eq!(lut.dimension, 2);

    // Identity LUT tetrahedral interpolation should return input
    let out = interpolate_tetrahedral(&lut, [0.35, 0.65, 0.85]);
    assert!((out[0] - 0.35).abs() < 1e-4);
    assert!((out[1] - 0.65).abs() < 1e-4);
    assert!((out[2] - 0.85).abs() < 1e-4);
}

#[test]
fn test_grade_curves_identity() {
    let desc = EffectRegistry::descriptor("color.curves").unwrap();
    let mut pixels = [100u8, 150, 200, 255];
    let curve = GradeCurve::default();
    let json = serde_json::to_string(&curve).unwrap();

    let mut strings = HashMap::new();
    strings.insert("curve".to_string(), json);
    let params = ResolvedEffectParams::new(HashMap::new(), strings, 0, 1.0);

    (desc.apply)(&mut pixels, 1, 1, &params);

    assert_eq!(pixels[0], 100);
    assert_eq!(pixels[1], 150);
    assert_eq!(pixels[2], 200);
}

#[test]
fn test_hue_curves_identity() {
    let desc = EffectRegistry::descriptor("color.hueCurves").unwrap();
    let mut pixels = [100u8, 150, 200, 255];
    let curves = HueCurves::default();
    let json = serde_json::to_string(&curves).unwrap();

    let mut strings = HashMap::new();
    strings.insert("curves".to_string(), json);
    let params = ResolvedEffectParams::new(HashMap::new(), strings, 0, 1.0);

    (desc.apply)(&mut pixels, 1, 1, &params);

    assert_eq!(pixels[0], 100);
    assert_eq!(pixels[1], 150);
    assert_eq!(pixels[2], 200);
}

#[test]
fn test_text_animator() {
    let anim = TextAnimation {
        preset: AnimationPreset::PopIn,
        per_word_frames: 10,
        highlight: None,
    };

    // Frame 0: t=0 -> opacity=0, scale=0.6
    let state0 = TextAnimator::clip_entry(&anim, 0);
    assert_eq!(state0.opacity, 0.0);
    assert!((state0.scale - 0.6).abs() < 1e-4);

    // Frame 10: t=1 -> opacity=1, scale=1.0
    let state10 = TextAnimator::clip_entry(&anim, 10);
    assert_eq!(state10.opacity, 1.0);
    assert!((state10.scale - 1.0).abs() < 1e-4);
}

#[test]
fn test_text_rendering() {
    let style = TextStyle {
        font_size: 48.0,
        color: Rgba::new(1.0, 1.0, 1.0, 1.0),
        ..Default::default()
    };

    let frame = TextRenderer::render_text("Hello Clawvinci", &style, None, 0, 320, 180)
        .expect("text render should succeed");

    assert_eq!(frame.width, 320);
    assert_eq!(frame.height, 180);

    // Confirm non-zero alpha pixels were drawn
    let has_visible_pixels = frame.data.chunks_exact(4).any(|p| p[3] > 0);
    assert!(has_visible_pixels, "text frame should contain rendered glyphs");
}

#[test]
fn test_full_pipeline_with_effects_and_rounding() {
    let mut timeline = Timeline::new("test_pipeline", 320, 180, 30);
    let mut clip = Clip::new("clip_1", "solid_1", ClipType::SolidColor, 0, 30);
    clip.edge_rounding = 0.5;

    // Add an exposure effect
    let mut effect = Effect::new("color.exposure");
    effect.params.insert(
        "ev".to_string(),
        clawvinci_model::effect::EffectParam::from_value(1.0),
    );
    clip.effects = Some(vec![effect]);

    let mut track = Track::new("t1", "Track 1", ClipType::Video);
    track.clips.push(clip);
    timeline.tracks.push(track);

    let plan = CompositionBuilder::build_frame_plan(&timeline, 0)
        .expect("should build plan");

    assert_eq!(plan.layers.len(), 1);
    assert_eq!(plan.layers[0].effects.len(), 1);
    assert_eq!(plan.layers[0].effects[0].effect_type, "color.exposure");
    assert_eq!(plan.layers[0].edge_rounding, 0.5);

    let rendered = composite_frame(&plan, &HashMap::new())
        .expect("should composite frame");

    assert_eq!(rendered.width, 320);
    assert_eq!(rendered.height, 180);
}
