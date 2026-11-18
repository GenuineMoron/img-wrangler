// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Sean Hawkey

//! Encoders. Each format implements `Encoder`; `encoder_for` is the single routing table.

mod avif;
mod jpeg;
mod png;
mod simple;
mod webp;

use image::RgbaImage;

use super::Format;
use crate::colour::Rgba;
use crate::error::{Error, Result};

/// How hard to compress, in the encoder's own terms.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EncodeQuality {
    Lossless,
    /// 1-100, higher is better.
    Lossy(u8),
}

#[derive(Debug, Clone, Copy)]
pub struct EncodeOptions {
    /// Colour that transparent areas become in formats without alpha.
    pub matte: Rgba,
    pub progressive: bool,
    /// Trade compression for speed; used while searching for a quality, never for the final file.
    pub fast: bool,
}

impl Default for EncodeOptions {
    fn default() -> Self {
        Self { matte: Rgba::WHITE, progressive: true, fast: false }
    }
}

pub trait Encoder: Send + Sync {
    fn format(&self) -> Format;
    /// Whether `EncodeQuality::Lossy` changes the result; quality searches only run when it does.
    fn lossy(&self) -> bool;
    fn encode(&self, px: &RgbaImage, alpha: bool, q: EncodeQuality, opts: &EncodeOptions) -> Result<Vec<u8>>;
}

pub fn encoder_for(format: Format) -> Result<&'static dyn Encoder> {
    Ok(match format {
        Format::Jpeg => &jpeg::Jpeg,
        Format::Png => &png::Png,
        Format::Webp => &webp::Webp,
        Format::Avif => &avif::Avif,
        Format::Gif => &simple::GIF,
        Format::Tiff => &simple::TIFF,
        Format::Bmp => &simple::BMP,
        Format::Ico => &simple::ICO,
        Format::Qoi => &simple::QOI,
        Format::Tga => &simple::TGA,
        Format::Pnm => &simple::PNM,
        other => return Err(Error::Unsupported(format!("saving as {}", other.name()))),
    })
}

/// Composite onto the matte colour and drop alpha, for formats that have none.
pub(crate) fn flatten_rgb(px: &RgbaImage, matte: Rgba) -> Vec<u8> {
    let [mr, mg, mb, _] = matte.0;
    let mut out = Vec::with_capacity(px.as_raw().len() / 4 * 3);
    for p in px.as_raw().as_chunks::<4>().0 {
        let a = u32::from(p[3]);
        let mix = |c: u8, m: u8| ((u32::from(c) * a + u32::from(m) * (255 - a) + 127) / 255) as u8;
        out.extend_from_slice(&[mix(p[0], mr), mix(p[1], mg), mix(p[2], mb)]);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codec::decode_bytes;

    fn sample(alpha: bool) -> RgbaImage {
        RgbaImage::from_fn(96, 64, |x, y| image::Rgba([(x * 2) as u8, (y * 3) as u8, ((x + y) % 255) as u8, if alpha && x < 20 { 0 } else { 255 }]))
    }

    #[test]
    fn every_encoder_round_trips() {
        for f in crate::codec::formats().iter().filter(|f| f.encode) {
            let enc = encoder_for(f.id).unwrap();
            let px = sample(f.alpha);
            let q = if enc.lossy() { EncodeQuality::Lossy(80) } else { EncodeQuality::Lossless };
            let bytes = enc.encode(&px, f.alpha, q, &EncodeOptions::default()).unwrap_or_else(|e| panic!("{:?}: {e}", f.id));
            let back = decode_bytes(&bytes, Some(f.id)).unwrap_or_else(|e| panic!("{:?} decode: {e}", f.id));
            assert_eq!(back.dims(), crate::img::Dims::new(96, 64), "{:?}", f.id);
        }
    }

    #[test]
    fn higher_quality_is_bigger() {
        for fmt in [Format::Jpeg, Format::Webp, Format::Avif] {
            let enc = encoder_for(fmt).unwrap();
            let px = sample(false);
            let lo = enc.encode(&px, false, EncodeQuality::Lossy(30), &EncodeOptions::default()).unwrap().len();
            let hi = enc.encode(&px, false, EncodeQuality::Lossy(95), &EncodeOptions::default()).unwrap().len();
            assert!(hi > lo, "{fmt:?}: {hi} <= {lo}");
        }
    }

    #[test]
    fn flatten_uses_matte() {
        let px = RgbaImage::from_pixel(1, 1, image::Rgba([0, 0, 0, 0]));
        assert_eq!(flatten_rgb(&px, Rgba::WHITE), vec![255, 255, 255]);
    }
}
