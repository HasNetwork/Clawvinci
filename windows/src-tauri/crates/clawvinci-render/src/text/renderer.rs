// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only
// Derived from Sources/PalmierPro/Compositing/TextFrameRenderer.swift (GPLv3).

use super::animator::TextAnimator;
use clawvinci_media::decode::VideoFrame;
use clawvinci_model::text_animation::TextAnimation;
use clawvinci_model::text_style::{Rgba, TextAlignment, TextStyle};
use fontdue::{Font, FontSettings};
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex};

pub struct TextRenderer;

static FONT_CACHE: Mutex<Option<HashMap<String, Arc<Font>>>> = Mutex::new(None);

impl TextRenderer {
    pub const REFERENCE_CANVAS_HEIGHT: f64 = 1080.0;

    /// Attempts to find and load a font from the system font directory.
    pub fn load_font(family: &str) -> Option<Arc<Font>> {
        let key = family.to_lowercase();
        {
            let mut guard = FONT_CACHE.lock().unwrap();
            let cache = guard.get_or_insert_with(HashMap::new);
            if let Some(font) = cache.get(&key) {
                return Some(Arc::clone(font));
            }
        }

        let candidates = [
            format!("C:\\Windows\\Fonts\\{}.ttf", family),
            format!("C:\\Windows\\Fonts\\{}.otf", family),
            "C:\\Windows\\Fonts\\arial.ttf".to_string(),
            "C:\\Windows\\Fonts\\segoeui.ttf".to_string(),
            "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf".to_string(),
            "/System/Library/Fonts/Helvetica.ttc".to_string(),
        ];

        for path in &candidates {
            if Path::new(path).exists() {
                if let Ok(bytes) = std::fs::read(path) {
                    if let Ok(font) = Font::from_bytes(bytes, FontSettings::default()) {
                        let arc = Arc::new(font);
                        let mut guard = FONT_CACHE.lock().unwrap();
                        let cache = guard.get_or_insert_with(HashMap::new);
                        cache.insert(key, Arc::clone(&arc));
                        return Some(arc);
                    }
                }
            }
        }

        None
    }

    /// Renders text into an RGBA VideoFrame.
    pub fn render_text(
        content: &str,
        style: &TextStyle,
        anim: Option<&TextAnimation>,
        frame: usize,
        render_width: u32,
        render_height: u32,
    ) -> Option<VideoFrame> {
        if render_width == 0 || render_height == 0 || content.is_empty() {
            return None;
        }

        let formatted = style.font_case.apply(content);
        if formatted.is_empty() {
            return None;
        }

        let scale = render_height as f64 / Self::REFERENCE_CANVAS_HEIGHT;
        let mut font_size = (style.font_size * scale).max(8.0) as f32;

        // Apply clip entrance animation if present
        let mut clip_opacity = 1.0f32;
        let mut offset_y = 0.0f64;
        if let Some(a) = anim {
            if a.is_active {
                let entry = TextAnimator::clip_entry(a, frame as i64);
                clip_opacity = entry.opacity;
                font_size *= entry.scale as f32;
                offset_y = entry.dy * render_height as f64;
            }
        }

        if clip_opacity <= 0.0 {
            return None;
        }

        let mut buffer = vec![0u8; (render_width * render_height * 4) as usize];
        let font_opt = Self::load_font(&style.font_family);

        let params = TextRenderParams {
            style,
            font_size,
            clip_opacity,
            offset_y,
            render_w: render_width,
            render_h: render_height,
        };

        if let Some(font) = font_opt {
            Self::render_with_font(&font, &formatted, &params, &mut buffer);
        } else {
            Self::render_fallback(&formatted, &params, &mut buffer);
        }

        Some(VideoFrame {
            width: render_width,
            height: render_height,
            frame_index: frame,
            data: buffer,
        })
    }

