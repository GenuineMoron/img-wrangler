// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Sean Hawkey

//! AVIF via ravif (rav1e), pure Rust.

use image::RgbaImage;
use rgb::FromSlice;

use super::{EncodeOptions, EncodeQuality, Encoder};
use crate::codec::Format;
use crate::error::{Error, Result};

pub struct Avif;

impl Encoder for Avif {
    fn format(&self) -> Format {
        Format::Avif
    }
    fn lossy(&self) -> bool {
        true
    }
    fn encode(&self, px: &RgbaImage, alpha: bool, q: EncodeQuality, opts: &EncodeOptions) -> Result<Vec<u8>> {
        let quality = match q {
            EncodeQuality::Lossless => 100.0,
            EncodeQuality::Lossy(q) => f32::from(q.clamp(1, 100)),
        };
        let enc = ravif::Encoder::new()
            .with_quality(quality)
            .with_alpha_quality(quality)
            .with_speed(if opts.fast { 9 } else { 6 })
            // Batches already run one image per core; one thread each avoids oversubscription.
            .with_num_threads(Some(1));
        let (w, h) = (px.width() as usize, px.height() as usize);
        let out = if alpha {
            enc.encode_rgba(ravif::Img::new(px.as_raw().as_rgba(), w, h))
        } else {
            let rgb: Vec<rgb::RGB8> = px.as_raw().as_rgba().iter().map(|p| rgb::RGB8::new(p.r, p.g, p.b)).collect();
            enc.encode_rgb(ravif::Img::new(&rgb[..], w, h))
        };
        out.map(|e| e.avif_file).map_err(|e| Error::Encode { format: "AVIF", reason: e.to_string() })
    }
}
