// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Sean Hawkey

//! One image through a recipe: decode, stages, encode, metadata, and the never-bigger rule.

use std::path::Path;

use crate::codec::metadata::{attach_exif, exif_for_output};
use crate::codec::{decode_bytes, decode_path, Format};
use crate::error::{Error, Result};
use crate::img::{Dims, Img};
use crate::modules::{self, Cx};
use crate::quality::{self, Encoded};
use crate::recipe::{Look, OutputFormat, Quality, Recipe};

#[derive(Debug, Clone)]
pub struct Outcome {
    pub bytes: Vec<u8>,
    pub format: Format,
    pub dims: Dims,
    /// The original bytes were kept because the result wouldn't have been smaller.
    pub kept_original: bool,
    pub over_limit: bool,
    pub downscaled: bool,
    pub quality: Option<u8>,
    pub score: Option<f64>,
}

/// The format a file will be written in. Deterministic from the source format, so a plan and
/// a run always agree on names.
pub fn output_format(recipe: &Recipe, source: Format) -> Format {
    if recipe.format.auto {
        return match source {
            Format::Png | Format::Gif | Format::Svg | Format::Bmp | Format::Ico | Format::Tga | Format::Qoi => Format::Png,
            _ => Format::Webp,
        };
    }
    recipe.format.id.resolve(source)
}

pub fn process_path(path: &Path, recipe: &Recipe, cx: &mut Cx) -> Result<Outcome> {
    let original = std::fs::read(path).map_err(|e| Error::io(path, e))?;
    let img = if Format::sniff(&original[..original.len().min(64)]) == Some(Format::Heic) || Format::from_path(path) == Some(Format::Heic) {
        decode_path(path)?
    } else {
        decode_bytes(&original, Format::from_path(path))?
    };
    process_img(img, &original, recipe, cx)
}

pub fn process_img(img: Img, original: &[u8], recipe: &Recipe, cx: &mut Cx) -> Result<Outcome> {
    let source = img.source;
    let source_dims = img.dims();
    if recipe.is_passthrough() {
        return Ok(Outcome { bytes: original.to_vec(), format: source, dims: source_dims, kept_original: true, over_limit: false, downscaled: false, quality: None, score: None });
    }
    let stages = modules::stages(&recipe.size, &recipe.addons);
    let img = modules::apply(&stages, img, cx)?;
    cx.check()?;
    let format = output_format(recipe, source);
    // "Untouched" can't hold once pixels change; the closest honest choice is the top look.
    let q = if recipe.quality.look == Look::Untouched { Quality { look: Look::Pristine, ..recipe.quality } } else { recipe.quality };
    let Encoded { bytes, dims, quality, score, over_limit, downscaled, .. } = quality::encode(&img.px, img.has_transparency(), format, &q, recipe.format.matte)?;
    let exif = exif_for_output(img.exif.as_deref(), recipe.metadata.keep);
    let bytes = attach_exif(bytes, format, exif.as_deref());

    let same_file_shape = format == source && dims == source_dims;
    if recipe.quality.never_bigger && same_file_shape && bytes.len() >= original.len() {
        return Ok(Outcome { bytes: original.to_vec(), format, dims, kept_original: true, over_limit: false, downscaled: false, quality: None, score: None });
    }
    Ok(Outcome { bytes, format, dims, kept_original: false, over_limit, downscaled, quality, score })
}

/// Whether a recipe's output format is the same as its source.
pub fn keeps_format(recipe: &Recipe) -> bool {
    recipe.format.id == OutputFormat::Same && !recipe.format.auto
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codec::encode::{encoder_for, EncodeOptions, EncodeQuality};
    use crate::modules::Size;
    use image::RgbaImage;

    fn jpeg(w: u32, h: u32, q: u8) -> Vec<u8> {
        let px = RgbaImage::from_fn(w, h, |x, y| image::Rgba([(x % 256) as u8, (y % 256) as u8, 90, 255]));
        encoder_for(Format::Jpeg).unwrap().encode(&px, false, EncodeQuality::Lossy(q), &EncodeOptions::default()).unwrap()
    }

    #[test]
    fn passthrough_copies_bytes() {
        let src = jpeg(64, 64, 80);
        let recipe = Recipe { quality: Quality { look: Look::Untouched, ..Quality::default() }, ..Recipe::default() };
        let out = process_img(decode_bytes(&src, None).unwrap(), &src, &recipe, &mut Cx::default()).unwrap();
        assert!(out.kept_original);
        assert_eq!(out.bytes, src);
    }

    #[test]
    fn never_bigger_keeps_small_originals() {
        // Re-saving a low-quality JPEG at quality 100 can only grow it.
        let src = jpeg(200, 150, 20);
        let recipe = Recipe { quality: Quality { look: Look::Exact(100), ..Quality::default() }, ..Recipe::default() };
        let out = process_img(decode_bytes(&src, None).unwrap(), &src, &recipe, &mut Cx::default()).unwrap();
        assert!(out.kept_original);
        assert_eq!(out.bytes, src);
    }

    #[test]
    fn resize_and_convert() {
        let src = jpeg(800, 600, 92);
        let mut recipe = Recipe { size: Size { long_side: Some(400), ..Size::default() }, ..Recipe::default() };
        recipe.format.id = OutputFormat::To(Format::Webp);
        let out = process_img(decode_bytes(&src, None).unwrap(), &src, &recipe, &mut Cx::default()).unwrap();
        assert_eq!(out.format, Format::Webp);
        assert_eq!(out.dims, Dims::new(400, 300));
        assert!(!out.kept_original);
        assert_eq!(Format::sniff(&out.bytes), Some(Format::Webp));
    }

    #[test]
    fn auto_format_is_decided_by_source() {
        let r = Recipe { format: crate::recipe::FormatChoice { auto: true, ..Default::default() }, ..Recipe::default() };
        assert_eq!(output_format(&r, Format::Jpeg), Format::Webp);
        assert_eq!(output_format(&r, Format::Png), Format::Png);
        assert_eq!(output_format(&Recipe::default(), Format::Heic), Format::Jpeg);
    }
}
