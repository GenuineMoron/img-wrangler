// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Sean Hawkey

//! Unsharp mask: add back a share of the difference between the image and a blurred copy.

use image::{imageops, RgbaImage};

/// `amount` 0–2 (1 = 100%), `radius` in pixels, `threshold` 0–255 ignores small differences.
pub fn unsharp(px: &RgbaImage, amount: f32, radius: f32, threshold: u8) -> RgbaImage {
    if amount <= 0.0 || radius <= 0.0 {
        return px.clone();
    }
    let blurred = imageops::blur(px, radius);
    let mut out = px.clone();
    for (o, b) in out.pixels_mut().zip(blurred.pixels()) {
        for c in 0..3 {
            let diff = i32::from(o.0[c]) - i32::from(b.0[c]);
            if diff.unsigned_abs() > u32::from(threshold) {
                o.0[c] = (f32::from(o.0[c]) + amount * diff as f32).round().clamp(0.0, 255.0) as u8;
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn increases_edge_contrast() {
        let px = RgbaImage::from_fn(20, 20, |x, _| if x < 10 { image::Rgba([100, 100, 100, 255]) } else { image::Rgba([160, 160, 160, 255]) });
        let out = unsharp(&px, 1.0, 1.0, 0);
        assert!(out.get_pixel(9, 10).0[0] < 100);
        assert!(out.get_pixel(10, 10).0[0] > 160);
        assert_eq!(out.get_pixel(0, 10).0[0], 100);
    }
}
