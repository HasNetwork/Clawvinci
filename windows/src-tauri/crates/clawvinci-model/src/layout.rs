// Copyright (C) 2026 Clawvinci Contributors.
// SPDX-License-Identifier: GPL-3.0-only
// Derived from Sources/PalmierPro/Models/VideoLayout.swift (GPLv3).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct LayoutRect {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

impl LayoutRect {
    pub fn new(x: f64, y: f64, w: f64, h: f64) -> Self {
        Self { x, y, w, h }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LayoutSlot {
    pub id: String,
    pub rect: LayoutRect,
    #[serde(default)]
    pub z: i32,
}

impl LayoutSlot {
    pub fn new(id: impl Into<String>, rect: LayoutRect, z: i32) -> Self {
        Self {
            id: id.into(),
            rect,
            z,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum VideoLayout {
    #[default]
    Full,
    SideBySide,
    TopBottom,
    PipBottomRight,
    PipBottomLeft,
    PipTopRight,
    PipTopLeft,
    Grid2x2,
    Grid3x3,
    Grid4x4,
    MainSidebar,
    ThreeUp,
    ThreeStack,
}

impl VideoLayout {
    const PIP_INSET: f64 = 0.28;
    const PIP_MARGIN: f64 = 0.035;

    pub fn slots(&self) -> Vec<LayoutSlot> {
        match self {
            Self::Full => vec![LayoutSlot::new("main", LayoutRect::new(0.0, 0.0, 1.0, 1.0), 0)],
            Self::SideBySide => vec![
                LayoutSlot::new("left", LayoutRect::new(0.0, 0.0, 0.5, 1.0), 0),
                LayoutSlot::new("right", LayoutRect::new(0.5, 0.0, 0.5, 1.0), 0),
            ],
            Self::TopBottom => vec![
                LayoutSlot::new("top", LayoutRect::new(0.0, 0.0, 1.0, 0.5), 0),
                LayoutSlot::new("bottom", LayoutRect::new(0.0, 0.5, 1.0, 0.5), 0),
            ],
            Self::PipBottomRight => Self::pip(
                1.0 - Self::PIP_MARGIN - Self::PIP_INSET,
                1.0 - Self::PIP_MARGIN - Self::PIP_INSET,
            ),
            Self::PipBottomLeft => {
                Self::pip(Self::PIP_MARGIN, 1.0 - Self::PIP_MARGIN - Self::PIP_INSET)
            }
            Self::PipTopRight => {
                Self::pip(1.0 - Self::PIP_MARGIN - Self::PIP_INSET, Self::PIP_MARGIN)
            }
            Self::PipTopLeft => Self::pip(Self::PIP_MARGIN, Self::PIP_MARGIN),
            Self::Grid2x2 => Self::grid(2, 2),
            Self::Grid3x3 => Self::grid(3, 3),
            Self::Grid4x4 => Self::grid(4, 4),
            Self::MainSidebar => vec![
                LayoutSlot::new("main", LayoutRect::new(0.0, 0.0, 0.7, 1.0), 0),
                LayoutSlot::new("sidebar", LayoutRect::new(0.7, 0.0, 0.3, 1.0), 0),
            ],
            Self::ThreeUp => {
                let third = 1.0 / 3.0;
                vec![
                    LayoutSlot::new("left", LayoutRect::new(0.0, 0.0, third, 1.0), 0),
                    LayoutSlot::new("center", LayoutRect::new(third, 0.0, third, 1.0), 0),
                    LayoutSlot::new("right", LayoutRect::new(third * 2.0, 0.0, third, 1.0), 0),
                ]
            }
            Self::ThreeStack => {
                let third = 1.0 / 3.0;
                vec![
                    LayoutSlot::new("top", LayoutRect::new(0.0, 0.0, 1.0, third), 0),
                    LayoutSlot::new("middle", LayoutRect::new(0.0, third, 1.0, third), 0),
                    LayoutSlot::new("bottom", LayoutRect::new(0.0, third * 2.0, 1.0, third), 0),
                ]
            }
        }
    }

    fn grid(rows: usize, columns: usize) -> Vec<LayoutSlot> {
        let width = 1.0 / columns as f64;
        let height = 1.0 / rows as f64;
        let mut slots = Vec::with_capacity(rows * columns);
        for row in 0..rows {
            for col in 0..columns {
                slots.push(LayoutSlot::new(
                    format!("r{}c{}", row + 1, col + 1),
                    LayoutRect::new(col as f64 * width, row as f64 * height, width, height),
                    0,
                ));
            }
        }
        slots
    }

    fn pip(inset_x: f64, inset_y: f64) -> Vec<LayoutSlot> {
        vec![
            LayoutSlot::new("main", LayoutRect::new(0.0, 0.0, 1.0, 1.0), 0),
            LayoutSlot::new(
                "inset",
                LayoutRect::new(inset_x, inset_y, Self::PIP_INSET, Self::PIP_INSET),
                1,
            ),
        ]
    }
}
