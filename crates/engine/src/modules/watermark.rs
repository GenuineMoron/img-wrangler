// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Sean Hawkey

//! Watermark add-on: a logo in the same place on every image, after sizing and sharpening.
//!
//! Sizes are relative to each image's shorter side, so the mark looks the same on portraits,
//! landscapes and squares. The logo file is read once per run and kept in memory.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock};
use std::time::SystemTime;

use image::RgbaImage;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};

use super::size::Anchor;
use super::{Cx, Planned, Stage};
use crate::codec::decode_path;
use crate::error::{Error, Result};
use crate::img::Img;
use crate::ops::overlay::overlay;
use crate::ops::resize::{resize, Filter};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[serde(default)]
pub struct Watermark {
    pub enabled: bool,
    /// The logo: a PNG or SVG with transparency works best.
    #[cfg_attr(feature = "ts", ts(type = "string | null"))]
    pub image: Option<PathBuf>,
    pub anchor: Anchor,
    /// The logo's longer side, as a percentage of the image's shorter side.
    pub size: f32,
    /// Percent, 1 to 100.
    pub opacity: f32,
    /// Gap from the edges, as a percentage of the image's shorter side.
    pub margin: f32,
}

impl Default for Watermark {
    fn default() -> Self {
        Self { enabled: true, image: None, anchor: Anchor::BottomRight, size: 20.0, opacity: 70.0, margin: 3.0 }
    }
}

type Key = (PathBuf, Option<SystemTime>);

fn cache() -> &'static Mutex<HashMap<Key, Arc<RgbaImage>>> {
    static CACHE: OnceLock<Mutex<HashMap<Key, Arc<RgbaImage>>>> = OnceLock::new();
    CACHE.get_or_init(Default::default)
}

/// The logo's pixels, decoded once per file version.
fn logo(path: &Path) -> Result<Arc<RgbaImage>> {
    let modified = std::fs::metadata(path).map_err(|e| Error::io(path, e))?.modified().ok();
    let key = (path.to_path_buf(), modified);
    if let Some(px) = cache().lock().get(&key) {
        return Ok(px.clone());
    }
    let px = Arc::new(decode_path(path)?.px);
    cache().lock().insert(key, px.clone());
    Ok(px)
}

/// Where the mark goes: its size and its top-left corner on an image of `w` × `h`.
pub fn placement(w: u32, h: u32, mark: (u32, u32), wm: &Watermark) -> ((u32, u32), (i64, i64)) {
    let short = f64::from(w.min(h));
    let long = (short * f64::from(wm.size.clamp(1.0, 100.0)) / 100.0).max(1.0);
    let s = long / f64::from(mark.0.max(mark.1).max(1));
    let size = (((f64::from(mark.0) * s).round() as u32).max(1), ((f64::from(mark.1) * s).round() as u32).max(1));
    let m = short * f64::from(wm.margin.clamp(0.0, 50.0)) / 100.0;
    let (fx, fy) = wm.anchor.fractions();
    let free = |total: u32, used: u32| f64::from(total) - f64::from(used) - 2.0 * m;
    let x = (m + free(w, size.0) * f64::from(fx)).round() as i64;
    let y = (m + free(h, size.1) * f64::from(fy)).round() as i64;
    (size, (x, y))
}

impl Stage for Watermark {
    fn id(&self) -> &'static str {
        "watermark"
    }
    fn order(&self) -> u16 {
        60
    }
    fn active(&self) -> bool {
        self.enabled && self.image.is_some() && self.opacity > 0.0
    }
    fn plan(&self, input: Planned) -> Planned {
        input
    }
    fn apply(&self, mut img: Img, _cx: &mut Cx) -> Result<Img> {
        let Some(path) = &self.image else { return Ok(img) };
        let mark = logo(path)?;
        let (w, h) = img.px.dimensions();
        let ((mw, mh), (x, y)) = placement(w, h, mark.dimensions(), self);
        let scaled = if (mw, mh) == mark.dimensions() { (*mark).clone() } else { resize(&mark, mw, mh, Filter::Lanczos3) };
        overlay(&mut img.px, &scaled, x, y, self.opacity / 100.0);
        Ok(img)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codec::Format;
    use image::Rgba;

    #[test]
    fn placed_by_the_shorter_side_with_a_margin() {
        let wm = Watermark { size: 20.0, margin: 5.0, ..Watermark::default() };
        // A 2:1 logo on a 2000 × 1000 image: 200 px long, 50 px from the bottom-right edges.
        let ((w, h), (x, y)) = placement(2000, 1000, (400, 200), &wm);
        assert_eq!((w, h), (200, 100));
        assert_eq!((x, y), (2000 - 50 - 200, 1000 - 50 - 100));
        let centre = Watermark { anchor: Anchor::Center, ..wm };
        assert_eq!(placement(2000, 1000, (400, 200), &centre).1, (900, 450));
    }

    #[test]
    fn draws_the_logo_from_a_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("logo.png");
        RgbaImage::from_pixel(20, 10, Rgba([255, 0, 0, 255])).save(&path).unwrap();
        let wm = Watermark { image: Some(path), size: 50.0, opacity: 100.0, margin: 0.0, ..Watermark::default() };
        let img = Img::new(RgbaImage::from_pixel(100, 100, Rgba([0, 0, 255, 255])), Format::Png);
        let out = wm.apply(img, &mut Cx::default()).unwrap();
        assert_eq!(out.px.get_pixel(99, 99).0, [255, 0, 0, 255], "bottom-right corner is logo");
        assert_eq!(out.px.get_pixel(0, 0).0, [0, 0, 255, 255], "the rest is untouched");
        assert!(!Watermark::default().active(), "no logo, nothing to do");
    }
}
