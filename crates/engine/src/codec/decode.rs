// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Sean Hawkey

//! Decoding every supported format into an upright, sRGB `Img`, and cheap header probes.

use std::io::{Cursor, Read};
use std::path::Path;
use std::sync::{Arc, OnceLock};

use image::{DynamicImage, ImageDecoder, ImageReader, Limits, RgbaImage};
use serde::Serialize;

use super::Format;
use crate::colour::convert_to_srgb;
use crate::error::{Error, Result};
use crate::img::{Dims, Img};

/// What a file is, without decoding its pixels.
#[derive(Debug, Clone, Serialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub struct Probe {
    pub format: Format,
    /// Upright dimensions when the header gives them cheaply; `None` for HEIC.
    pub dims: Option<Dims>,
    pub bytes: u64,
}

const SVG_MAX: u32 = 4096;
const SVG_MIN: u32 = 1024;

fn limits() -> Limits {
    let mut l = Limits::default();
    l.max_image_width = Some(65_535);
    l.max_image_height = Some(65_535);
    l.max_alloc = Some(4 << 30);
    l
}

fn image_format(f: Format) -> Option<image::ImageFormat> {
    Some(match f {
        Format::Jpeg => image::ImageFormat::Jpeg,
        Format::Png => image::ImageFormat::Png,
        Format::Webp => image::ImageFormat::WebP,
        Format::Gif => image::ImageFormat::Gif,
        Format::Tiff => image::ImageFormat::Tiff,
        Format::Bmp => image::ImageFormat::Bmp,
        Format::Ico => image::ImageFormat::Ico,
        Format::Qoi => image::ImageFormat::Qoi,
        Format::Tga => image::ImageFormat::Tga,
        Format::Pnm => image::ImageFormat::Pnm,
        _ => return None,
    })
}

/// Identify a file by its bytes, falling back to its extension.
fn identify(path: &Path, head: &[u8]) -> Result<Format> {
    Format::sniff(head)
        .or_else(|| Format::from_path(path))
        .ok_or_else(|| Error::Unsupported(path.extension().and_then(|e| e.to_str()).unwrap_or("these").to_uppercase()))
}

pub fn decode_path(path: &Path) -> Result<Img> {
    let bytes = std::fs::read(path).map_err(|e| Error::io(path, e))?;
    let format = identify(path, &bytes[..bytes.len().min(64)])?;
    if format == Format::Heic {
        return decode_heic(path);
    }
    decode_bytes(&bytes, Some(format))
}

pub fn decode_bytes(bytes: &[u8], hint: Option<Format>) -> Result<Img> {
    let format = Format::sniff(&bytes[..bytes.len().min(64)])
        .or(hint)
        .ok_or_else(|| Error::Decode("unrecognised file".into()))?;
    match format {
        Format::Avif => decode_avif(bytes),
        Format::Jxl => {
            let dec = jxl_oxide::integration::JxlDecoder::new(Cursor::new(bytes)).map_err(|e| Error::Decode(e.to_string()))?;
            from_decoder(dec, Format::Jxl)
        }
        Format::Svg => decode_svg(bytes),
        Format::Heic => Err(Error::Unsupported("HEIC from memory".into())),
        other => {
            let mut reader = ImageReader::with_format(Cursor::new(bytes), image_format(other).expect("raster format"));
            reader.limits(limits());
            from_decoder(reader.into_decoder()?, other)
        }
    }
}

fn from_decoder(mut dec: impl ImageDecoder, format: Format) -> Result<Img> {
    let orientation = dec.orientation().unwrap_or(image::metadata::Orientation::NoTransforms);
    let icc = dec.icc_profile().ok().flatten();
    let exif = dec.exif_metadata().ok().flatten();
    let mut dynimg = DynamicImage::from_decoder(dec)?;
    dynimg.apply_orientation(orientation);
    let alpha = dynimg.color().has_alpha();
    let mut px = dynimg.into_rgba8();
    if let Some(icc) = icc {
        convert_to_srgb(&mut px, &icc);
    }
    Ok(Img { px, source: format, alpha, exif })
}

