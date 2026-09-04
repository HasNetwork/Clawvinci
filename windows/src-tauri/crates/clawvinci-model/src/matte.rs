// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only
// Derived from Sources/PalmierPro/Models/Matte.swift (GPLv3).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MatteAspect {
    #[serde(rename = "Project")]
    Project,
    #[serde(rename = "16:9")]
    SixteenNine,
    #[serde(rename = "9:16")]
    NineSixteen,
    #[serde(rename = "1:1")]
    OneOne,
    #[serde(rename = "4:3")]
    FourThree,
    #[serde(rename = "9:14")]
    NineFourteen,
    #[serde(rename = "2.4:1")]
    TwoPointFourOne,
}

impl MatteAspect {
    pub fn ratio(&self) -> Option<(i32, i32)> {
        match self {
            Self::Project => None,
            Self::SixteenNine => Some((16, 9)),
            Self::NineSixteen => Some((9, 16)),
            Self::OneOne => Some((1, 1)),
            Self::FourThree => Some((4, 3)),
            Self::NineFourteen => Some((9, 14)),
            Self::TwoPointFourOne => Some((24, 10)),
        }
    }

    pub fn pixel_size(&self, timeline_w: i32, timeline_h: i32) -> (i32, i32) {
        match self.ratio() {
            None => Matte::even(timeline_w, timeline_h),
            Some((aw, ah)) => Matte::fit(timeline_w.min(timeline_h), aw, ah),
        }
    }
}

pub struct Matte;

impl Matte {
    pub fn even(w: i32, h: i32) -> (i32, i32) {
        (2.max((2.max(w) / 2) * 2), 2.max((2.max(h) / 2) * 2))
    }

    pub fn fit(short_edge: i32, aspect_w: i32, aspect_h: i32) -> (i32, i32) {
        let e = 2.max(short_edge);
        let aw = aspect_w as f64;
        let ah = aspect_h as f64;
        if aw >= ah {
            Self::even(((e as f64 * aw / ah).round()) as i32, e)
        } else {
            Self::even(e, ((e as f64 * ah / aw).round()) as i32)
        }
    }
}
