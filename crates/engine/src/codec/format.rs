// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Sean Hawkey

//! Image formats: detection from bytes and extensions, and the registry of what each can do.
//! The desktop app builds its Format list from `formats()`, so a new codec needs no UI change.

use std::path::Path;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[serde(rename_all = "snake_case")]
pub enum Format {
    Jpeg,
    Png,
    Webp,
    Avif,
    Gif,
    Tiff,
    Bmp,
    Ico,
    Qoi,
    Tga,
    Pnm,
    Jxl,
    Svg,
    Heic,
}

/// The extensions of every format Wrangler reads.
const EXTENSIONS: &[(&str, Format)] = &[
    ("jpg", Format::Jpeg), ("jpeg", Format::Jpeg), ("jpe", Format::Jpeg), ("jfif", Format::Jpeg),
    ("png", Format::Png), ("apng", Format::Png),
    ("webp", Format::Webp),
    ("avif", Format::Avif),
    ("gif", Format::Gif),
    ("tif", Format::Tiff), ("tiff", Format::Tiff),
    ("bmp", Format::Bmp), ("dib", Format::Bmp),
    ("ico", Format::Ico),
    ("qoi", Format::Qoi),
    ("tga", Format::Tga),
    ("pbm", Format::Pnm), ("pgm", Format::Pnm), ("ppm", Format::Pnm), ("pnm", Format::Pnm), ("pam", Format::Pnm),
    ("jxl", Format::Jxl),
    ("svg", Format::Svg),
    ("heic", Format::Heic), ("heif", Format::Heic), ("hif", Format::Heic),
];

impl Format {
    pub const ALL: [Format; 14] = [
        Format::Jpeg,
        Format::Png,
        Format::Webp,
        Format::Avif,
        Format::Gif,
        Format::Tiff,
        Format::Bmp,
        Format::Ico,
        Format::Qoi,
        Format::Tga,
        Format::Pnm,
        Format::Jxl,
        Format::Svg,
        Format::Heic,
    ];

    pub fn from_extension(ext: &str) -> Option<Format> {
        let ext = ext.to_ascii_lowercase();
        EXTENSIONS.iter().find(|(e, _)| *e == ext).map(|(_, f)| *f)
    }

