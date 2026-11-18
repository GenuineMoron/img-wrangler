// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Sean Hawkey

//! Pictures for the monitor. The exact result needs the full encode, which can take most of a
//! second on a large photo, so the app first shows a draft: the recipe's framing (resize, crop,
//! pad, trim, rotate, watermark) at screen size, ready in a fraction of that. The exact result
//! replaces it when it's done.

use std::borrow::Cow;
use std::path::Path;

use image::RgbaImage;

use crate::codec::Format;
use crate::error::{Error, Result};
use crate::img::Dims;
use crate::modules::{self, Cx};
use crate::ops::resize::{resize, Filter};
use crate::process::{output_format, read_source};
use crate::recipe::Recipe;

/// Longest side of a screen picture: enough for a monitor on a 5K display.
pub const SCREEN: u32 = 1600;

/// What the recipe does to an image, at screen size.
pub struct Draft {
    /// The original and the result, as WebP no larger than `SCREEN`.
    pub before: Vec<u8>,
    pub after: Vec<u8>,
    pub before_bytes: u64,
    pub source_dims: Dims,
    /// The result's real size in pixels.
    pub dims: Dims,
    pub format: Format,
}

pub fn draft(path: &Path, recipe: &Recipe, cx: &mut Cx) -> Result<Draft> {
    let (original, img) = read_source(path)?;
    let source_dims = img.dims();
    let format = output_format(recipe, img.source);
    let before = screen_webp(&img.px)?;
    cx.check()?;
    let out = if recipe.is_passthrough() { img } else { modules::apply(&modules::stages(&recipe.size, &recipe.addons), img, cx)? };
    cx.check()?;
    Ok(Draft { before, after: screen_webp(&out.px)?, before_bytes: original.len() as u64, source_dims, dims: out.dims(), format })
}

/// An image shrunk to screen size and saved as a quick WebP, transparency kept.
pub fn screen_webp(px: &RgbaImage) -> Result<Vec<u8>> {
    let (w, h) = px.dimensions();
    let s = (SCREEN as f64 / f64::from(w.max(h))).min(1.0);
    let small: Cow<RgbaImage> = if s < 1.0 {
        Cow::Owned(resize(px, ((f64::from(w) * s).round() as u32).max(1), ((f64::from(h) * s).round() as u32).max(1), Filter::Bilinear))
    } else {
        Cow::Borrowed(px)
    };
    let mut config = webp::WebPConfig::new().map_err(|_| Error::Encode { format: "WebP", reason: "could not set up the encoder".into() })?;
    config.quality = 86.0;
    config.method = 0;
    let mem = webp::Encoder::from_rgba(small.as_raw(), small.width(), small.height())
        .encode_advanced(&config)
        .map_err(|e| Error::Encode { format: "WebP", reason: format!("{e:?}") })?;
    Ok(mem.to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codec::encode::{encoder_for, EncodeOptions, EncodeQuality};
    use crate::modules::Size;

    #[test]
    fn draft_shows_the_framing_at_screen_size() {
        let dir = tempfile::tempdir().unwrap();
        let src = dir.path().join("big.jpg");
        let px = RgbaImage::from_fn(4000, 3000, |x, y| image::Rgba([(x % 251) as u8, (y % 241) as u8, 90, 255]));
        std::fs::write(&src, encoder_for(Format::Jpeg).unwrap().encode(&px, false, EncodeQuality::Lossy(80), &EncodeOptions::default()).unwrap()).unwrap();
        let recipe = Recipe { size: Size { long_side: Some(2000), ..Size::default() }, ..Recipe::default() };
        let d = draft(&src, &recipe, &mut Cx::default()).unwrap();
        assert_eq!(d.source_dims, Dims::new(4000, 3000));
        assert_eq!(d.dims, Dims::new(2000, 1500));
        let shown = crate::codec::decode_bytes(&d.before, Some(Format::Webp)).unwrap();
        assert_eq!(shown.px.dimensions(), (SCREEN, 1200));
        let after = crate::codec::decode_bytes(&d.after, Some(Format::Webp)).unwrap();
        assert_eq!(after.px.dimensions(), (SCREEN, 1200));
    }
}
