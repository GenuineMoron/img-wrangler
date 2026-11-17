// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Sean Hawkey

//! JPEG via MozJPEG: trellis quantisation and optimised progressive scans.

use image::RgbaImage;

use super::{flatten_rgb, EncodeOptions, EncodeQuality, Encoder};
use crate::codec::Format;
use crate::error::{Error, Result};

pub struct Jpeg;

impl Encoder for Jpeg {
    fn format(&self) -> Format {
        Format::Jpeg
    }
    fn lossy(&self) -> bool {
        true
    }
    fn encode(&self, px: &RgbaImage, _alpha: bool, q: EncodeQuality, opts: &EncodeOptions) -> Result<Vec<u8>> {
        let quality = match q {
            EncodeQuality::Lossless => 100,
            EncodeQuality::Lossy(q) => q.clamp(1, 100),
        };
        let rgb = flatten_rgb(px, opts.matte);
        let (w, h) = (px.width() as usize, px.height() as usize);
        let (progressive, fast) = (opts.progressive, opts.fast);
        // MozJPEG reports errors by unwinding; keep that inside this function.
        std::panic::catch_unwind(move || -> std::io::Result<Vec<u8>> {
            let mut c = mozjpeg::Compress::new(mozjpeg::ColorSpace::JCS_RGB);
            c.set_size(w, h);
            if fast {
                c.set_fastest_defaults();
            }
            c.set_quality(f32::from(quality));
            // Full-resolution colour keeps red and blue edges clean at the top of the range.
            if quality >= 90 {
                c.set_chroma_sampling_pixel_sizes((1, 1), (1, 1));
            }
            if progressive && !fast {
                c.set_progressive_mode();
                c.set_optimize_scans(true);
            }
            c.set_optimize_coding(true);
            let mut started = c.start_compress(Vec::new())?;
            started.write_scanlines(&rgb)?;
            started.finish()
        })
        .map_err(|_| Error::Encode { format: "JPEG", reason: "the encoder stopped unexpectedly".into() })?
        .map_err(|e| Error::Encode { format: "JPEG", reason: e.to_string() })
    }
}
