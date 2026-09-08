// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only
// Derived from Sources/PalmierPro/Generation/Catalog/CostEstimator.swift (GPLv3).

use super::models::{AudioPricing, ModelCatalogEntry};
use std::collections::HashMap;

pub struct CostEstimator;

impl CostEstimator {
    /// Estimate video generation cost in credits.
    pub fn estimate_video_cost(
        model: &ModelCatalogEntry,
        duration_seconds: i32,
        resolution: Option<&str>,
        generate_audio: bool,
        draft: bool,
        uses_source_video: bool,
    ) -> Option<u32> {
        if duration_seconds <= 0 {
            return None;
        }

        let caps = model.video_caps()?;
        let rates = if uses_source_video {
            caps.source_video_credits_per_second
                .as_ref()
                .or(model.pricing.credits_per_second.as_ref())
        } else {
            model.pricing.credits_per_second.as_ref()
        };

        let mut rate = if draft {
            if uses_source_video {
                caps.source_video_draft_credits_per_second
                    .or(caps.draft_credits_per_second)?
            } else {
                caps.draft_credits_per_second?
            }
        } else {
            rates.and_then(|r| Self::resolved_rate(r, resolution))?
        };

        if !draft && !generate_audio {
            if let Some(discounts) = &model.pricing.audio_discount_rate {
                if let Some(discount) = Self::resolved_rate(discounts, resolution) {
                    rate *= discount;
                }
            }
        }

        Some(Self::ceil_credits(rate * (duration_seconds as f64)))
    }

    /// Estimate image generation cost in credits.
    pub fn estimate_image_cost(
        model: &ModelCatalogEntry,
        resolution: Option<&str>,
        quality: Option<&str>,
        num_images: usize,
    ) -> Option<u32> {
        let pricing = model.pricing.credits_per_image.as_ref()?;
        if pricing.is_empty() {
            return None;
        }

        let count = num_images.max(1) as f64;

        // 1. Matrix lookup "resolution|quality"
        if let (Some(r), Some(q)) = (resolution, quality) {
            let key = format!("{r}|{q}");
            if let Some(&price) = pricing.get(&key) {
                return Some(Self::ceil_credits(price * count));
            }
        }

        // 2. Quality-only lookup
        if let Some(q) = quality {
            if let Some(&price) = pricing.get(q) {
                return Some(Self::ceil_credits(price * count));
            }
        }

        // 3. Resolution or default lookup
        let rate = Self::resolved_rate(pricing, resolution)?;
        Some(Self::ceil_credits(rate * count))
    }

    /// Estimate speech audio generation cost in credits.
    pub fn estimate_speech_cost(
        model: &ModelCatalogEntry,
        char_count: usize,
        duration_seconds: Option<i32>,
    ) -> Option<u32> {
        let pricing = model.pricing.audio_pricing.as_ref()?;
        match pricing {
            AudioPricing::PerThousandChars { rate } => {
                if char_count == 0 {
                    return None;
                }
                Some(Self::ceil_credits(rate * (char_count as f64 / 1000.0)))
            }
            AudioPricing::PerSecond { rate, .. } => {
                let secs = duration_seconds?;
                if secs <= 0 {
                    return None;
                }
                Some(Self::ceil_credits(rate * (secs as f64)))
            }
            AudioPricing::Flat { price } => Some(Self::ceil_credits(*price)),
        }
    }

    /// Estimate music audio generation cost in credits.
    pub fn estimate_music_cost(model: &ModelCatalogEntry, duration_seconds: i32) -> Option<u32> {
        if duration_seconds <= 0 {
            return None;
        }
        let pricing = model.pricing.audio_pricing.as_ref()?;
        match pricing {
            AudioPricing::PerSecond { rate, .. } => {
                Some(Self::ceil_credits(rate * (duration_seconds as f64)))
            }
            AudioPricing::Flat { price } => Some(Self::ceil_credits(*price)),
            AudioPricing::PerThousandChars { rate } => {
                Some(Self::ceil_credits(rate * (duration_seconds as f64)))
            }
        }
    }

    /// Estimate upscale cost in credits.
    pub fn estimate_upscale_cost(
        model: &ModelCatalogEntry,
        duration_seconds: i32,
        scale_factor: f64,
    ) -> Option<u32> {
        let d = duration_seconds.max(1) as f64;
        let mut rate = model.pricing.credits_per_second_upscale.unwrap_or(1.0);
        if scale_factor > 2.0 {
            rate *= 1.5;
        }
        Some(Self::ceil_credits(rate * d))
    }

    fn resolved_rate(dict: &HashMap<String, f64>, key: Option<&str>) -> Option<f64> {
        if let Some(k) = key {
            if let Some(&v) = dict.get(k) {
                return Some(v);
            }
        }
        dict.get("").copied()
    }

    fn ceil_credits(credits: f64) -> u32 {
        if credits <= 0.0 {
            0
        } else {
            credits.ceil() as u32
        }
    }
}
