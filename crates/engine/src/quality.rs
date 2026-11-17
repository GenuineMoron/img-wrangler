// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Sean Hawkey

//! Choosing how hard to compress.
//!
//! A look ("balanced") aims at a SSIMULACRA 2 score, a measure that tracks what people see.
//! Searching on the whole image would be slow, so the search runs on a strip of the image's
//! most detailed crops, then the full image is encoded once at the quality found.

use image::{imageops, RgbaImage};
use ssimulacra2::{compute_frame_ssimulacra2, ColorPrimaries, Rgb, TransferCharacteristic};

use crate::codec::encode::{encoder_for, flatten_rgb, EncodeOptions, EncodeQuality, Encoder};
use crate::codec::{decode_bytes, Format};
use crate::colour::Rgba;
use crate::error::Result;
use crate::img::Dims;
use crate::ops::resize::{resize, Filter};
use crate::recipe::{Look, Quality};

/// The result of encoding one image.
#[derive(Debug, Clone)]
pub struct Encoded {
    pub bytes: Vec<u8>,
    pub format: Format,
    pub dims: Dims,
    /// Encoder quality used, for lossy formats.
    pub quality: Option<u8>,
    /// Score of the search sample at that quality, when a look was targeted.
    pub score: Option<f64>,
    /// The size limit couldn't be met.
    pub over_limit: bool,
    /// The image was shrunk to meet the size limit.
    pub downscaled: bool,
}

const CROP: u32 = 256;
const SEARCH_STEPS: usize = 6;
/// Lowest quality the size limit may go to before giving up (or shrinking, if allowed).
const LIMIT_FLOOR: u8 = 25;

fn search_range(format: Format) -> (u8, u8) {
    match format {
        Format::Avif => (20, 92),
        Format::Png => (40, 99),
        _ => (25, 96),
    }
}

/// Up to three of the most detailed `CROP`×`CROP` regions, side by side. Small images are
/// used whole.
pub fn sample(px: &RgbaImage) -> RgbaImage {
    let (w, h) = px.dimensions();
    if u64::from(w) * u64::from(h) <= u64::from(CROP * CROP * 3) || w < CROP || h < CROP {
        return px.clone();
    }
    // Score a coarse grid by local contrast and take the three busiest, non-overlapping cells.
    let cols = (w / CROP).max(1);
    let rows = (h / CROP).max(1);
    let small = imageops::thumbnail(px, cols * 8, rows * 8);
    let mut cells: Vec<(u64, u32, u32)> = Vec::new();
    for cy in 0..rows {
        for cx in 0..cols {
            let mut energy = 0u64;
            for y in cy * 8..(cy * 8 + 7) {
                for x in cx * 8..(cx * 8 + 7) {
                    let a = small.get_pixel(x, y).0;
                    let b = small.get_pixel(x + 1, y).0;
                    let c = small.get_pixel(x, y + 1).0;
                    for i in 0..3 {
                        energy += u64::from(a[i].abs_diff(b[i])) + u64::from(a[i].abs_diff(c[i]));
                    }
                }
            }
            cells.push((energy, cx, cy));
        }
    }
    cells.sort_by_key(|c| std::cmp::Reverse(c.0));
    let picked: Vec<_> = cells.into_iter().take(3).collect();
    let mut strip = RgbaImage::new(CROP * picked.len() as u32, CROP);
    for (i, (_, cx, cy)) in picked.iter().enumerate() {
        let x = (cx * CROP).min(w - CROP);
        let y = (cy * CROP).min(h - CROP);
        imageops::replace(&mut strip, &imageops::crop_imm(px, x, y, CROP, CROP).to_image(), i64::from(i as u32 * CROP), 0);
    }
    strip
}

