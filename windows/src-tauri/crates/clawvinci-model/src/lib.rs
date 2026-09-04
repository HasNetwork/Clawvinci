// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only
//! Clawvinci Domain Models & Project File Format (Phase 1).

pub mod blend_mode;
pub mod clip_type;
pub mod effect;
pub mod error;
pub mod grade;
pub mod keyframe;
pub mod layout;
pub mod matte;
pub mod media_manifest;
pub mod media_resolver;
pub mod multicam;
pub mod project_file;
pub mod speaker;
pub mod text_animation;
pub mod text_fill_mode;
pub mod text_layout;
pub mod text_style;
pub mod timeline;
pub mod timeline_marker;

pub use blend_mode::BlendMode;
pub use clip_type::ClipType;
pub use effect::{Effect, EffectParam};
pub use error::ModelError;
pub use grade::{CurvePoint, GradeCurve, HueCurves};
pub use keyframe::{
    smoothstep, AnimPair, AnimatableProperty, Interpolation, Keyframe, KeyframeInterpolatable,
    KeyframeTrack,
};
pub use layout::{LayoutRect, LayoutSlot, VideoLayout};
pub use matte::{Matte, MatteAspect};
pub use media_manifest::{
    GenerationInput, MediaFolder, MediaImportInput, MediaManifest, MediaManifestEntry, MediaSource,
    UpscaleSettings,
};
pub use media_resolver::MediaResolver;
pub use multicam::{MemberKind, MulticamMember, MulticamSource, SyncMap};
pub use project_file::ProjectFile;
pub use speaker::SpeakerRegistryEntry;
pub use text_animation::{AnimationPreset, TextAnimation, WordTiming};
pub use text_fill_mode::TextFillMode;
pub use text_layout::{TextLayout, TextSize};
pub use text_style::{
    FontCase, Rgba, TextAlignment, TextBackground, TextOutline, TextShadow, TextStyle,
};
pub use timeline::{
    Clip, ClipLocation, Crop, CropAspectRatio, Timeline, TimelineViewState, Track, Transform,
};
pub use timeline_marker::{MarkerStatus, TimelineMarker};
