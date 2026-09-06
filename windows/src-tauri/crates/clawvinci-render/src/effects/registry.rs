// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only
// Derived from Sources/PalmierPro/Compositing/EffectRegistry.swift (GPLv3).

use super::blur::{apply_gaussian_blur, apply_motion_blur, apply_noise_reduction, apply_sharpen};
use super::chroma_key::apply_chroma_key;
use super::clarity::apply_clarity;
use super::color_space::{convert_linear_to_srgb, convert_srgb_to_linear};
use super::color_wheels::apply_color_wheels;
use super::contrast::apply_contrast;
use super::exposure::apply_exposure;
use super::glow::apply_glow;
use super::grade_curves::apply_grade_curves;
use super::grain::apply_grain;
use super::highlights_shadows::apply_highlights_shadows;
use super::hue_curves::apply_hue_curves;
use super::invert::apply_invert;
use super::levels::apply_levels;
use super::lut::apply_lut;
use super::saturation::apply_saturation;
use super::temperature::apply_temperature;
use super::types::{EffectDescriptor, EffectParamSpec, ResolvedEffectParams};
use super::vibrance::apply_vibrance;
use super::vignette::apply_vignette;
use clawvinci_model::effect::Effect;

pub const CANONICAL_ORDER: &[&str] = &[
    "color.exposure",
    "color.contrast",
    "color.highlightsShadows",
    "color.blacksWhites",
    "color.temperature",
    "color.vibrance",
    "color.saturation",
    "color.wheels",
    "color.curves",
    "color.hueCurves",
    "color.lut",
    "detail.clarity",
    "key.chroma",
    "blur.gaussian",
    "blur.sharpen",
    "blur.noiseReduction",
    "blur.motion",
    "stylize.invert",
    "stylize.grain",
    "stylize.vignette",
    "stylize.glow",
];

// Parameter specs matching EffectRegistry.swift exactly
static EXPOSURE_PARAMS: &[EffectParamSpec] = &[EffectParamSpec::new(
    "ev", "Exposure", -3.0, 3.0, 0.0, "",
)];

static CONTRAST_PARAMS: &[EffectParamSpec] = &[EffectParamSpec::new(
    "amount", "Contrast", 0.5, 1.5, 1.0, "",
)];

static SATURATION_PARAMS: &[EffectParamSpec] = &[EffectParamSpec::new(
    "amount", "Saturation", 0.0, 2.0, 1.0, "",
)];

static TEMPERATURE_PARAMS: &[EffectParamSpec] = &[
    EffectParamSpec::new("temperature", "Temperature", 2000.0, 11000.0, 6500.0, "K"),
    EffectParamSpec::new("tint", "Tint", -100.0, 100.0, 0.0, ""),
];

static HIGHLIGHTS_SHADOWS_PARAMS: &[EffectParamSpec] = &[
    EffectParamSpec::new("highlights", "Highlights", -1.0, 1.0, 0.0, ""),
    EffectParamSpec::new("shadows", "Shadows", -1.0, 1.0, 0.0, ""),
];

static LEVELS_PARAMS: &[EffectParamSpec] = &[
    EffectParamSpec::new("blacks", "Blacks", -1.0, 1.0, 0.0, ""),
    EffectParamSpec::new("whites", "Whites", -1.0, 1.0, 0.0, ""),
];

static VIBRANCE_PARAMS: &[EffectParamSpec] = &[EffectParamSpec::new(
    "amount", "Vibrance", -1.0, 1.0, 0.0, "",
)];

static WHEELS_PARAMS: &[EffectParamSpec] = &[
    EffectParamSpec::new("lift_x", "Lift", -1.0, 1.0, 0.0, ""),
    EffectParamSpec::new("lift_y", "Lift", -1.0, 1.0, 0.0, ""),
    EffectParamSpec::new("lift_m", "Lift", -0.5, 0.5, 0.0, ""),
    EffectParamSpec::new("gamma_x", "Gamma", -1.0, 1.0, 0.0, ""),
    EffectParamSpec::new("gamma_y", "Gamma", -1.0, 1.0, 0.0, ""),
    EffectParamSpec::new("gamma_m", "Gamma", 0.5, 2.0, 1.0, ""),
    EffectParamSpec::new("gain_x", "Gain", -1.0, 1.0, 0.0, ""),
    EffectParamSpec::new("gain_y", "Gain", -1.0, 1.0, 0.0, ""),
    EffectParamSpec::new("gain_m", "Gain", 0.5, 1.5, 1.0, ""),
];