fn to_metric(px: &RgbaImage, alpha: bool, matte: Rgba) -> Option<Rgb> {
    // Score what the viewer sees: transparency composited on the matte.
    let rgb = if alpha { flatten_rgb(px, Rgba::WHITE) } else { flatten_rgb(px, matte) };
    let data = rgb.as_chunks::<3>().0.iter().map(|p| [f32::from(p[0]) / 255.0, f32::from(p[1]) / 255.0, f32::from(p[2]) / 255.0]).collect();
    Rgb::new(data, px.width() as usize, px.height() as usize, TransferCharacteristic::SRGB, ColorPrimaries::BT709).ok()
}

/// SSIMULACRA 2 between a reference and an encoded version of it.
pub fn score(reference: &RgbaImage, encoded: &[u8], format: Format, alpha: bool, matte: Rgba) -> Option<f64> {
    let back = decode_bytes(encoded, Some(format)).ok()?;
    if back.px.dimensions() != reference.dimensions() || reference.width() < 8 || reference.height() < 8 {
        return None;
    }
    compute_frame_ssimulacra2(to_metric(reference, alpha, matte)?, to_metric(&back.px, alpha, matte)?).ok()
}

/// The lowest encoder quality whose sample reaches `target`.
fn search(enc: &dyn Encoder, px: &RgbaImage, alpha: bool, target: f64, opts: &EncodeOptions) -> Result<(u8, Option<f64>)> {
    let strip = sample(px);
    let fast = EncodeOptions { fast: true, ..*opts };
    let (mut lo, mut hi) = search_range(enc.format());
    let mut best = (hi, None);
    for _ in 0..SEARCH_STEPS {
        if lo >= hi {
            break;
        }
        let mid = lo + (hi - lo) / 2;
        let bytes = enc.encode(&strip, alpha, EncodeQuality::Lossy(mid), &fast)?;
        match score(&strip, &bytes, enc.format(), alpha, opts.matte) {
            Some(s) if s >= target => {
                hi = mid;
                best = (mid, Some(s));
            }
            Some(_) => lo = mid + 1,
            None => break,
        }
    }
    if best.1.is_none() {
        best.0 = hi;
    }
    Ok(best)
}

