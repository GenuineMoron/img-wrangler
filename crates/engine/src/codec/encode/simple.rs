// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Sean Hawkey

//! Formats written by the `image` crate with no tuning: GIF, TIFF, BMP, ICO, QOI, TGA, PNM.

use std::io::Cursor;

use image::{imageops::FilterType, DynamicImage, ImageFormat, RgbaImage};

use super::{flatten_rgb, EncodeOptions, EncodeQuality, Encoder};
use crate::codec::Format;
use crate::error::{Error, Result};

pub struct Simple {
    format: Format,
    target: ImageFormat,
    keeps_alpha: bool,
    /// ICO can't exceed 256 px; larger images are scaled to fit.
    max_side: Option<u32>,
}

pub const GIF: Simple = Simple { format: Format::Gif, target: ImageFormat::Gif, keeps_alpha: true, max_side: None };
pub const TIFF: Simple = Simple { format: Format::Tiff, target: ImageFormat::Tiff, keeps_alpha: true, max_side: None };
pub const BMP: Simple = Simple { format: Format::Bmp, target: ImageFormat::Bmp, keeps_alpha: false, max_side: None };
pub const ICO: Simple = Simple { format: Format::Ico, target: ImageFormat::Ico, keeps_alpha: true, max_side: Some(256) };
pub const QOI: Simple = Simple { format: Format::Qoi, target: ImageFormat::Qoi, keeps_alpha: true, max_side: None };
pub const TGA: Simple = Simple { format: Format::Tga, target: ImageFormat::Tga, keeps_alpha: true, max_side: None };
pub const PNM: Simple = Simple { format: Format::Pnm, target: ImageFormat::Pnm, keeps_alpha: false, max_side: None };

impl Encoder for Simple {
    fn format(&self) -> Format {
        self.format
    }
    fn lossy(&self) -> bool {
        false
    }
    fn encode(&self, px: &RgbaImage, alpha: bool, _q: EncodeQuality, opts: &EncodeOptions) -> Result<Vec<u8>> {
        let mut img = if alpha && self.keeps_alpha {
            DynamicImage::ImageRgba8(px.clone())
        } else {
            DynamicImage::ImageRgb8(image::RgbImage::from_raw(px.width(), px.height(), flatten_rgb(px, opts.matte)).expect("rgb size"))
        };
        if let Some(max) = self.max_side.filter(|m| img.width().max(img.height()) > *m) {
            img = img.resize(max, max, FilterType::Lanczos3);
        }
        let mut out = Cursor::new(Vec::new());
        img.write_to(&mut out, self.target).map_err(|e| Error::Encode { format: self.format.name(), reason: e.to_string() })?;
        Ok(out.into_inner())
    }
}
