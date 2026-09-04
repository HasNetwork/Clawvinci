// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only
// Derived from Sources/PalmierPro/Models/TextStyle.swift (GPLv3).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Rgba {
    #[serde(default = "default_one")]
    pub r: f64,
    #[serde(default = "default_one")]
    pub g: f64,
    #[serde(default = "default_one")]
    pub b: f64,
    #[serde(default = "default_one")]
    pub a: f64,
}

fn default_one() -> f64 {
    1.0
}

impl Default for Rgba {
    fn default() -> Self {
        Self {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 1.0,
        }
    }
}

impl Rgba {
    pub fn new(r: f64, g: f64, b: f64, a: f64) -> Self {
        Self { r, g, b, a }
    }

    pub fn from_hex(hex: &str) -> Option<Self> {
        let hex = hex.trim().trim_start_matches('#');
        match hex.len() {
            6 => {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()? as f64 / 255.0;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()? as f64 / 255.0;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()? as f64 / 255.0;
                Some(Self::new(r, g, b, 1.0))
            }
            8 => {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()? as f64 / 255.0;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()? as f64 / 255.0;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()? as f64 / 255.0;
                let a = u8::from_str_radix(&hex[6..8], 16).ok()? as f64 / 255.0;
                Some(Self::new(r, g, b, a))
            }
            _ => None,
        }
    }

    pub fn to_hex(&self) -> String {
        let r = (self.r.clamp(0.0, 1.0) * 255.0).round() as u8;
        let g = (self.g.clamp(0.0, 1.0) * 255.0).round() as u8;
        let b = (self.b.clamp(0.0, 1.0) * 255.0).round() as u8;
        format!("#{:02X}{:02X}{:02X}", r, g, b)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum TextAlignment {
    Left,
    #[default]
    Center,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum FontCase {
    #[default]
    Mixed,
    Uppercase,
    Lowercase,
}

impl FontCase {
    pub fn apply(&self, text: &str) -> String {
        match self {
            Self::Mixed => text.to_string(),
            Self::Uppercase => text.to_uppercase(),
            Self::Lowercase => text.to_lowercase(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextShadow {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_shadow_color")]
    pub color: Rgba,
    #[serde(default)]
    pub offset_x: f64,
    #[serde(default = "default_shadow_offset_y")]
    pub offset_y: f64,
    #[serde(default = "default_shadow_blur")]
    pub blur: f64,
}

fn default_shadow_color() -> Rgba {
    Rgba::new(0.0, 0.0, 0.0, 0.6)
}
fn default_shadow_offset_y() -> f64 {
    -2.0
}
fn default_shadow_blur() -> f64 {
    6.0
}

impl Default for TextShadow {
    fn default() -> Self {
        Self {
            enabled: false,
            color: default_shadow_color(),
            offset_x: 0.0,
            offset_y: default_shadow_offset_y(),
            blur: default_shadow_blur(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextOutline {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_outline_color")]
    pub color: Rgba,
    #[serde(default = "default_outline_width")]
    pub width: f64,
}

fn default_outline_color() -> Rgba {
    Rgba::new(0.0, 0.0, 0.0, 1.0)
}
fn default_outline_width() -> f64 {
    4.0
}

impl Default for TextOutline {
    fn default() -> Self {
        Self {
            enabled: false,
            color: default_outline_color(),
            width: default_outline_width(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextBackground {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_shadow_color")]
    pub color: Rgba,
    #[serde(default)]
    pub padding_x: f64,
    #[serde(default)]
    pub padding_y: f64,
    #[serde(default)]
    pub corner_radius: f64,
    #[serde(default)]
    pub offset_x: f64,
    #[serde(default)]
    pub offset_y: f64,
    #[serde(default = "default_outline_color")]
    pub outline_color: Rgba,
    #[serde(default)]
    pub outline_width: f64,
}

impl Default for TextBackground {
    fn default() -> Self {
        Self {
            enabled: false,
            color: default_shadow_color(),
            padding_x: 0.0,
            padding_y: 0.0,
            cornerRadius: 0.0,
            offset_x: 0.0,
            offset_y: 0.0,
            outline_color: default_outline_color(),
            outline_width: 0.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextStyle {
    #[serde(default = "default_font_name")]
    pub font_name: String,
    #[serde(default = "default_font_size")]
    pub font_size: f64,
    #[serde(default = "default_one")]
    pub font_scale: f64,
    #[serde(default = "default_one")]
    pub width_scale: f64,
    #[serde(default = "default_one")]
    pub height_scale: f64,
    #[serde(default)]
    pub tracking: f64,
    #[serde(default)]
    pub line_spacing: f64,
    #[serde(default)]
    pub font_case: FontCase,
    #[serde(default)]
    pub is_bold: bool,
    #[serde(default)]
    pub is_italic: bool,
    #[serde(default)]
    pub is_underlined: bool,
    #[serde(default)]
    pub is_struck_through: bool,
    #[serde(default)]
    pub is_overlined: bool,
    #[serde(default)]
    pub color: Rgba,
    #[serde(default)]
    pub alignment: TextAlignment,
    #[serde(default)]
    pub blur: f64,
    #[serde(default)]
    pub shadow: TextShadow,
    #[serde(default)]
    pub background: TextBackground,
    #[serde(default)]
    pub border: TextOutline,
}

fn default_font_name() -> String {
    "Helvetica".to_string()
}
fn default_font_size() -> f64 {
    96.0
}

impl Default for TextStyle {
    fn default() -> Self {
        Self {
            font_name: default_font_name(),
            font_size: default_font_size(),
            font_scale: 1.0,
            width_scale: 1.0,
            height_scale: 1.0,
            tracking: 0.0,
            line_spacing: 0.0,
            font_case: FontCase::Mixed,
            is_bold: false,
            is_italic: false,
            is_underlined: false,
            is_struck_through: false,
            is_overlined: false,
            color: Rgba::default(),
            alignment: TextAlignment::Center,
            blur: 0.0,
            shadow: TextShadow::default(),
            background: TextBackground::default(),
            border: TextOutline::default(),
        }
    }
}
