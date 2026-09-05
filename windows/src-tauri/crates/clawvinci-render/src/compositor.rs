// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only
// Derived from Sources/PalmierPro/Compositing/CustomVideoCompositor.swift and FrameRenderer.swift (GPLv3).

use crate::error::{RenderError, RenderResult};
use crate::plan::{FramePlan, LayerPlan, LayerSource};
use clawvinci_media::decode::VideoFrame;
use clawvinci_model::blend_mode::BlendMode;
use std::collections::HashMap;

/// Authoritative compositing function shared between preview and export.
/// Composites all active layers bottom-to-top onto an RGBA canvas according to `FramePlan`.
pub fn composite_frame(
    plan: &FramePlan,
    sources: &HashMap<String, VideoFrame>,
) -> RenderResult<VideoFrame> {
    let width = plan.render_width;
    let height = plan.render_height;

    if width == 0 || height == 0 {
        return Err(RenderError::InvalidTimeline(format!(
            "Invalid frame dimensions: {}x{}",
            width, height
        )));
    }

    let mut canvas = vec![0u8; (width * height * 4) as usize];
    // Fill canvas with opaque black background (R=0, G=0, B=0, A=255)
    for chunk in canvas.chunks_exact_mut(4) {
        chunk[0] = 0;
        chunk[1] = 0;
        chunk[2] = 0;
        chunk[3] = 255;
    }

    for layer in &plan.layers {
        if layer.opacity <= 0.0 {
            continue;
        }
        composite_layer(&mut canvas, width, height, layer, sources)?;
    }

    Ok(VideoFrame {
        width,
        height,
        frame_index: plan.frame_index,
        data: canvas,
    })
}

fn composite_layer(
    canvas: &mut [u8],
    canvas_w: u32,
    canvas_h: u32,
    layer: &LayerPlan,
    sources: &HashMap<String, VideoFrame>,
) -> RenderResult<()> {
    // Determine source image pixels
    let source_frame_opt = match &layer.source {
        LayerSource::Video { media_ref, .. } | LayerSource::Image { media_ref } => {
            sources.get(&layer.clip_id).or_else(|| sources.get(media_ref))
        }
        LayerSource::Nested { sub_plan, .. } => {
            let nested_rendered = composite_frame(sub_plan, sources)?;
            return composite_rendered_frame(canvas, canvas_w, canvas_h, layer, &nested_rendered);
        }
        LayerSource::SolidColor { r, g, b, a } => {
            let solid = VideoFrame {
                width: 1,
                height: 1,
                frame_index: 0,
                data: vec![*r, *g, *b, *a],
            };
            return composite_rendered_frame(canvas, canvas_w, canvas_h, layer, &solid);
        }
        LayerSource::Text { .. } => {
            // Placeholder text raster: semi-transparent accent block
            let text_buf = VideoFrame {
                width: 2,
                height: 2,
                frame_index: 0,
                data: vec![240, 240, 245, 255; 4],
            };
            return composite_rendered_frame(canvas, canvas_w, canvas_h, layer, &text_buf);
        }
    };

    if let Some(src) = source_frame_opt {
        composite_rendered_frame(canvas, canvas_w, canvas_h, layer, src)?;
    }

    Ok(())
}

