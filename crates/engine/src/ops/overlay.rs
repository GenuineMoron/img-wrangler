// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Sean Hawkey

//! Laying one image over another: a logo on a photo.

use image::RgbaImage;

/// Draw `top` onto `base` with its top-left corner at (`x`, `y`), at `opacity` (0 to 1) on top of
/// its own transparency. Parts that fall outside `base` are left off.
pub fn overlay(base: &mut RgbaImage, top: &RgbaImage, x: i64, y: i64, opacity: f32) {
    let k = opacity.clamp(0.0, 1.0);
    let (bw, bh) = (i64::from(base.width()), i64::from(base.height()));
    for (tx, ty, p) in top.enumerate_pixels() {
        let (px, py) = (x + i64::from(tx), y + i64::from(ty));
        if px < 0 || py < 0 || px >= bw || py >= bh {
            continue;
        }
        let a = f32::from(p.0[3]) / 255.0 * k;
        if a <= 0.0 {
            continue;
        }
        let d = base.get_pixel_mut(px as u32, py as u32);
        let da = f32::from(d.0[3]) / 255.0;
        let out_a = a + da * (1.0 - a);
        for c in 0..3 {
            let v = (f32::from(p.0[c]) * a + f32::from(d.0[c]) * da * (1.0 - a)) / out_a.max(f32::EPSILON);
            d.0[c] = v.round().clamp(0.0, 255.0) as u8;
        }
        d.0[3] = (out_a * 255.0).round() as u8;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::Rgba;

    #[test]
    fn blends_at_opacity_and_clips_to_the_base() {
        let mut base = RgbaImage::from_pixel(10, 10, Rgba([0, 0, 0, 255]));
        let top = RgbaImage::from_pixel(4, 4, Rgba([255, 255, 255, 255]));
        overlay(&mut base, &top, 8, 8, 0.5);
        assert_eq!(base.get_pixel(9, 9).0, [128, 128, 128, 255]);
        assert_eq!(base.get_pixel(7, 7).0, [0, 0, 0, 255]);
    }

    #[test]
    fn transparent_pixels_leave_the_base_alone() {
        let mut base = RgbaImage::from_pixel(4, 4, Rgba([10, 20, 30, 255]));
        overlay(&mut base, &RgbaImage::from_pixel(4, 4, Rgba([255, 0, 0, 0])), 0, 0, 1.0);
        assert_eq!(base.get_pixel(1, 1).0, [10, 20, 30, 255]);
    }
}
