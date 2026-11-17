// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Sean Hawkey

//! Rotate add-on: quarter turns and mirror flips, applied before sizing.

use serde::{Deserialize, Serialize};

use super::{Cx, Planned, Stage};
use crate::error::Result;
use crate::img::Img;
use crate::ops::rotate::{flip, turn};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[serde(default)]
pub struct Rotate {
    pub enabled: bool,
    /// 0, 90, 180 or 270 degrees clockwise.
    pub turn: u32,
    pub flip_horizontal: bool,
    pub flip_vertical: bool,
}

impl Default for Rotate {
    fn default() -> Self {
        Self { enabled: true, turn: 0, flip_horizontal: false, flip_vertical: false }
    }
}

impl Stage for Rotate {
    fn id(&self) -> &'static str {
        "rotate"
    }
    fn order(&self) -> u16 {
        20
    }
    fn active(&self) -> bool {
        self.enabled && (!self.turn.is_multiple_of(360) || self.flip_horizontal || self.flip_vertical)
    }
    fn plan(&self, input: Planned) -> Planned {
        if self.turn % 180 == 90 { Planned { dims: input.dims.swapped(), ..input } } else { input }
    }
    fn apply(&self, mut img: Img, _cx: &mut Cx) -> Result<Img> {
        img.px = flip(turn(&img.px, self.turn), self.flip_horizontal, self.flip_vertical);
        Ok(img)
    }
}
