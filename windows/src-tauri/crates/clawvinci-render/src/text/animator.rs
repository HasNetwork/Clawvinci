// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only
// Derived from Sources/PalmierPro/Compositing/TextAnimator.swift (GPLv3).

use clawvinci_model::text_animation::{AnimationPreset, TextAnimation, WordTiming};
use clawvinci_model::text_style::Rgba;

#[derive(Debug, Clone, PartialEq)]
pub struct ClipState {
    pub opacity: f32,
    pub scale: f64,
    /// Vertical offset as a fraction of render height (positive = down).
    pub dy: f64,
}

impl Default for ClipState {
    fn default() -> Self {
        Self {
            opacity: 1.0,
            scale: 1.0,
            dy: 0.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WordState {
    pub opacity: f32,
    pub scale: f64,
    pub dy: f64,
    pub color: Rgba,
    pub bg_color: Option<Rgba>,
}

impl Default for WordState {
    fn default() -> Self {
        Self {
            opacity: 1.0,
            scale: 1.0,
            dy: 0.0,
            color: Rgba::default(),
            bg_color: None,
        }
    }
}

pub struct TextAnimator;

impl TextAnimator {
    /// Whole-clip entrance. Non-entrance presets return default identity.
    pub fn clip_entry(anim: &TextAnimation, rel: i64) -> ClipState {
        let dur = anim.per_word_frames.max(1);
        let t = Self::progress(rel, 0, dur);
        match anim.preset {
            AnimationPreset::PopIn => ClipState {
                opacity: t as f32,
                scale: 0.6 + 0.4 * t,
                dy: 0.0,
            },
            AnimationPreset::SlideUp => ClipState {
                opacity: t as f32,
                scale: 1.0,
                dy: 0.05 * (1.0 - t),
            },
            _ => ClipState::default(),
        }
    }

    /// Per-word state. `base` is the clip's static text color.
    pub fn word_state(
        anim: &TextAnimation,
        word: &WordTiming,
        next_word: Option<&WordTiming>,
        rel: i64,
        base: Rgba,
    ) -> WordState {
        let highlight = anim.highlight.unwrap_or(Rgba::new(1.0, 0.85, 0.0, 1.0));
        let hand = anim.per_word_frames.max(1);
        let word_start = word.start_frame;

        match anim.preset {
            AnimationPreset::WordReveal => {
                let t = Self::progress(rel, word_start, hand);
                WordState {
                    opacity: t as f32,
                    scale: 1.0,
                    dy: 0.0,
                    color: Self::active_tint(anim, word, rel, base),
                    bg_color: None,
                }
            }
            AnimationPreset::WordSlide => {
                let t = Self::progress(rel, word_start, hand);
                WordState {
                    opacity: t as f32,
                    scale: 1.0,
                    dy: 0.5 * (1.0 - t),
                    color: Self::active_tint(anim, word, rel, base),
                    bg_color: None,
                }
            }
            AnimationPreset::HighlightPop => {
                let ramp = hand.min(4) as usize;
                let on = Self::held_highlight_amount(rel, word, next_word, ramp);
                WordState {
                    opacity: 1.0,
                    scale: 1.0,
                    dy: 0.0,
                    color: Self::lerp(base, highlight, on),
                    bg_color: None,
                }
            }
            AnimationPreset::HighlightBlock => {
                let ramp = hand.min(4) as usize;
                let on = Self::held_highlight_amount(rel, word, next_word, ramp);
                let mut bg = highlight;
                bg.a *= on;
                WordState {
                    opacity: 1.0,
                    scale: 1.0,
                    dy: 0.0,
                    color: base,
                    bg_color: Some(bg),
                }
            }
            _ => WordState {
                opacity: 1.0,
                scale: 1.0,
                dy: 0.0,
                color: base,
                bg_color: None,
            },
        }
    }

    fn active_tint(anim: &TextAnimation, word: &WordTiming, rel: i64, base: Rgba) -> Rgba {
        let Some(hl) = anim.highlight else {
            return base;
        };
        let on = Self::active_ramp(rel, word, anim.per_word_frames.max(1));
        Self::lerp(base, hl, on)
    }

    fn held_highlight_amount(
        rel: i64,
        word: &WordTiming,
        next_word: Option<&WordTiming>,
        ramp: usize,
    ) -> f64 {
        let word_start = word.start_frame;
        if rel < word_start {
            return 0.0;
        }

        let ramp_in = if let Some(duration) = Self::ramp_duration(word, ramp) {
            let elapsed = rel - word_start;
            Self::smoothstep((elapsed as f64 / duration as f64).min(1.0))
        } else {
            1.0
        };

        if let Some(next) = next_word {
            let next_start = next.start_frame;
            if rel >= next_start {
                if let Some(duration) = Self::ramp_duration(next, ramp) {
                    let elapsed = rel - next_start;
                    let ramp_out = Self::smoothstep(1.0 - (elapsed as f64 / duration as f64).min(1.0));
                    return ramp_in.min(ramp_out);
                }
                return 0.0;
            }
        }

        ramp_in
    }

    fn progress(rel: i64, start: i64, dur: i64) -> f64 {
        Self::smoothstep(Self::linear(rel, start, dur))
    }

    fn linear(rel: i64, start: i64, dur: i64) -> f64 {
        if rel <= start {
            0.0
        } else if rel >= start + dur {
            1.0
        } else {
            (rel - start) as f64 / dur as f64
        }
    }

    fn active_ramp(rel: i64, word: &WordTiming, ramp: i64) -> f64 {
        let start = word.start_frame;
        let end = word.end_frame;
        if rel < start || rel >= end {
            return 0.0;
        }
        let Some(r) = Self::ramp_duration(word, ramp as usize) else {
            return 1.0;
        };
        let ramp_in = Self::smoothstep(((rel - start) as f64 / r as f64).min(1.0));
        let ramp_out = Self::smoothstep(((end - rel) as f64 / r as f64).min(1.0));
        ramp_in.min(ramp_out)
    }

    fn ramp_duration(word: &WordTiming, maximum: usize) -> Option<usize> {
        let span = word.end_frame.saturating_sub(word.start_frame);
        if span <= 1 {
            return None;
        }
        Some(maximum.max(1).min((span / 2) as usize).max(1))
    }

    fn smoothstep(t: f64) -> f64 {
        let x = t.clamp(0.0, 1.0);
        x * x * (3.0 - 2.0 * x)
    }

    fn lerp(a: Rgba, b: Rgba, t: f64) -> Rgba {
        let t = t.clamp(0.0, 1.0);
        Rgba::new(
            a.r + (b.r - a.r) * t,
            a.g + (b.g - a.g) * t,
            a.b + (b.b - a.b) * t,
            a.a + (b.a - a.a) * t,
        )
    }
}