static LUT_PARAMS: &[EffectParamSpec] = &[EffectParamSpec::new(
    "intensity",
    "Intensity",
    0.0,
    1.0,
    1.0,
    "",
)];

static GAUSSIAN_BLUR_PARAMS: &[EffectParamSpec] = &[EffectParamSpec::new(
    "radius", "Radius", 0.0, 100.0, 8.0, "px",
)];

static SHARPEN_PARAMS: &[EffectParamSpec] = &[EffectParamSpec::new(
    "amount",
    "Sharpness",
    0.0,
    2.0,
    0.4,
    "",
)];

static NOISE_REDUCTION_PARAMS: &[EffectParamSpec] = &[EffectParamSpec::new(
    "amount",
    "Noise Reduction",
    0.0,
    1.0,
    0.0,
    "",
)];

static MOTION_BLUR_PARAMS: &[EffectParamSpec] = &[
    EffectParamSpec::new("radius", "Motion Blur", 0.0, 100.0, 0.0, "px"),
    EffectParamSpec::new("angle", "Angle", -180.0, 180.0, 0.0, "°"),
];

static GRAIN_PARAMS: &[EffectParamSpec] = &[
    EffectParamSpec::new("amount", "Amount", 0.0, 1.0, 0.0, ""),
    EffectParamSpec::new("size", "Size", 0.5, 4.0, 1.5, ""),
];

static VIGNETTE_PARAMS: &[EffectParamSpec] = &[
    EffectParamSpec::new("amount", "Amount", -1.0, 1.0, 0.0, ""),
    EffectParamSpec::new("midpoint", "Midpoint", 0.0, 1.0, 0.5, ""),
    EffectParamSpec::new("roundness", "Roundness", -1.0, 1.0, 0.0, ""),
    EffectParamSpec::new("feather", "Feather", 0.0, 1.0, 0.5, ""),
];

static GLOW_PARAMS: &[EffectParamSpec] = &[
    EffectParamSpec::new("intensity", "Glow", 0.0, 1.0, 0.0, ""),
    EffectParamSpec::new("radius", "Radius", 0.0, 100.0, 20.0, "px"),
    EffectParamSpec::new("threshold", "Threshold", 0.0, 1.0, 0.6, ""),
    EffectParamSpec::new("warmth", "Warmth", 0.0, 1.0, 0.0, ""),
];

static CLARITY_PARAMS: &[EffectParamSpec] = &[
    EffectParamSpec::new("clarity", "Clarity", -1.0, 1.0, 0.0, ""),
    EffectParamSpec::new("dehaze", "Dehaze", -1.0, 1.0, 0.0, ""),
];

static CHROMA_KEY_PARAMS: &[EffectParamSpec] = &[
    EffectParamSpec::new("keyHue", "Key Hue", 0.0, 1.0, 0.333, ""),
    EffectParamSpec::new("tolerance", "Tolerance", 0.0, 1.0, 0.0, ""),
    EffectParamSpec::new("softness", "Softness", 0.0, 1.0, 0.1, ""),
    EffectParamSpec::new("spill", "Spill", 0.0, 1.0, 0.5, ""),
];

static NO_PARAMS: &[EffectParamSpec] = &[];

