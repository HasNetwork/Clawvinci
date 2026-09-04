// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only
// Derived from Sources/PalmierPro/Models/TextLayout.swift (GPLv3).

use crate::text_style::TextStyle;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TextSize {
    pub width: f64,
    pub height: f64,
}

pub struct TextLayout;

impl TextLayout {
    pub const REFERENCE_CANVAS_HEIGHT: f64 = 1080.0;
    pub const SHADOW_PADDING: f64 = 12.0;

    /// Calculates natural bounding size of rendered text based on text attributes.
    pub fn natural_size(
        content: &str,
        style: &TextStyle,
        max_width: f64,
        canvas_height: f64,
    ) -> TextSize {
        let visual_scale = style.font_scale;
        let canvas_scale = canvas_height / Self::REFERENCE_CANVAS_HEIGHT;
        let render_size = style.font_size * canvas_scale;

        // Base glyph measurement approximation (avg glyph width ~0.55 * fontSize)
        let char_count = content
            .lines()
            .map(|l| l.chars().count())
            .max()
            .unwrap_or(1)
            .max(1);
        let line_count = content.lines().count().max(1);

        let approx_glyph_w = render_size * 0.55 * style.width_scale + style.tracking;
        let approx_line_h = render_size * 1.2 * style.height_scale + style.line_spacing;

        let raw_w = char_count as f64 * approx_glyph_w;
        let raw_h = line_count as f64 * approx_line_h;

        let proposed_max_w = max_width * visual_scale;
        let bounded_w = if proposed_max_w.is_finite() && proposed_max_w > 0.0 {
            raw_w.min(proposed_max_w)
        } else {
            raw_w
        };

        let slack = visual_scale.max(0.0) * 4.0;
        let shadow_blur = style.shadow.blur.max(0.0);
        let shadow_x = if style.shadow.enabled {
            (Self::SHADOW_PADDING * visual_scale).max(shadow_blur + style.shadow.offset_x.abs())
                * canvas_scale
                * 2.0
        } else {
            0.0
        };
        let shadow_y = if style.shadow.enabled {
            (Self::SHADOW_PADDING * visual_scale).max(shadow_blur + style.shadow.offset_y.abs())
                * canvas_scale
                * 2.0
        } else {
            0.0
        };

        let border_pad = if style.border.enabled {
            style.border.width * canvas_scale * 2.0
        } else {
            0.0
        };
        let bg_pad_x = if style.background.enabled {
            style.background.padding_x.max(0.0) * canvas_scale * 2.0
        } else {
            0.0
        };
        let bg_pad_y = if style.background.enabled {
            style.background.padding_y.max(0.0) * canvas_scale * 2.0
        } else {
            0.0
        };

        TextSize {
            width: (bounded_w.ceil() + shadow_x + border_pad + bg_pad_x + slack).max(1.0),
            height: (raw_h.ceil() + shadow_y + border_pad + bg_pad_y + slack).max(1.0),
        }
    }
}