fn decode_avif(bytes: &[u8]) -> Result<Img> {
    use avif_decode::Image;
    let image = avif_decode::Decoder::from_avif(bytes).and_then(|d| d.to_image()).map_err(|e| Error::Decode(e.to_string()))?;
    let (px, alpha) = match image {
        Image::Rgba8(v) => {
            let (buf, w, h) = v.into_contiguous_buf();
            (rgba_from(w, h, buf.into_iter().flat_map(|p| [p.r, p.g, p.b, p.a])), true)
        }
        Image::Rgb8(v) => {
            let (buf, w, h) = v.into_contiguous_buf();
            (rgba_from(w, h, buf.into_iter().flat_map(|p| [p.r, p.g, p.b, 255])), false)
        }
        Image::Rgba16(v) => {
            let (buf, w, h) = v.into_contiguous_buf();
            (rgba_from(w, h, buf.into_iter().flat_map(|p| [p.r, p.g, p.b, p.a].map(|c| (c >> 8) as u8))), true)
        }
        Image::Rgb16(v) => {
            let (buf, w, h) = v.into_contiguous_buf();
            (rgba_from(w, h, buf.into_iter().flat_map(|p| [(p.r >> 8) as u8, (p.g >> 8) as u8, (p.b >> 8) as u8, 255])), false)
        }
        Image::Gray8(v) => {
            let (buf, w, h) = v.into_contiguous_buf();
            (rgba_from(w, h, buf.into_iter().flat_map(|g| { let g = g.value(); [g, g, g, 255] })), false)
        }
        Image::Gray16(v) => {
            let (buf, w, h) = v.into_contiguous_buf();
            (rgba_from(w, h, buf.into_iter().flat_map(|g| { let g = (g.value() >> 8) as u8; [g, g, g, 255] })), false)
        }
    };
    Ok(Img { px, source: Format::Avif, alpha, exif: None })
}

fn rgba_from(w: usize, h: usize, data: impl Iterator<Item = u8>) -> RgbaImage {
    RgbaImage::from_raw(w as u32, h as u32, data.collect()).expect("buffer matches dimensions")
}

fn fonts() -> Arc<resvg::usvg::fontdb::Database> {
    static DB: OnceLock<Arc<resvg::usvg::fontdb::Database>> = OnceLock::new();
    DB.get_or_init(|| {
        let mut db = resvg::usvg::fontdb::Database::new();
        db.load_system_fonts();
        Arc::new(db)
    })
    .clone()
}

fn decode_svg(bytes: &[u8]) -> Result<Img> {
    use resvg::{tiny_skia, usvg};
    let opt = usvg::Options { fontdb: fonts(), ..usvg::Options::default() };
    let tree = usvg::Tree::from_data(bytes, &opt).map_err(|e| Error::Decode(e.to_string()))?;
    let size = tree.size();
    let long = size.width().max(size.height()).max(1.0);
    let target = (long as u32).clamp(SVG_MIN, SVG_MAX) as f32;
    let scale = target / long;
    let (w, h) = (((size.width() * scale).round() as u32).max(1), ((size.height() * scale).round() as u32).max(1));
    let mut pixmap = tiny_skia::Pixmap::new(w, h).ok_or_else(|| Error::Decode("SVG is empty".into()))?;
    resvg::render(&tree, tiny_skia::Transform::from_scale(scale, scale), &mut pixmap.as_mut());
    let data = pixmap
        .pixels()
        .iter()
        .flat_map(|p| {
            let c = p.demultiply();
            [c.red(), c.green(), c.blue(), c.alpha()]
        })
        .collect();
    let px = RgbaImage::from_raw(w, h, data).expect("pixmap size");
    Ok(Img { px, source: Format::Svg, alpha: true, exif: None })
}

