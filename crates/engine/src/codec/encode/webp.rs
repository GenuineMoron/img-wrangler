// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Sean Hawkey

//! WebP via libwebp, lossy and lossless.

use image::RgbaImage;

use super::{flatten_rgb, EncodeOptions, EncodeQuality, Encoder};
use crate::codec::Format;
use crate::error::{Error, Result};

pub struct Webp;

impl Encoder for Webp {
    fn format(&self) -> Format {
        Format::Webp
    }
    fn lossy(&self) -> bool {
        true
    }
    fn encode(&self, px: &RgbaImage, alpha: bool, q: EncodeQuality, opts: &EncodeOptions) -> Result<Vec<u8>> {
        let (w, h) = (px.width(), px.height());
        let rgb;
        let enc = if alpha {
            webp::Encoder::from_rgba(px.as_raw(), w, h)
        } else {
            rgb = flatten_rgb(px, opts.matte);
            webp::Encoder::from_rgb(&rgb, w, h)
        };
        let mut config = webp::WebPConfig::new().map_err(|_| Error::Encode { format: "WebP", reason: "could not set up the encoder".into() })?;
        match q {
            EncodeQuality::Lossless => {
                config.lossless = 1;
                config.quality = if opts.fast { 25.0 } else { 75.0 };
                config.exact = 0;
            }
            EncodeQuality::Lossy(q) => {
                config.quality = f32::from(q.clamp(1, 100));
                config.alpha_quality = i32::from(q.clamp(1, 100));
                config.use_sharp_yuv = i32::from(!opts.fast);
            }
        }
        config.method = if opts.fast { 2 } else { 5 };
        let mem = enc.encode_advanced(&config).map_err(|e| Error::Encode { format: "WebP", reason: format!("{e:?}") })?;
        Ok(mem.to_vec())
    }
}
