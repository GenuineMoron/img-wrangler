// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Sean Hawkey

//! Sharpen add-on: output sharpening after a downscale.

use serde::{Deserialize, Serialize};

use super::{Cx, Planned, Stage};
use crate::error::Result;
use crate::img::Img;
use crate::ops::sharpen::unsharp;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[serde(default)]
pub struct Sharpen {
    pub enabled: bool,
    /// Strength in percent, 0–200.
    pub amount: u16,
    /// Radius in pixels.
    pub radius: f32,
    /// Ignore differences smaller than this (0–255) so flat areas stay clean.
    pub threshold: u8,
}

impl Default for Sharpen {
    fn default() -> Self {
        Self { enabled: true, amount: 40, radius: 0.6, threshold: 2 }
    }
}

impl Stage for Sharpen {
    fn id(&self) -> &'static str {
        "sharpen"
    }
    fn order(&self) -> u16 {
        50
    }
    fn active(&self) -> bool {
        self.enabled && self.amount > 0
    }
    fn plan(&self, input: Planned) -> Planned {
        input
    }
    fn apply(&self, mut img: Img, _cx: &mut Cx) -> Result<Img> {
        img.px = unsharp(&img.px, f32::from(self.amount) / 100.0, self.radius, self.threshold);
        Ok(img)
    }
}
