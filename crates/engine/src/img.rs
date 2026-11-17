// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Sean Hawkey

//! The image every stage works on: 8-bit RGBA in sRGB, plus what encoding needs later.

use image::RgbaImage;
use serde::{Deserialize, Serialize};

use crate::codec::Format;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub struct Dims {
    pub w: u32,
    pub h: u32,
}

impl Dims {
    pub const fn new(w: u32, h: u32) -> Self {
        Self { w, h }
    }
    pub fn long(self) -> u32 {
        self.w.max(self.h)
    }
    pub fn short(self) -> u32 {
        self.w.min(self.h)
    }
    pub fn area(self) -> u64 {
        u64::from(self.w) * u64::from(self.h)
    }
    pub fn aspect(self) -> f64 {
        f64::from(self.w) / f64::from(self.h.max(1))
    }
    pub fn swapped(self) -> Self {
        Self { w: self.h, h: self.w }
    }
}

impl std::fmt::Display for Dims {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}×{}", self.w, self.h)
    }
}

#[derive(Debug, Clone)]
pub struct Img {
    pub px: RgbaImage,
    /// The format the file was decoded from.
    pub source: Format,
    /// The source declared an alpha channel. Use `has_transparency` to know if it is used.
    pub alpha: bool,
    /// Raw EXIF (TIFF structure), already rotated upright; orientation is reset on output.
    pub exif: Option<Vec<u8>>,
}

impl Img {
    pub fn new(px: RgbaImage, source: Format) -> Self {
        Self { px, source, alpha: false, exif: None }
    }

    pub fn dims(&self) -> Dims {
        Dims::new(self.px.width(), self.px.height())
    }

    /// True when any pixel is not fully opaque.
    pub fn has_transparency(&self) -> bool {
        self.alpha && self.px.pixels().any(|p| p.0[3] != 255)
    }
}