/// Encode an image to a format with a look and an optional size limit.
pub fn encode(px: &RgbaImage, alpha: bool, format: Format, q: &Quality, matte: Rgba) -> Result<Encoded> {
    let enc = encoder_for(format)?;
    let alpha = alpha && format.supports_alpha();
    let opts = EncodeOptions { matte, progressive: q.progressive, fast: false };
    let dims = Dims::new(px.width(), px.height());

    let (quality, score) = match q.look {
        _ if !enc.lossy() => (None, None),
        Look::Lossless | Look::Untouched => (None, None),
        Look::Exact(v) => (Some(v), None),
        look => {
            let (v, s) = search(enc, px, alpha, look.target_score().expect("targeted look"), &opts)?;
            (Some(v), s)
        }
    };
    let as_q = |v: Option<u8>| v.map_or(EncodeQuality::Lossless, EncodeQuality::Lossy);
    let mut out = Encoded { bytes: enc.encode(px, alpha, as_q(quality), &opts)?, format, dims, quality, score, over_limit: false, downscaled: false };

    let Some(max_kb) = q.max_kb else { return Ok(out) };
    let limit = u64::from(max_kb) * 1024;
    if out.bytes.len() as u64 <= limit {
        return Ok(out);
    }
    if enc.lossy() {
        // Bisect the quality down towards the floor on the full image.
        let (mut lo, mut hi) = (LIMIT_FLOOR, quality.unwrap_or(search_range(format).1));
        let mut fit: Option<(u8, Vec<u8>)> = None;
        for _ in 0..SEARCH_STEPS + 1 {
            if lo > hi {
                break;
            }
            let mid = lo + (hi - lo) / 2;
            let bytes = enc.encode(px, alpha, EncodeQuality::Lossy(mid), &opts)?;
            if bytes.len() as u64 <= limit {
                fit = Some((mid, bytes));
                lo = mid + 1;
            } else if mid == 0 {
                break;
            } else {
                hi = mid - 1;
            }
        }
        if let Some((v, bytes)) = fit {
            out.quality = Some(v);
            out.score = None;
            out.bytes = bytes;
            return Ok(out);
        }
    }
    if q.allow_downscale {
        let mut scaled = px.clone();
        let base = if enc.lossy() { EncodeQuality::Lossy(LIMIT_FLOOR.max(quality.unwrap_or(LIMIT_FLOOR).min(60))) } else { EncodeQuality::Lossless };
        for _ in 0..12 {
            let (w, h) = ((scaled.width() as f32 * 0.85) as u32, (scaled.height() as f32 * 0.85) as u32);
            if w < 16 || h < 16 {
                break;
            }
            scaled = resize(&scaled, w, h, Filter::Lanczos3);
            let bytes = enc.encode(&scaled, alpha, base, &opts)?;
            if bytes.len() as u64 <= limit {
                return Ok(Encoded { bytes, format, dims: Dims::new(w, h), quality: if let EncodeQuality::Lossy(v) = base { Some(v) } else { None }, score: None, over_limit: false, downscaled: true });
            }
        }
    }
    out.over_limit = true;
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn photo(w: u32, h: u32) -> RgbaImage {
        // Smooth gradients with a band of fine detail, like a product with texture.
        RgbaImage::from_fn(w, h, |x, y| {
            let d = if (y / 4) % 2 == 0 && x > w / 3 && x < w / 2 { 60 } else { 0 };
            image::Rgba([((x * 255) / w) as u8, ((y * 255) / h) as u8, (128 + d) as u8, 255])
        })
    }

    #[test]
    fn sample_picks_detailed_crops() {
        let s = sample(&photo(2048, 1024));
        assert_eq!(s.dimensions(), (CROP * 3, CROP));
        assert_eq!(sample(&photo(300, 200)).dimensions(), (300, 200));
    }

    #[test]
    fn higher_looks_cost_more_bytes() {
        let px = photo(640, 480);
        let enc = |look| encode(&px, false, Format::Jpeg, &Quality { look, ..Quality::default() }, Rgba::WHITE).unwrap();
        let (small, high) = (enc(Look::Small), enc(Look::High));
        assert!(high.bytes.len() > small.bytes.len());
        assert!(high.quality > small.quality);
    }

    #[test]
    fn targeted_look_reaches_its_score() {
        let px = photo(512, 512);
        let out = encode(&px, false, Format::Webp, &Quality { look: Look::Balanced, ..Quality::default() }, Rgba::WHITE).unwrap();
        let s = score(&sample(&px), &crate::codec::encode::encoder_for(Format::Webp).unwrap().encode(&sample(&px), false, EncodeQuality::Lossy(out.quality.unwrap()), &EncodeOptions::default()).unwrap(), Format::Webp, false, Rgba::WHITE).unwrap();
        assert!(s >= 73.0, "score {s}");
    }

    #[test]
    fn size_limit_is_met_or_flagged() {
        let px = photo(1200, 900);
        let q = Quality { look: Look::Pristine, max_kb: Some(60), ..Quality::default() };
        let out = encode(&px, false, Format::Jpeg, &q, Rgba::WHITE).unwrap();
        assert!(out.bytes.len() <= 60 * 1024 || out.over_limit);
        let impossible = Quality { look: Look::High, max_kb: Some(1), ..Quality::default() };
        assert!(encode(&px, false, Format::Jpeg, &impossible, Rgba::WHITE).unwrap().over_limit);
        let shrink = Quality { allow_downscale: true, ..impossible };
        let out = encode(&px, false, Format::Jpeg, &shrink, Rgba::WHITE).unwrap();
        assert!(out.downscaled || out.over_limit);
    }

    #[test]
    fn lossless_formats_skip_the_search() {
        let out = encode(&photo(100, 100), false, Format::Tiff, &Quality::default(), Rgba::WHITE).unwrap();
        assert_eq!(out.quality, None);
    }
}