    fn render_with_font(
        font: &Font,
        text: &str,
        p: &TextRenderParams,
        canvas: &mut [u8],
    ) {
        let line_spacing = p.style.line_spacing as f32;
        let line_height = p.font_size * 1.25 + line_spacing;
        let lines: Vec<&str> = text.lines().collect();
        let total_text_h = lines.len() as f32 * line_height;

        let start_y = ((p.render_h as f32 - total_text_h) * 0.5 + p.offset_y as f32).max(0.0);

        // Render background box if enabled
        if p.style.background.enabled {
            let pad_x = p.style.background.padding_x as f32;
            let pad_y = p.style.background.padding_y as f32;
            let bg_color = p.style.background.color;
            let bg_alpha = (bg_color.a as f32 * p.clip_opacity).clamp(0.0, 1.0);

            let box_y0 = (start_y - pad_y).clamp(0.0, p.render_h as f32) as u32;
            let box_y1 = (start_y + total_text_h + pad_y).clamp(0.0, p.render_h as f32) as u32;

            for y in box_y0..box_y1 {
                for x in 0..p.render_w {
                    let idx = ((y * p.render_w + x) * 4) as usize;
                    blend_rgba_over(
                        &mut canvas[idx..idx + 4],
                        (bg_color.r * 255.0) as u8,
                        (bg_color.g * 255.0) as u8,
                        (bg_color.b * 255.0) as u8,
                        bg_alpha,
                    );
                }
            }
        }

        let text_color = p.style.color;
        let text_alpha = (text_color.a as f32 * p.clip_opacity).clamp(0.0, 1.0);

        for (line_idx, line) in lines.iter().enumerate() {
            let mut line_width = 0.0f32;
            for ch in line.chars() {
                let metrics = font.metrics(ch, p.font_size);
                line_width += metrics.advance_width + p.style.tracking as f32;
            }

            let line_x = match p.style.alignment {
                TextAlignment::Left => 40.0f32,
                TextAlignment::Center => ((p.render_w as f32 - line_width) * 0.5).max(0.0),
                TextAlignment::Right => (p.render_w as f32 - line_width - 40.0).max(0.0),
            };

            let cur_y = start_y + line_idx as f32 * line_height;
            let mut cur_x = line_x;

            for ch in line.chars() {
                let (metrics, bitmap) = font.rasterize(ch, p.font_size);
                let gx = cur_x + metrics.xmin as f32;
                let gy = cur_y + p.font_size - metrics.ymin as f32 - metrics.height as f32;

                for by in 0..metrics.height {
                    let py = (gy as i32 + by as i32).clamp(0, p.render_h as i32 - 1) as u32;
                    for bx in 0..metrics.width {
                        let px = (gx as i32 + bx as i32).clamp(0, p.render_w as i32 - 1) as u32;
                        let cov = bitmap[by * metrics.width + bx] as f32 / 255.0;
                        if cov <= 0.0 {
                            continue;
                        }

                        let alpha = text_alpha * cov;
                        let idx = ((py * p.render_w + px) * 4) as usize;

                        blend_rgba_over(
                            &mut canvas[idx..idx + 4],
                            (text_color.r * 255.0) as u8,
                            (text_color.g * 255.0) as u8,
                            (text_color.b * 255.0) as u8,
                            alpha,
                        );
                    }
                }

                cur_x += metrics.advance_width + p.style.tracking as f32;
            }
        }
    }

    fn render_fallback(
        text: &str,
        p: &TextRenderParams,
        canvas: &mut [u8],
    ) {
        let scale = 3u32;
        let char_w = 6 * scale;
        let char_h = 9 * scale;
        let lines: Vec<&str> = text.lines().collect();
        let total_h = lines.len() as u32 * char_h;

        let start_y = ((p.render_h.saturating_sub(total_h)) / 2).saturating_add(p.offset_y as u32);
        let color = p.style.color;
        let alpha = (color.a as f32 * p.clip_opacity).clamp(0.0, 1.0);

        for (line_idx, line) in lines.iter().enumerate() {
            let line_w = line.len() as u32 * char_w;
            let line_x = match p.style.alignment {
                TextAlignment::Left => 40,
                TextAlignment::Center => (p.render_w.saturating_sub(line_w)) / 2,
                TextAlignment::Right => p.render_w.saturating_sub(line_w + 40),
            };
            let y_pos = start_y + line_idx as u32 * char_h;

            for (ci, _ch) in line.chars().enumerate() {
                let x_pos = line_x + ci as u32 * char_w;
                for dy in 0..char_h.min(p.render_h.saturating_sub(y_pos)) {
                    for dx in 0..char_w.min(p.render_w.saturating_sub(x_pos)) {
                        let py = y_pos + dy;
                        let px = x_pos + dx;
                        if px < p.render_w && py < p.render_h {
                            let idx = ((py * p.render_w + px) * 4) as usize;
                            blend_rgba_over(
                                &mut canvas[idx..idx + 4],
                                (color.r * 255.0) as u8,
                                (color.g * 255.0) as u8,
                                (color.b * 255.0) as u8,
                                alpha,
                            );
                        }
                    }
                }
            }
        }
    }
}

struct TextRenderParams<'a> {
    pub style: &'a TextStyle,
    pub font_size: f32,
    pub clip_opacity: f32,
    pub offset_y: f64,
    pub render_w: u32,
    pub render_h: u32,
}

#[inline(always)]
fn blend_rgba_over(dst: &mut [u8], s_r: u8, s_g: u8, s_b: u8, s_a: f32) {
    if s_a <= 0.0 {
        return;
    }
    let d_r = dst[0] as f32;
    let d_g = dst[1] as f32;
    let d_b = dst[2] as f32;
    let d_a = dst[3] as f32 / 255.0;

    let out_a = s_a + d_a * (1.0 - s_a);
    if out_a > 0.0 {
        let inv_out_a = 1.0 / out_a;
        let out_r = (s_r as f32 * s_a + d_r * d_a * (1.0 - s_a)) * inv_out_a;
        let out_g = (s_g as f32 * s_a + d_g * d_a * (1.0 - s_a)) * inv_out_a;
        let out_b = (s_b as f32 * s_a + d_b * d_a * (1.0 - s_a)) * inv_out_a;

        dst[0] = out_r.round() as u8;
        dst[1] = out_g.round() as u8;
        dst[2] = out_b.round() as u8;
        dst[3] = (out_a * 255.0).round() as u8;
    }
}
