// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only

pub mod blur;
pub mod chroma_key;
pub mod clarity;
pub mod color_space;
pub mod color_wheels;
pub mod contrast;
pub mod edge_rounding;
pub mod exposure;
pub mod glow;
pub mod grade_curves;
pub mod grain;
pub mod highlights_shadows;
pub mod hue_curves;
pub mod invert;
pub mod levels;
pub mod lut;
pub mod lut_loader;
pub mod registry;
pub mod saturation;
pub mod temperature;
pub mod types;
pub mod vibrance;
pub mod vignette;

pub use edge_rounding::apply_edge_rounding;
pub use lut_loader::{CubeLut, LutLoader};
pub use registry::{EffectRegistry, ALL_EFFECTS, CANONICAL_ORDER};
pub use types::{EffectDescriptor, EffectParamSpec, ResolvedEffectParams};
