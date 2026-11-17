// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Sean Hawkey

//! Small cached previews for the batch sheet. JPEGs decode at 1/2–1/8 scale, which is many
//! times faster than a full decode.

use std::fs;
use std::io::{BufReader, Cursor};
use std::path::{Path, PathBuf};

use image::{imageops, DynamicImage, RgbaImage};

use crate::codec::{decode_path, Format};
use crate::error::{Error, Result};

/// Bump to invalidate every cached thumbnail when the method changes.
const VERSION: u32 = 1;

fn key(path: &Path, max: u32) -> Result<String> {
    let m = fs::metadata(path).map_err(|e| Error::io(path, e))?;
    let modified = m.modified().ok().and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok()).map_or(0, |d| d.as_nanos());
    let canonical = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    let mut h = blake3::Hasher::new();
    h.update(canonical.to_string_lossy().as_bytes());
    h.update(&m.len().to_le_bytes());
    h.update(&modified.to_le_bytes());
    h.update(&max.to_le_bytes());
    h.update(&VERSION.to_le_bytes());
    Ok(h.finalize().to_hex()[..32].to_string())
}

fn orientation(path: &Path) -> Option<image::metadata::Orientation> {
    let f = fs::File::open(path).ok()?;
    let exif = exif::Reader::new().read_from_container(&mut BufReader::new(f)).ok()?;
    let v = exif.get_field(exif::Tag::Orientation, exif::In::PRIMARY)?.value.get_uint(0)?;
    image::metadata::Orientation::from_exif(v as u8)
}

fn fast_jpeg(path: &Path, max: u32) -> Option<RgbaImage> {
    let bytes = fs::read(path).ok()?;
    let mut dec = jpeg_decoder::Decoder::new(Cursor::new(bytes));
    dec.read_info().ok()?;
    let (w, h) = dec.scale(max as u16, max as u16).ok()?;
    let data = dec.decode().ok()?;
    let info = dec.info()?;
    let rgba: Vec<u8> = match info.pixel_format {
        jpeg_decoder::PixelFormat::RGB24 => data.as_chunks::<3>().0.iter().flat_map(|p| [p[0], p[1], p[2], 255]).collect(),
        jpeg_decoder::PixelFormat::L8 => data.iter().flat_map(|&g| [g, g, g, 255]).collect(),
        _ => return None,
    };
    let mut img = DynamicImage::ImageRgba8(RgbaImage::from_raw(u32::from(w), u32::from(h), rgba)?);
    if let Some(o) = orientation(path) {
        img.apply_orientation(o);
    }
    Some(img.into_rgba8())
}

/// Path to a cached WebP thumbnail no larger than `max` on its long side, making it if needed.
pub fn thumbnail(path: &Path, max: u32, cache: &Path) -> Result<PathBuf> {
    let out = cache.join(format!("{}.webp", key(path, max)?));
    if out.exists() {
        return Ok(out);
    }
    let px = match Format::from_path(path) {
        Some(Format::Jpeg) => fast_jpeg(path, max),
        _ => None,
    };
    let px = match px {
        Some(p) => p,
        None => decode_path(path)?.px,
    };
    let (w, h) = px.dimensions();
    let s = (max as f32 / w.max(h) as f32).min(1.0);
    let small = imageops::thumbnail(&px, ((w as f32 * s) as u32).max(1), ((h as f32 * s) as u32).max(1));
    let bytes = webp::Encoder::from_rgba(small.as_raw(), small.width(), small.height()).encode(74.0).to_vec();
    fs::create_dir_all(cache).map_err(|e| Error::io(cache, e))?;
    let tmp = out.with_extension("tmp");
    fs::write(&tmp, bytes).map_err(|e| Error::io(&tmp, e))?;
    fs::rename(&tmp, &out).map_err(|e| Error::io(&out, e))?;
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codec::encode::{encoder_for, EncodeOptions, EncodeQuality};

    #[test]
    fn makes_and_reuses_thumbnails() {
        let d = tempfile::tempdir().unwrap();
        let src = d.path().join("big.jpg");
        let px = RgbaImage::from_fn(1600, 1200, |x, y| image::Rgba([(x % 256) as u8, (y % 256) as u8, 60, 255]));
        fs::write(&src, encoder_for(Format::Jpeg).unwrap().encode(&px, false, EncodeQuality::Lossy(85), &EncodeOptions::default()).unwrap()).unwrap();
        let t = thumbnail(&src, 256, &d.path().join("cache")).unwrap();
        let back = crate::codec::decode_path(&t).unwrap();
        assert!(back.dims().long() <= 256);
        assert_eq!(thumbnail(&src, 256, &d.path().join("cache")).unwrap(), t);
    }
}