fn composite_rendered_frame(
    canvas: &mut [u8],
    canvas_w: u32,
    canvas_h: u32,
    layer: &LayerPlan,
    src: &VideoFrame,
) -> RenderResult<()> {
    if src.width == 0 || src.height == 0 || src.data.is_empty() {
        return Ok(());
    }

    // Source crop calculations
    let crop = &layer.crop;
    let src_crop_x = (src.width as f64 * crop.left).clamp(0.0, src.width as f64) as u32;
    let src_crop_y = (src.height as f64 * crop.top).clamp(0.0, src.height as f64) as u32;
    let src_crop_w = (src.width as f64 * crop.visible_width_fraction())
        .clamp(1.0, (src.width - src_crop_x).max(1) as f64) as u32;
    let src_crop_h = (src.height as f64 * crop.visible_height_fraction())
        .clamp(1.0, (src.height - src_crop_y).max(1) as f64) as u32;

    // Target geometry on canvas
    let transform = &layer.transform;
    let target_w = (canvas_w as f64 * transform.width.abs()).round().max(1.0) as i32;
    let target_h = (canvas_h as f64 * transform.height.abs()).round().max(1.0) as i32;

    let target_cx = (canvas_w as f64 * transform.center_x).round() as i32;
    let target_cy = (canvas_h as f64 * transform.center_y).round() as i32;

    let target_x0 = target_cx - target_w / 2;
    let target_y0 = target_cy - target_h / 2;

    // Intersect target rect with canvas boundaries
    let clip_x0 = target_x0.max(0);
    let clip_y0 = target_y0.max(0);
    let clip_x1 = (target_x0 + target_w).min(canvas_w as i32);
    let clip_y1 = (target_y0 + target_h).min(canvas_h as i32);

    if clip_x1 <= clip_x0 || clip_y1 <= clip_y0 {
        return Ok(());
    }

    let opacity = layer.opacity.clamp(0.0, 1.0);
    let blend_mode = layer.blend_mode;

    for cy in clip_y0..clip_y1 {
        let ty = cy - target_y0;
        let norm_y = if transform.flip_vertical {
            1.0 - (ty as f64 / target_h as f64)
        } else {
            ty as f64 / target_h as f64
        };
        let sy = (src_crop_y as f64 + norm_y * src_crop_h as f64).clamp(0.0, (src.height - 1) as f64) as u32;

        for cx in clip_x0..clip_x1 {
            let tx = cx - target_x0;
            let norm_x = if transform.flip_horizontal {
                1.0 - (tx as f64 / target_w as f64)
            } else {
                tx as f64 / target_w as f64
            };
            let sx = (src_crop_x as f64 + norm_x * src_crop_w as f64).clamp(0.0, (src.width - 1) as f64) as u32;

            let src_idx = ((sy * src.width + sx) * 4) as usize;
            let dst_idx = ((cy as u32 * canvas_w + cx as u32) * 4) as usize;
            if src_idx + 3 < src.data.len() && dst_idx + 3 < canvas.len() {
                let s_r = src.data[src_idx];
                let s_g = src.data[src_idx + 1];
                let s_b = src.data[src_idx + 2];
                let s_a = src.data[src_idx + 3];

                blend_pixel(&mut canvas[dst_idx..dst_idx + 4], s_r, s_g, s_b, s_a, opacity, blend_mode);
            }
        }
    }

    Ok(())
}

#[inline(always)]
fn blend_pixel(
    dst: &mut [u8],
    s_r: u8,
    s_g: u8,
    s_b: u8,
    s_a: u8,
    layer_opacity: f32,
    mode: BlendMode,
) {
    let src_alpha = (s_a as f32 / 255.0) * layer_opacity;
    if src_alpha <= 0.0 {
        return;
    }

    let d_r = dst[0] as f32;
    let d_g = dst[1] as f32;
    let d_b = dst[2] as f32;
    let d_a = dst[3] as f32 / 255.0;

    let s_rf = s_r as f32;
    let s_gf = s_g as f32;
    let s_bf = s_b as f32;

    let (blended_r, blended_g, blended_b) = match mode {
        BlendMode::Normal => (s_rf, s_gf, s_bf),
        BlendMode::Multiply => (
            (s_rf * d_r) / 255.0,
            (s_gf * d_g) / 255.0,
            (s_bf * d_b) / 255.0,
        ),
        BlendMode::Screen => (
            255.0 - ((255.0 - s_rf) * (255.0 - d_r)) / 255.0,
            255.0 - ((255.0 - s_gf) * (255.0 - d_g)) / 255.0,
            255.0 - ((255.0 - s_bf) * (255.0 - d_b)) / 255.0,
        ),
        BlendMode::Darken => (s_rf.min(d_r), s_gf.min(d_g), s_bf.min(d_b)),
        BlendMode::Lighten => (s_rf.max(d_r), s_gf.max(d_g), s_bf.max(d_b)),
        BlendMode::Difference => (
            (s_rf - d_r).abs(),
            (s_gf - d_g).abs(),
            (s_bf - d_b).abs(),
        ),
        BlendMode::Overlay => (
            overlay_channel(s_rf, d_r),
            overlay_channel(s_gf, d_g),
            overlay_channel(s_bf, d_b),
        ),
        _ => (s_rf, s_gf, s_bf),
    };

    // Standard Porter-Duff source-over composite with blended colors
    let out_a = src_alpha + d_a * (1.0 - src_alpha);
    if out_a > 0.0 {
        let inv_out_a = 1.0 / out_a;
        let out_r = (blended_r * src_alpha + d_r * d_a * (1.0 - src_alpha)) * inv_out_a;
        let out_g = (blended_g * src_alpha + d_g * d_a * (1.0 - src_alpha)) * inv_out_a;
        let out_b = (blended_b * src_alpha + d_b * d_a * (1.0 - src_alpha)) * inv_out_a;

        dst[0] = out_r.clamp(0.0, 255.0) as u8;
        dst[1] = out_g.clamp(0.0, 255.0) as u8;
        dst[2] = out_b.clamp(0.0, 255.0) as u8;
        dst[3] = (out_a * 255.0).clamp(0.0, 255.0) as u8;
    }
}

#[inline(always)]
fn overlay_channel(src: f32, dst: f32) -> f32 {
    if dst <= 128.0 {
        (2.0 * src * dst) / 255.0
    } else {
        255.0 - (2.0 * (255.0 - src) * (255.0 - dst)) / 255.0
    }
}
