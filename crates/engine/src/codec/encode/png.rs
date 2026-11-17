// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Sean Hawkey

//! PNG: lossless with oxipng, or palette-reduced with imagequant for the smaller looks.

use std::io::Cursor;

use image::{codecs::png::{CompressionType, FilterType, PngEncoder}, ExtendedColorType, ImageEncoder, RgbaImage};

use super::{flatten_rgb, EncodeOptions, EncodeQuality, Encoder};
use crate::codec::Format;
use crate::error::{Error, Result};

pub struct Png;

fn err(reason: impl ToString) -> Error {
    Error::Encode { format: "PNG", reason: reason.to_string() }
}

impl Encoder for Png {
    fn format(&self) -> Format {
        Format::Png
    }
    fn lossy(&self) -> bool {
        true
    }
    fn encode(&self, px: &RgbaImage, alpha: bool, q: EncodeQuality, opts: &EncodeOptions) -> Result<Vec<u8>> {
        let raw = match q {
            EncodeQuality::Lossy(q) if q < 100 => quantised(px, q).or_else(|_| truecolour(px, alpha, opts))?,
            _ => truecolour(px, alpha, opts)?,
        };
        if opts.fast {
            return Ok(raw);
        }
        let mut o = oxipng::Options::from_preset(2);
        o.strip = oxipng::StripChunks::Safe;
        oxipng::optimize_from_memory(&raw, &o).map_err(err)
    }
}

fn truecolour(px: &RgbaImage, alpha: bool, opts: &EncodeOptions) -> Result<Vec<u8>> {
    let mut out = Vec::new();
    let enc = PngEncoder::new_with_quality(Cursor::new(&mut out), CompressionType::Fast, FilterType::Adaptive);
    if alpha {
        enc.write_image(px.as_raw(), px.width(), px.height(), ExtendedColorType::Rgba8).map_err(err)?;
    } else {
        enc.write_image(&flatten_rgb(px, opts.matte), px.width(), px.height(), ExtendedColorType::Rgb8).map_err(err)?;
    }
    Ok(out)
}

/// Reduce to a palette of up to 256 colours. Fails (and the caller falls back to
/// lossless) when the image can't reach the requested quality with a palette.
fn quantised(px: &RgbaImage, q: u8) -> Result<Vec<u8>> {
    let mut attr = imagequant::new();
    attr.set_quality(q.saturating_sub(20), q).map_err(err)?;
    attr.set_speed(4).map_err(err)?;
    let pixels: Vec<imagequant::RGBA> = px.as_raw().as_chunks::<4>().0.iter().map(|p| imagequant::RGBA::new(p[0], p[1], p[2], p[3])).collect();
    let mut img = attr.new_image(pixels, px.width() as usize, px.height() as usize, 0.0).map_err(err)?;
    let mut res = attr.quantize(&mut img).map_err(err)?;
    res.set_dithering_level(1.0).map_err(err)?;
    let (palette, indices) = res.remapped(&mut img).map_err(err)?;

    let mut out = Vec::new();
    {
        let mut enc = png::Encoder::new(Cursor::new(&mut out), px.width(), px.height());
        enc.set_color(png::ColorType::Indexed);
        enc.set_depth(png::BitDepth::Eight);
        enc.set_palette(palette.iter().flat_map(|c| [c.r, c.g, c.b]).collect::<Vec<u8>>());
        let trns: Vec<u8> = palette.iter().map(|c| c.a).collect();
        if trns.iter().any(|&a| a != 255) {
            enc.set_trns(trns);
        }
        let mut w = enc.write_header().map_err(err)?;
        w.write_image_data(&indices).map_err(err)?;
    }
    Ok(out)
}