pub static ALL_EFFECTS: &[EffectDescriptor] = &[
    EffectDescriptor::new("color.exposure", "Exposure", "Color", EXPOSURE_PARAMS, true, None, apply_exposure),
    EffectDescriptor::new("color.contrast", "Contrast", "Color", CONTRAST_PARAMS, false, None, apply_contrast),
    EffectDescriptor::new("color.highlightsShadows", "Highlights & Shadows", "Color", HIGHLIGHTS_SHADOWS_PARAMS, false, None, apply_highlights_shadows),
    EffectDescriptor::new("color.blacksWhites", "Levels", "Color", LEVELS_PARAMS, false, None, apply_levels),
    EffectDescriptor::new("color.temperature", "Temperature & Tint", "Color", TEMPERATURE_PARAMS, false, None, apply_temperature),
    EffectDescriptor::new("color.vibrance", "Vibrance", "Color", VIBRANCE_PARAMS, false, None, apply_vibrance),
    EffectDescriptor::new("color.saturation", "Saturation", "Color", SATURATION_PARAMS, false, None, apply_saturation),
    EffectDescriptor::new("color.wheels", "Color Wheels", "Color", WHEELS_PARAMS, false, None, apply_color_wheels),
    EffectDescriptor::new("color.curves", "Curves", "Color", NO_PARAMS, false, None, apply_grade_curves),
    EffectDescriptor::new("color.hueCurves", "Hue Curves", "Color", NO_PARAMS, false, None, apply_hue_curves),
    EffectDescriptor::new("color.lut", "LUT", "Color", LUT_PARAMS, false, Some("path"), apply_lut),
    EffectDescriptor::new("detail.clarity", "Clarity & Haze", "Detail", CLARITY_PARAMS, false, None, apply_clarity),
    EffectDescriptor::new("key.chroma", "Chroma Key", "Key", CHROMA_KEY_PARAMS, false, None, apply_chroma_key),
    EffectDescriptor::new("blur.gaussian", "Gaussian Blur", "Blur & Sharpen", GAUSSIAN_BLUR_PARAMS, false, None, apply_gaussian_blur),
    EffectDescriptor::new("blur.sharpen", "Sharpen", "Blur & Sharpen", SHARPEN_PARAMS, false, None, apply_sharpen),
    EffectDescriptor::new("blur.noiseReduction", "Noise Reduction", "Blur & Sharpen", NOISE_REDUCTION_PARAMS, false, None, apply_noise_reduction),
    EffectDescriptor::new("blur.motion", "Motion Blur", "Blur & Sharpen", MOTION_BLUR_PARAMS, false, None, apply_motion_blur),
    EffectDescriptor::new("stylize.invert", "Invert", "Stylize", NO_PARAMS, false, None, apply_invert),
    EffectDescriptor::new("stylize.grain", "Film Grain", "Stylize", GRAIN_PARAMS, false, None, apply_grain),
    EffectDescriptor::new("stylize.vignette", "Vignette", "Stylize", VIGNETTE_PARAMS, false, None, apply_vignette),
    EffectDescriptor::new("stylize.glow", "Glow", "Stylize", GLOW_PARAMS, false, None, apply_glow),
];

pub struct EffectRegistry;

impl EffectRegistry {
    pub fn all() -> &'static [EffectDescriptor] {
        ALL_EFFECTS
    }

    pub fn descriptor(id: &str) -> Option<&'static EffectDescriptor> {
        ALL_EFFECTS.iter().find(|d| d.id == id)
    }

    pub fn canonical_order() -> &'static [&'static str] {
        CANONICAL_ORDER
    }

    pub fn insert_index(effects: &[Effect], id: &str) -> usize {
        let rank = CANONICAL_ORDER
            .iter()
            .position(|&k| k == id)
            .unwrap_or(usize::MAX);
        effects
            .iter()
            .position(|e| {
                let e_rank = CANONICAL_ORDER
                    .iter()
                    .position(|&k| k == e.effect_type)
                    .unwrap_or(usize::MAX);
                e_rank > rank
            })
            .unwrap_or(effects.len())
    }

    pub fn resolve_params(
        desc: &EffectDescriptor,
        effect: &Effect,
        offset: usize,
        spatial_scale: f64,
    ) -> ResolvedEffectParams {
        let mut values = std::collections::HashMap::new();
        for spec in desc.params {
            let raw = effect
                .params
                .get(spec.key)
                .map(|p| p.resolved(offset as i64, spec.default_value))
                .unwrap_or(spec.default_value);
            values.insert(spec.key.to_string(), spec.clamp(raw));
        }

        let mut strings = std::collections::HashMap::new();
        for (k, v) in &effect.params {
            if let Some(s) = &v.string {
                strings.insert(k.clone(), s.clone());
            }
        }

        ResolvedEffectParams::new(values, strings, offset, spatial_scale)
    }

    pub fn render_effect(
        desc: &EffectDescriptor,
        pixels: &mut [u8],
        width: u32,
        height: u32,
        params: &ResolvedEffectParams,
    ) {
        if desc.linearizes {
            convert_srgb_to_linear(pixels);
        }
        (desc.apply)(pixels, width, height, params);
        if desc.linearizes {
            convert_linear_to_srgb(pixels);
        }
    }
}