    /// Every file extension Wrangler reads, lower case: for file pickers and file managers.
    pub fn extensions() -> Vec<&'static str> {
        EXTENSIONS.iter().map(|(e, _)| *e).collect()
    }

    pub fn from_path(path: &Path) -> Option<Format> {
        path.extension().and_then(|e| e.to_str()).and_then(Format::from_extension)
    }

    /// Identify a file by its first bytes. Extensions lie; magic numbers rarely do.
    pub fn sniff(head: &[u8]) -> Option<Format> {
        let starts = |sig: &[u8]| head.len() >= sig.len() && &head[..sig.len()] == sig;
        if starts(&[0xFF, 0xD8, 0xFF]) {
            return Some(Format::Jpeg);
        }
        if starts(b"\x89PNG\r\n\x1a\n") {
            return Some(Format::Png);
        }
        if starts(b"RIFF") && head.len() >= 12 && &head[8..12] == b"WEBP" {
            return Some(Format::Webp);
        }
        if starts(b"GIF87a") || starts(b"GIF89a") {
            return Some(Format::Gif);
        }
        if starts(b"II*\0") || starts(b"MM\0*") {
            return Some(Format::Tiff);
        }
        if starts(b"BM") {
            return Some(Format::Bmp);
        }
        if starts(&[0, 0, 1, 0]) {
            return Some(Format::Ico);
        }
        if starts(b"qoif") {
            return Some(Format::Qoi);
        }
        if starts(&[0xFF, 0x0A]) || starts(b"\0\0\0\x0cJXL \r\n\x87\n") {
            return Some(Format::Jxl);
        }
        if head.len() >= 12 && &head[4..8] == b"ftyp" {
            let brand = &head[8..12];
            return match brand {
                b"avif" | b"avis" => Some(Format::Avif),
                b"heic" | b"heix" | b"hevc" | b"hevx" | b"heim" | b"heis" | b"mif1" | b"msf1" => {
                    // mif1 is used by both; look for an avif brand among the compatible ones.
                    let tail = &head[12..head.len().min(64)];
                    if tail.windows(4).any(|w| w == b"avif") { Some(Format::Avif) } else { Some(Format::Heic) }
                }
                _ => None,
            };
        }
        if starts(b"P1") || starts(b"P2") || starts(b"P3") || starts(b"P4") || starts(b"P5") || starts(b"P6") || starts(b"P7") {
            return Some(Format::Pnm);
        }
        let text = std::str::from_utf8(&head[..head.len().min(512)]).unwrap_or("");
        if text.contains("<svg") {
            return Some(Format::Svg);
        }
        None
    }

    pub fn extension(self) -> &'static str {
        match self {
            Format::Jpeg => "jpg",
            Format::Png => "png",
            Format::Webp => "webp",
            Format::Avif => "avif",
            Format::Gif => "gif",
            Format::Tiff => "tif",
            Format::Bmp => "bmp",
            Format::Ico => "ico",
            Format::Qoi => "qoi",
            Format::Tga => "tga",
            Format::Pnm => "ppm",
            Format::Jxl => "jxl",
            Format::Svg => "svg",
            Format::Heic => "heic",
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Format::Jpeg => "JPEG",
            Format::Png => "PNG",
            Format::Webp => "WebP",
            Format::Avif => "AVIF",
            Format::Gif => "GIF",
            Format::Tiff => "TIFF",
            Format::Bmp => "BMP",
            Format::Ico => "ICO",
            Format::Qoi => "QOI",
            Format::Tga => "TGA",
            Format::Pnm => "PNM",
            Format::Jxl => "JPEG XL",
            Format::Svg => "SVG",
            Format::Heic => "HEIC",
        }
    }


    /// Format to write when the recipe says "same as the original" but the
    /// original can only be read (JPEG XL, SVG, HEIC).
    pub fn writable_equivalent(self) -> Format {
        match self {
            Format::Jxl | Format::Heic => Format::Jpeg,
            Format::Svg => Format::Png,
            other => other,
        }
    }


    /// Formats whose quality setting changes the result.
    pub fn can_encode(self) -> bool {
        self.info().encode
    }

    pub fn supports_alpha(self) -> bool {
        self.info().alpha
    }

    pub fn mime(self) -> &'static str {
        self.info().mime
    }

    pub fn is_lossy(self) -> bool {
        matches!(self, Format::Jpeg | Format::Webp | Format::Avif)
    }

    /// Whether a typical webview can show this format directly.
    pub fn browser_displayable(self) -> bool {
        matches!(self, Format::Jpeg | Format::Png | Format::Webp | Format::Gif | Format::Avif)
    }

}

/// What a format can do, for the UI and for planning.
#[derive(Debug, Clone, Serialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub struct FormatInfo {
    pub id: Format,
    pub label: &'static str,
    pub ext: &'static str,
    pub mime: &'static str,
    pub decode: bool,
    pub encode: bool,
    pub alpha: bool,
    pub animation: bool,
    pub lossy: bool,
    pub lossless: bool,
    /// Share of browsers that display it (percent); 0 means not a web format.
    pub browsers: u8,
    /// Typical file size relative to JPEG at the same look, used for estimates before sampling.
    pub size_vs_jpeg: f32,
    /// Shown on the main format keys rather than under More.
    pub primary: bool,
    pub hint: &'static str,
}

#[allow(clippy::too_many_arguments)] // a table row; named fields would triple the table
const fn info(id: Format, label: &'static str, ext: &'static str, mime: &'static str, encode: bool, alpha: bool, animation: bool, lossy: bool, lossless: bool, browsers: u8, size_vs_jpeg: f32, primary: bool, hint: &'static str) -> FormatInfo {
    FormatInfo { id, label, ext, mime, decode: true, encode, alpha, animation, lossy, lossless, browsers, size_vs_jpeg, primary, hint }
}