/// HEIC: the HEVC codec is patent-encumbered, so by default Wrangler uses the decoder the
/// operating system already licenses (`sips` on macOS, `heif-dec` on Linux).
fn decode_heic(path: &Path) -> Result<Img> {
    #[cfg(feature = "heic")]
    {
        return decode_heic_libheif(path);
    }
    #[allow(unreachable_code)]
    {
        let tmp = std::env::temp_dir().join(format!("wrangler-heic-{}.png", uuid::Uuid::new_v4()));
        let attempts: [(&str, Vec<std::ffi::OsString>); 3] = [
            ("sips", vec!["-s".into(), "format".into(), "png".into(), path.into(), "--out".into(), tmp.clone().into()]),
            ("heif-dec", vec![path.into(), tmp.clone().into()]),
            ("heif-convert", vec![path.into(), tmp.clone().into()]),
        ];
        for (cmd, args) in attempts {
            let ok = std::process::Command::new(cmd).args(&args).stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null()).status().is_ok_and(|s| s.success());
            if ok && tmp.exists() {
                let bytes = std::fs::read(&tmp).map_err(|e| Error::io(&tmp, e));
                let _ = std::fs::remove_file(&tmp);
                let mut img = decode_bytes(&bytes?, Some(Format::Png))?;
                img.source = Format::Heic;
                return Ok(img);
            }
        }
        Err(Error::Unsupported("HEIC (install libheif, e.g. `heif-dec`, to read them on Linux)".into()))
    }
}

#[cfg(feature = "heic")]
fn decode_heic_libheif(path: &Path) -> Result<Img> {
    use libheif_rs::{ColorSpace, HeifContext, LibHeif, RgbChroma};
    let lib = LibHeif::new();
    let ctx = HeifContext::read_from_file(&path.to_string_lossy()).map_err(|e| Error::Decode(e.to_string()))?;
    let handle = ctx.primary_image_handle().map_err(|e| Error::Decode(e.to_string()))?;
    let decoded = lib.decode(&handle, ColorSpace::Rgb(RgbChroma::Rgba), None).map_err(|e| Error::Decode(e.to_string()))?;
    let planes = decoded.planes();
    let plane = planes.interleaved.ok_or_else(|| Error::Decode("HEIC has no pixels".into()))?;
    let (w, h) = (plane.width, plane.height);
    let mut data = Vec::with_capacity((w * h * 4) as usize);
    for row in 0..h as usize {
        data.extend_from_slice(&plane.data[row * plane.stride..row * plane.stride + w as usize * 4]);
    }
    Ok(Img { px: RgbaImage::from_raw(w, h, data).expect("plane size"), source: Format::Heic, alpha: handle.has_alpha_channel(), exif: None })
}

/// Read the format and upright dimensions without decoding pixels.
pub fn probe(path: &Path) -> Result<Probe> {
    let meta = std::fs::metadata(path).map_err(|e| Error::io(path, e))?;
    let mut head = [0u8; 64];
    let n = std::fs::File::open(path).and_then(|mut f| f.read(&mut head)).map_err(|e| Error::io(path, e))?;
    let format = identify(path, &head[..n])?;
    let dims = match format {
        Format::Avif => {
            let mut f = std::fs::File::open(path).map_err(|e| Error::io(path, e))?;
            avif_parse::read_avif(&mut f)
                .ok()
                .and_then(|d| d.primary_item_metadata().ok())
                .map(|m| Dims::new(m.max_frame_width.get(), m.max_frame_height.get()))
        }
        Format::Jxl => {
            let f = std::fs::File::open(path).map_err(|e| Error::io(path, e))?;
            jxl_oxide::JxlImage::builder().read(f).ok().map(|j| Dims::new(j.width(), j.height()))
        }
        Format::Svg => decode_svg(&std::fs::read(path).map_err(|e| Error::io(path, e))?).ok().map(|i| i.dims()),
        Format::Heic => None,
        other => {
            let mut reader = ImageReader::open(path).map_err(|e| Error::io(path, e))?;
            reader.set_format(image_format(other).expect("raster format"));
            reader.into_decoder().ok().map(|mut d| {
                let (w, h) = d.dimensions();
                let turned = matches!(
                    d.orientation().unwrap_or(image::metadata::Orientation::NoTransforms),
                    image::metadata::Orientation::Rotate90 | image::metadata::Orientation::Rotate270 | image::metadata::Orientation::Rotate90FlipH | image::metadata::Orientation::Rotate270FlipH
                );
                if turned { Dims::new(h, w) } else { Dims::new(w, h) }
            })
        }
    };
    Ok(Probe { format, dims, bytes: meta.len() })
}
