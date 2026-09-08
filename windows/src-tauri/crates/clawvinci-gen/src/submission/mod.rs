// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only

pub mod audio;
pub mod image;
pub mod music;
pub mod types;
pub mod video;

pub use audio::AudioGenerationSubmission;
pub use image::ImageGenerationSubmission;
pub use music::{MusicGenerationSubmission, MusicMode};
pub use types::{
    AudioGenerationParams, BackendGenerationParams, ImageGenerationParams,
    UpscaleGenerationParams, VideoGenerationParams,
};
pub use video::VideoGenerationSubmission;