static FORMATS: [FormatInfo; 14] = [
    info(Format::Jpeg, "JPG", "jpg", "image/jpeg", true, false, false, true, false, 100, 1.0, true, "works everywhere"),
    info(Format::Png, "PNG", "png", "image/png", true, true, false, true, true, 100, 2.6, true, "graphics and transparency"),
    info(Format::Webp, "WebP", "webp", "image/webp", true, true, true, true, true, 97, 0.7, true, "smaller than JPG, nearly universal"),
    info(Format::Avif, "AVIF", "avif", "image/avif", true, true, true, true, true, 96, 0.52, true, "smallest, slower to save"),
    info(Format::Jxl, "JPEG XL", "jxl", "image/jxl", false, true, true, true, true, 69, 0.46, false, "read only for now"),
    info(Format::Gif, "GIF", "gif", "image/gif", true, true, true, true, false, 100, 1.7, false, "256 colours"),
    info(Format::Tiff, "TIFF", "tif", "image/tiff", true, true, false, false, true, 0, 3.6, false, "print and archive"),
    info(Format::Bmp, "BMP", "bmp", "image/bmp", true, false, false, false, true, 100, 6.0, false, "uncompressed"),
    info(Format::Ico, "ICO", "ico", "image/x-icon", true, true, false, false, true, 100, 1.0, false, "favicons, up to 256 px"),
    info(Format::Qoi, "QOI", "qoi", "image/qoi", true, true, false, false, true, 0, 3.0, false, "fast lossless"),
    info(Format::Tga, "TGA", "tga", "image/x-tga", true, true, false, false, true, 0, 4.0, false, "legacy"),
    info(Format::Pnm, "PNM", "ppm", "image/x-portable-anymap", true, false, false, false, true, 0, 6.0, false, "legacy"),
    info(Format::Svg, "SVG", "svg", "image/svg+xml", false, true, false, false, true, 100, 1.0, false, "read and rasterised"),
    info(Format::Heic, "HEIC", "heic", "image/heic", false, true, false, true, false, 0, 0.6, false, "read only"),
];

/// Every format Wrangler knows, in display order.
pub fn formats() -> &'static [FormatInfo] {
    &FORMATS
}

impl Format {
    pub fn info(self) -> &'static FormatInfo {
        FORMATS.iter().find(|f| f.id == self).expect("every format has registry info")
    }
}

/// Every extension Wrangler will pick up from a folder.
pub fn supported_extensions() -> &'static [&'static str] {
    &[
        "jpg", "jpeg", "jpe", "jfif", "png", "apng", "webp", "avif", "gif", "tif", "tiff", "bmp", "dib", "ico", "qoi",
        "tga", "pbm", "pgm", "ppm", "pnm", "pam", "jxl", "svg", "heic", "heif", "hif",
    ]
}

pub fn is_supported_path(path: &Path) -> bool {
    Format::from_path(path).is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sniffs_common_formats() {
        assert_eq!(Format::sniff(&[0xFF, 0xD8, 0xFF, 0xE0]), Some(Format::Jpeg));
        assert_eq!(Format::sniff(b"\x89PNG\r\n\x1a\n...."), Some(Format::Png));
        assert_eq!(Format::sniff(b"RIFF\0\0\0\0WEBPVP8 "), Some(Format::Webp));
        assert_eq!(Format::sniff(b"\0\0\0\x1cftypavif\0\0\0\0avifmif1"), Some(Format::Avif));
        assert_eq!(Format::sniff(b"\0\0\0\x18ftypheic\0\0\0\0mif1heic"), Some(Format::Heic));
        assert_eq!(Format::sniff(b"\0\0\0\x1cftypmif1\0\0\0\0mif1avif"), Some(Format::Avif));
        assert_eq!(Format::sniff(b"<?xml version=\"1.0\"?><svg xmlns"), Some(Format::Svg));
        assert_eq!(Format::sniff(b"hello"), None);
    }

    #[test]
    fn extensions_map_both_ways() {
        for f in Format::ALL {
            assert_eq!(Format::from_extension(f.extension()), Some(f), "{f:?}");
            assert_eq!(f.info().id, f);
        }
        assert_eq!(formats().iter().filter(|f| f.primary).count(), 4);
        assert_eq!(Format::from_extension("JPEG"), Some(Format::Jpeg));
        assert!(!is_supported_path(Path::new("notes.txt")));
    }
}
