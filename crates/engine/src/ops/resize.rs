// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Sean Hawkey

//! High-quality SIMD resampling with alpha handled correctly (premultiplied while filtering).

use fast_image_resize::{images::{Image, ImageRef}, FilterType, PixelType, ResizeAlg, ResizeOptions, Resizer};
use image::RgbaImage;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[serde(rename_all = "snake_case")]
pub enum Filter {
    /// Sharpest general-purpose filter; the default.
    #[default]
    Lanczos3,
    /// Softer, with less ringing on hard edges.
    Mitchell,
    CatmullRom,
    Bilinear,
    /// Keeps hard pixel edges, for pixel art and icons.
    Nearest,
}

impl Filter {
    fn alg(self) -> ResizeAlg {
        match self {
            Filter::Lanczos3 => ResizeAlg::Convolution(FilterType::Lanczos3),
            Filter::Mitchell => ResizeAlg::Convolution(FilterType::Mitchell),
            Filter::CatmullRom => ResizeAlg::Convolution(FilterType::CatmullRom),
            Filter::Bilinear => ResizeAlg::Convolution(FilterType::Bilinear),
            Filter::Nearest => ResizeAlg::Nearest,
        }
    }
}

pub fn resize(px: &RgbaImage, w: u32, h: u32, filter: Filter) -> RgbaImage {
    let (w, h) = (w.max(1), h.max(1));
    if (w, h) == px.dimensions() {
        return px.clone();
    }
    let src = ImageRef::new(px.width(), px.height(), px.as_raw(), PixelType::U8x4).expect("buffer matches its dimensions");
    let mut dst = Image::new(w, h, PixelType::U8x4);
    Resizer::new()
        .resize(&src, &mut dst, &ResizeOptions::new().resize_alg(filter.alg()))
        .expect("same pixel type on both sides");
    RgbaImage::from_raw(w, h, dst.into_vec()).expect("destination size")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resizes_and_keeps_flat_colour() {
        let px = RgbaImage::from_pixel(400, 300, image::Rgba([200, 100, 50, 255]));
        let out = resize(&px, 100, 75, Filter::Lanczos3);
        assert_eq!(out.dimensions(), (100, 75));
        assert_eq!(out.get_pixel(50, 37).0, [200, 100, 50, 255]);
    }

    #[test]
    fn transparent_edges_do_not_darken() {
        // A white square on transparent black must stay white, not grey, after filtering.
        let px = RgbaImage::from_fn(64, 64, |x, _| if x < 32 { image::Rgba([255, 255, 255, 255]) } else { image::Rgba([0, 0, 0, 0]) });
        let out = resize(&px, 16, 16, Filter::Lanczos3);
        let edge = out.get_pixel(7, 8).0;
        assert!(edge[0] > 240, "{edge:?}");
    }
}
