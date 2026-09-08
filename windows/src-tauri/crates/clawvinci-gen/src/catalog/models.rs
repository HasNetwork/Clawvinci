// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only
// Derived from Sources/PalmierPro/Generation/Catalog/ModelCatalog.swift (GPLv3).

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ModelModality {
    Video,
    Image,
    Audio,
    Music,
    Upscale,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct VideoCaps {
    #[serde(default)]
    pub durations: Vec<i32>,
    #[serde(default)]
    pub resolutions: Vec<String>,
    #[serde(default)]
    pub aspect_ratios: Vec<String>,
    #[serde(default)]
    pub supports_first_frame: bool,
    #[serde(default)]
    pub supports_last_frame: bool,
    #[serde(default)]
    pub max_reference_images: usize,
    #[serde(default)]
    pub max_reference_videos: usize,
    #[serde(default)]
    pub max_reference_audios: usize,
    #[serde(default)]
    pub requires_source_video: bool,
    #[serde(default)]
    pub max_source_video_seconds: Option<f64>,
    #[serde(default)]
    pub max_source_video_resolution: Option<String>,
    #[serde(default)]
    pub requires_reference_image: bool,
    #[serde(default)]
    pub draft_credits_per_second: Option<f64>,
    #[serde(default)]
    pub source_video_credits_per_second: Option<HashMap<String, f64>>,
    #[serde(default)]
    pub source_video_draft_credits_per_second: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ImageCaps {
    #[serde(default)]
    pub resolutions: Vec<String>,
    #[serde(default)]
    pub aspect_ratios: Vec<String>,
    #[serde(default)]
    pub qualities: Vec<String>,
    #[serde(default)]
    pub supports_image_reference: bool,
    #[serde(default)]
    pub max_images: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct AudioCaps {
    pub category: String, // "speech" | "music"
    #[serde(default)]
    pub voices: Vec<String>,
    #[serde(default)]
    pub default_voice: Option<String>,
    #[serde(default)]
    pub supports_lyrics: bool,
    #[serde(default)]
    pub supports_instrumental: bool,
    #[serde(default)]
    pub supports_style_instructions: bool,
    #[serde(default)]
    pub durations: Vec<i32>,
    #[serde(default)]
    pub min_prompt_length: usize,
    #[serde(default)]
    pub inputs: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct UpscaleCaps {
    pub speed: String, // "Fast" | "Medium" | "Slow"
    pub p75_duration_seconds: i32,
    #[serde(default)]
    pub maximum_upscale_factor: Option<f64>,
    #[serde(default)]
    pub supported_types: Vec<String>, // "video" | "image"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "capabilities", rename_all = "lowercase")]
pub enum ModelCapabilities {
    Video(VideoCaps),
    Image(ImageCaps),
    Audio(AudioCaps),
    Upscale(UpscaleCaps),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "mode", rename_all = "camelCase")]
pub enum AudioPricing {
    #[serde(rename_all = "camelCase")]
    PerThousandChars { rate: f64 },
    #[serde(rename_all = "camelCase")]
    PerSecond {
        rate: f64,
        #[serde(default)]
        text_rate: Option<f64>,
    },
    #[serde(rename_all = "camelCase")]
    Flat { price: f64 },
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ModelPricing {
    #[serde(default)]
    pub credits_per_second: Option<HashMap<String, f64>>,
    #[serde(default)]
    pub audio_discount_rate: Option<HashMap<String, f64>>,
    #[serde(default)]
    pub credits_per_image: Option<HashMap<String, f64>>,
    #[serde(default)]
    pub audio_pricing: Option<AudioPricing>,
    #[serde(default)]
    pub credits_per_second_upscale: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelCatalogEntry {
    pub id: String,
    pub display_name: String,
    pub modality: ModelModality,
    pub provider: String,
    #[serde(default)]
    pub provider_icon_key: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub pricing: ModelPricing,
    pub capabilities: ModelCapabilities,
    #[serde(default)]
    pub paid_only: bool,
}

impl ModelCatalogEntry {
    pub fn video_caps(&self) -> Option<&VideoCaps> {
        match &self.capabilities {
            ModelCapabilities::Video(c) => Some(c),
            _ => None,
        }
    }

    pub fn image_caps(&self) -> Option<&ImageCaps> {
        match &self.capabilities {
            ModelCapabilities::Image(c) => Some(c),
            _ => None,
        }
    }

    pub fn audio_caps(&self) -> Option<&AudioCaps> {
        match &self.capabilities {
            ModelCapabilities::Audio(c) => Some(c),
            _ => None,
        }
    }

    pub fn upscale_caps(&self) -> Option<&UpscaleCaps> {
        match &self.capabilities {
            ModelCapabilities::Upscale(c) => Some(c),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelCatalog {
    pub entries: Vec<ModelCatalogEntry>,
    #[serde(skip)]
    pub by_id: HashMap<String, ModelCatalogEntry>,
}

impl Default for ModelCatalog {
    fn default() -> Self {
        Self::default_catalog()
    }
}

impl ModelCatalog {
    pub fn new(entries: Vec<ModelCatalogEntry>) -> Self {
        let mut by_id = HashMap::with_capacity(entries.len());
        for entry in &entries {
            by_id.insert(entry.id.clone(), entry.clone());
        }
        Self { entries, by_id }
    }

    pub fn get(&self, id: &str) -> Option<&ModelCatalogEntry> {
        self.by_id.get(id)
    }

    pub fn models_for_modality(&self, modality: ModelModality) -> Vec<&ModelCatalogEntry> {
        self.entries
            .iter()
            .filter(|e| e.modality == modality)
            .collect()
    }

    pub fn apply(&mut self, new_entries: Vec<ModelCatalogEntry>) {
        self.by_id.clear();
        for entry in &new_entries {
            self.by_id.insert(entry.id.clone(), entry.clone());
        }
        self.entries = new_entries;
    }

    /// Built-in default catalog containing standard SOTA models.
    /// Provides deterministic offline functionality and immediate catalog availability.
    pub fn default_catalog() -> Self {
        let mut entries = Vec::new();

        // 1. Seedance Video Fast
        let mut seedance_pricing = HashMap::new();
        seedance_pricing.insert("720p".to_string(), 2.0);
        seedance_pricing.insert("1080p".to_string(), 4.0);
        seedance_pricing.insert("".to_string(), 2.0);

        let mut seedance_audio_discount = HashMap::new();
        seedance_audio_discount.insert("720p".to_string(), 0.8);
        seedance_audio_discount.insert("1080p".to_string(), 0.85);

        entries.push(ModelCatalogEntry {
            id: "seedance-v1-fast".to_string(),
            display_name: "Seedance Video Fast".to_string(),
            modality: ModelModality::Video,
            provider: "Seedance".to_string(),
            provider_icon_key: Some("seedance".to_string()),
            description: Some("Fast high-fidelity video generation model".to_string()),
            pricing: ModelPricing {
                credits_per_second: Some(seedance_pricing),
                audio_discount_rate: Some(seedance_audio_discount),
                credits_per_image: None,
                audio_pricing: None,
                credits_per_second_upscale: None,
            },
            capabilities: ModelCapabilities::Video(VideoCaps {
                durations: vec![5, 10],
                resolutions: vec!["720p".to_string(), "1080p".to_string()],
                aspect_ratios: vec!["16:9".to_string(), "9:16".to_string(), "1:1".to_string()],
                supports_first_frame: true,
                supports_last_frame: true,
                max_reference_images: 4,
                max_reference_videos: 1,
                max_reference_audios: 1,
                requires_source_video: false,
                max_source_video_seconds: Some(15.0),
                max_source_video_resolution: Some("1080p".to_string()),
                requires_reference_image: false,
                draft_credits_per_second: Some(1.0),
                source_video_credits_per_second: None,
                source_video_draft_credits_per_second: None,
            }),
            paid_only: false,
        });

        // 2. Kling Video
        let mut kling_pricing = HashMap::new();
        kling_pricing.insert("720p".to_string(), 3.0);
        kling_pricing.insert("1080p".to_string(), 6.0);
        kling_pricing.insert("".to_string(), 3.0);

        entries.push(ModelCatalogEntry {
            id: "kling-v1".to_string(),
            display_name: "Kling v1.5".to_string(),
            modality: ModelModality::Video,
            provider: "Kuaishou".to_string(),
            provider_icon_key: Some("kling".to_string()),
            description: Some("Cinematic camera motion and realistic physics".to_string()),
            pricing: ModelPricing {
                credits_per_second: Some(kling_pricing),
                audio_discount_rate: None,
                credits_per_image: None,
                audio_pricing: None,
                credits_per_second_upscale: None,
            },
            capabilities: ModelCapabilities::Video(VideoCaps {
                durations: vec![5, 10],
                resolutions: vec!["720p".to_string(), "1080p".to_string()],
                aspect_ratios: vec!["16:9".to_string(), "9:16".to_string(), "1:1".to_string()],
                supports_first_frame: true,
                supports_last_frame: false,
                max_reference_images: 2,
                max_reference_videos: 0,
                max_reference_audios: 0,
                requires_source_video: false,
                max_source_video_seconds: None,
                max_source_video_resolution: None,
                requires_reference_image: false,
                draft_credits_per_second: None,
                source_video_credits_per_second: None,
                source_video_draft_credits_per_second: None,
            }),
            paid_only: false,
        });

        // 3. Nano Banana Pro (Image)
        let mut banana_pricing = HashMap::new();
        banana_pricing.insert("1024x1024".to_string(), 1.0);
        banana_pricing.insert("1920x1080".to_string(), 2.0);
        banana_pricing.insert("".to_string(), 1.0);

        entries.push(ModelCatalogEntry {
            id: "nano-banana-pro".to_string(),
            display_name: "Nano Banana Pro".to_string(),
            modality: ModelModality::Image,
            provider: "Banana AI".to_string(),
            provider_icon_key: Some("banana".to_string()),
            description: Some("Ultra-fast stylized photo and asset generation".to_string()),
            pricing: ModelPricing {
                credits_per_second: None,
                audio_discount_rate: None,
                credits_per_image: Some(banana_pricing),
                audio_pricing: None,
                credits_per_second_upscale: None,
            },
            capabilities: ModelCapabilities::Image(ImageCaps {
                resolutions: vec!["1024x1024".to_string(), "1920x1080".to_string()],
                aspect_ratios: vec!["1:1".to_string(), "16:9".to_string(), "9:16".to_string()],
                qualities: vec!["standard".to_string(), "hd".to_string()],
                supports_image_reference: true,
                max_images: 4,
            }),
            paid_only: false,
        });

        // 4. FLUX.1 Schnell (Image)
        let mut flux_pricing = HashMap::new();
        flux_pricing.insert("1024x1024".to_string(), 1.5);
        flux_pricing.insert("".to_string(), 1.5);

        entries.push(ModelCatalogEntry {
            id: "flux-1-schnell".to_string(),
            display_name: "FLUX.1 Schnell".to_string(),
            modality: ModelModality::Image,
            provider: "Black Forest Labs".to_string(),
            provider_icon_key: Some("bfl".to_string()),
            description: Some("State-of-the-art open weights image synthesis".to_string()),
            pricing: ModelPricing {
                credits_per_second: None,
                audio_discount_rate: None,
                credits_per_image: Some(flux_pricing),
                audio_pricing: None,
                credits_per_second_upscale: None,
            },
            capabilities: ModelCapabilities::Image(ImageCaps {
                resolutions: vec!["1024x1024".to_string(), "1920x1080".to_string()],
                aspect_ratios: vec!["1:1".to_string(), "16:9".to_string(), "4:3".to_string()],
                qualities: vec!["standard".to_string()],
                supports_image_reference: true,
                max_images: 4,
            }),
            paid_only: false,
        });

        // 5. ElevenLabs Multilingual v2 (Speech Audio)
        entries.push(ModelCatalogEntry {
            id: "eleven-multilingual-v2".to_string(),
            display_name: "ElevenLabs Neural Voice".to_string(),
            modality: ModelModality::Audio,
            provider: "ElevenLabs".to_string(),
            provider_icon_key: Some("elevenlabs".to_string()),
            description: Some("Emotional natural speech synthesizer".to_string()),
            pricing: ModelPricing {
                credits_per_second: None,
                audio_discount_rate: None,
                credits_per_image: None,
                audio_pricing: Some(AudioPricing::PerThousandChars { rate: 2.0 }),
                credits_per_second_upscale: None,
            },
            capabilities: ModelCapabilities::Audio(AudioCaps {
                category: "speech".to_string(),
                voices: vec![
                    "Rachel".to_string(),
                    "Domi".to_string(),
                    "Bella".to_string(),
                    "Antoni".to_string(),
                    "Elli".to_string(),
                    "Josh".to_string(),
                ],
                default_voice: Some("Rachel".to_string()),
                supports_lyrics: false,
                supports_instrumental: false,
                supports_style_instructions: true,
                durations: vec![],
                min_prompt_length: 1,
                inputs: vec!["text".to_string()],
            }),
            paid_only: false,
        });

        // 6. Suno v3.5 (Music Audio)
        entries.push(ModelCatalogEntry {
            id: "suno-v3-5".to_string(),
            display_name: "Suno Music Engine v3.5".to_string(),
            modality: ModelModality::Music,
            provider: "Suno".to_string(),
            provider_icon_key: Some("suno".to_string()),
            description: Some("Full instrumental and vocal song generation".to_string()),
            pricing: ModelPricing {
                credits_per_second: None,
                audio_discount_rate: None,
                credits_per_image: None,
                audio_pricing: Some(AudioPricing::PerSecond {
                    rate: 0.2,
                    text_rate: None,
                }),
                credits_per_second_upscale: None,
            },
            capabilities: ModelCapabilities::Audio(AudioCaps {
                category: "music".to_string(),
                voices: vec![],
                default_voice: None,
                supports_lyrics: true,
                supports_instrumental: true,
                supports_style_instructions: true,
                durations: vec![15, 30, 60, 120],
                min_prompt_length: 5,
                inputs: vec!["text".to_string(), "video".to_string()],
            }),
            paid_only: false,
        });

        // 7. Real-ESRGAN (Video & Image Upscale)
        entries.push(ModelCatalogEntry {
            id: "real-esrgan-4x".to_string(),
            display_name: "Real-ESRGAN Ultra Clarity".to_string(),
            modality: ModelModality::Upscale,
            provider: "Clawvinci AI".to_string(),
            provider_icon_key: Some("clawvinci".to_string()),
            description: Some("AI upscaling and detail reconstruction up to 4K".to_string()),
            pricing: ModelPricing {
                credits_per_second: None,
                audio_discount_rate: None,
                credits_per_image: None,
                audio_pricing: None,
                credits_per_second_upscale: Some(1.5),
            },
            capabilities: ModelCapabilities::Upscale(UpscaleCaps {
                speed: "Fast".to_string(),
                p75_duration_seconds: 12,
                maximum_upscale_factor: Some(4.0),
                supported_types: vec!["video".to_string(), "image".to_string()],
            }),
            paid_only: false,
        });

        Self::new(entries)
    }
}
