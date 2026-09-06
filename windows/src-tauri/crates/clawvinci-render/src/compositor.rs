use crate::effects::{apply_edge_rounding, EffectRegistry};
use crate::error::{RenderError, RenderResult};
use crate::plan::{FramePlan, LayerPlan, LayerSource};
use crate::text::TextRenderer;
use clawvinci_media::decode::VideoFrame;
use clawvinci_model::blend_mode::BlendMode;
use clawvinci_model::text_style::TextStyle;
use clawvinci_model::timeline::Crop;
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
    for chunk in canvas.as_chunks_mut::<4>().0 {
        *chunk = [0, 0, 0, 255];
    }

    for layer in &plan.layers {
        if layer.opacity <= 0.0 {
            continue;
        }
        composite_layer(&mut canvas, width, height, layer, sources, plan.frame_index)?;
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
    frame_index: usize,
) -> RenderResult<()> {
    // 1. Obtain and crop raw source pixels
    let mut working_frame = match &layer.source {
        LayerSource::Video { media_ref, .. } | LayerSource::Image { media_ref } => {
            let src_opt = sources.get(&layer.clip_id).or_else(|| sources.get(media_ref));
            if let Some(src) = src_opt {
                crop_source(src, &layer.crop)
            } else {
                return Ok(());
            }
        }
        LayerSource::Nested { sub_plan, .. } => {
            let nested_rendered = composite_frame(sub_plan, sources)?;
            crop_source(&nested_rendered, &layer.crop)
        }
        LayerSource::SolidColor { r, g, b, a } => VideoFrame {
            width: 1,
            height: 1,
            frame_index,
            data: vec![*r, *g, *b, *a],
        },
        LayerSource::Text {
            content,
            style,
            animation,
        } => {
            let default_style = TextStyle::default();
            let s = style.as_ref().unwrap_or(&default_style);
            if let Some(text_frame) = TextRenderer::render_text(
                content,
                s,
                animation.as_ref(),
                frame_index,
                canvas_w,
                canvas_h,
            ) {
                text_frame
            } else {
                return Ok(());
            }
        }
    };

    if working_frame.width == 0 || working_frame.height == 0 || working_frame.data.is_empty() {
        return Ok(());
    }

    // 2. Effects apply in source-pixel space: after crop, before placement
    apply_layer_effects(&mut working_frame, layer);

    // 3. Edge rounding applies after effects
    apply_layer_edge_rounding(&mut working_frame, layer);

    // 4. Transform and composite onto canvas
    composite_placed_frame(canvas, canvas_w, canvas_h, layer, &working_frame);

    Ok(())
}

fn crop_source(src: &VideoFrame, crop: &Crop) -> VideoFrame {
    if crop.is_identity() || src.width == 0 || src.height == 0 {
        return src.clone();
    }

    let src_w = src.width as f64;
    let src_h = src.height as f64;
    let x0 = (src_w * crop.left).clamp(0.0, src_w) as u32;
    let y0 = (src_h * crop.top).clamp(0.0, src_h) as u32;
    let w = (src_w * crop.visible_width_fraction()).clamp(1.0, (src.width - x0).max(1) as f64) as u32;
    let h = (src_h * crop.visible_height_fraction()).clamp(1.0, (src.height - y0).max(1) as f64) as u32;

    let mut data = vec![0u8; (w * h * 4) as usize];
    for dy in 0..h {
        let sy = y0 + dy;
        let s_row = ((sy * src.width + x0) * 4) as usize;
        let d_row = ((dy * w) * 4) as usize;
        let row_len = (w * 4) as usize;
        if s_row + row_len <= src.data.len() && d_row + row_len <= data.len() {
            data[d_row..d_row + row_len].copy_from_slice(&src.data[s_row..s_row + row_len]);
        }
    }

    VideoFrame {
        width: w,
        height: h,
        frame_index: src.frame_index,
        data,
    }
}

fn apply_layer_effects(frame: &mut VideoFrame, layer: &LayerPlan) {
    for effect_plan in &layer.effects {
        if !effect_plan.enabled {
            continue;
        }
        if let Some(desc) = EffectRegistry::descriptor(&effect_plan.effect_type) {
            EffectRegistry::render_effect(
                desc,
                &mut frame.data,
                frame.width,
                frame.height,
                &effect_plan.params,
            );
        }
    }
}

fn apply_layer_edge_rounding(frame: &mut VideoFrame, layer: &LayerPlan) {
    if layer.edge_rounding > 0.0 || layer.edge_softness > 0.0 {
        apply_edge_rounding(
            &mut frame.data,
            frame.width,
            frame.height,
            layer.edge_rounding as f32,
            layer.edge_softness as f32,
        );
    }
}

fn composite_placed_frame(
    canvas: &mut [u8],
    canvas_w: u32,
    canvas_h: u32,
    layer: &LayerPlan,
    src: &VideoFrame,
) {
    let transform = &layer.transform;
    let target_w = (canvas_w as f64 * transform.width.abs()).round().max(1.0) as i32;
    let target_h = (canvas_h as f64 * transform.height.abs()).round().max(1.0) as i32;

    let target_cx = (canvas_w as f64 * transform.center_x).round() as i32;
    let target_cy = (canvas_h as f64 * transform.center_y).round() as i32;

    let target_x0 = target_cx - target_w / 2;
    let target_y0 = target_cy - target_h / 2;

    let clip_x0 = target_x0.max(0);
    let clip_y0 = target_y0.max(0);
    let clip_x1 = (target_x0 + target_w).min(canvas_w as i32);
    let clip_y1 = (target_y0 + target_h).min(canvas_h as i32);

    if clip_x1 <= clip_x0 || clip_y1 <= clip_y0 {
        return;
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
        let sy = (norm_y * (src.height as f64 - 1.0)).clamp(0.0, (src.height - 1) as f64) as u32;

        for cx in clip_x0..clip_x1 {
            let tx = cx - target_x0;
            let norm_x = if transform.flip_horizontal {
                1.0 - (tx as f64 / target_w as f64)
            } else {
                tx as f64 / target_w as f64
            };
            let sx = (norm_x * (src.width as f64 - 1.0)).clamp(0.0, (src.width - 1) as f64) as u32;

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
