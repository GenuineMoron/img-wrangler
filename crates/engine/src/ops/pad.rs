// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Sean Hawkey

//! Placing an image on a backdrop.

use image::{imageops, RgbaImage};

use crate::colour::Rgba;
use crate::img::Dims;

pub fn composite(content: &RgbaImage, canvas: Dims, offset: (u32, u32), backdrop: Rgba) -> RgbaImage {
    composite_feathered(content, canvas, offset, backdrop, 0, None)
}

/// Like `composite`, but fades the content's edges into the backdrop over `feather` pixels on
/// every side that has padding, so a photo's own backdrop never shows as a hard-edged box.
/// With `only_like = Some((colour, tolerance))`, only pixels within `tolerance` of `colour`
/// (the photo's own backdrop) fade; the subject itself stays crisp.
pub fn composite_feathered(content: &RgbaImage, canvas: Dims, offset: (u32, u32), backdrop: Rgba, feather: u32, only_like: Option<(Rgba, i32)>) -> RgbaImage {
    let mut out = RgbaImage::from_pixel(canvas.w, canvas.h, image::Rgba(backdrop.0));
    if feather == 0 {
        imageops::overlay(&mut out, content, i64::from(offset.0), i64::from(offset.1));
        return out;
    }
    let (w, h) = content.dimensions();
    let pad = [offset.0 > 0, offset.1 > 0, offset.0 + w < canvas.w, offset.1 + h < canvas.h];
    let f = feather as f32;
    let mut faded = content.clone();
    for (x, y, p) in faded.enumerate_pixels_mut() {
        let mut k: f32 = 1.0;
        for (on, d) in pad.iter().zip([x, y, w - 1 - x, h - 1 - y]) {
            if *on {
                k = k.min((d as f32 + 0.5) / f);
            }
        }
        let backdrop_like = only_like.is_none_or(|(c, tol)| (0..3).all(|i| (i32::from(p.0[i]) - i32::from(c.0[i])).abs() <= tol));
        if k < 1.0 && backdrop_like {
            p.0[3] = (f32::from(p.0[3]) * k * k * (3.0 - 2.0 * k)).round() as u8;
        }
    }
    imageops::overlay(&mut out, &faded, i64::from(offset.0), i64::from(offset.1));
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn places_content_on_backdrop() {
        let content = RgbaImage::from_pixel(2, 2, image::Rgba([0, 0, 0, 255]));
        let out = composite(&content, Dims::new(4, 4), (1, 1), Rgba::WHITE);
        assert_eq!(out.get_pixel(0, 0).0, [255, 255, 255, 255]);
        assert_eq!(out.get_pixel(1, 1).0, [0, 0, 0, 255]);
    }

    #[test]
    fn feathered_edges_blend_into_backdrop() {
        let content = RgbaImage::from_pixel(40, 40, image::Rgba([0, 0, 0, 255]));
        let out = composite_feathered(&content, Dims::new(60, 60), (10, 10), Rgba::WHITE, 8, None);
        assert!(out.get_pixel(10, 30).0[0] > 200, "outer edge mostly backdrop");
        assert_eq!(out.get_pixel(30, 30).0, [0, 0, 0, 255], "centre untouched");
        let subject_kept = composite_feathered(&content, Dims::new(60, 60), (10, 10), Rgba::WHITE, 8, Some((Rgba::WHITE, 20)));
        assert_eq!(subject_kept.get_pixel(10, 30).0, [0, 0, 0, 255], "subject pixels never fade");
    }
}
