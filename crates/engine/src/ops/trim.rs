// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Sean Hawkey

//! Finding the empty backdrop around a subject.

use image::RgbaImage;

use super::frame::Rect;
use crate::colour::Rgba;

/// The backdrop found on the image's edges.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Edge {
    pub colour: Rgba,
    /// Share of edge pixels near `colour`, 0 to 1. Low means a busy photo, not a backdrop.
    pub uniformity: f32,
    /// How much the backdrop itself varies (90th percentile distance from `colour`), so
    /// studio sweeps with a gradient or vignette still count as backdrop.
    pub spread: i32,
}

/// Distance a backdrop may vary and still count as one backdrop (about 12%).
const BACKDROP_RANGE: i32 = 30;

/// Below this share of matching edge pixels, the edges are a scene, not a backdrop.
const MIN_UNIFORMITY: f32 = 0.6;

fn close(a: [u8; 4], b: [u8; 4], tol: i32) -> bool {
    if a[3] < 16 && b[3] < 16 {
        return true;
    }
    (0..4).all(|i| (i32::from(a[i]) - i32::from(b[i])).abs() <= tol)
}

/// Sample the border and take the per-channel median as the backdrop colour.
pub fn edge_colour(px: &RgbaImage, tolerance_pct: u8) -> Edge {
    let (w, h) = px.dimensions();
    let step = ((2 * (w + h)) / 2000).max(1);
    let mut samples: Vec<[u8; 4]> = Vec::new();
    for x in (0..w).step_by(step as usize) {
        samples.push(px.get_pixel(x, 0).0);
        samples.push(px.get_pixel(x, h - 1).0);
    }
    for y in (0..h).step_by(step as usize) {
        samples.push(px.get_pixel(0, y).0);
        samples.push(px.get_pixel(w - 1, y).0);
    }
    let median = |i: usize| {
        let mut v: Vec<u8> = samples.iter().map(|s| s[i]).collect();
        v.sort_unstable();
        v[v.len() / 2]
    };
    let colour = [median(0), median(1), median(2), median(3)];
    let range = tolerance(tolerance_pct).max(BACKDROP_RANGE);
    let matching = samples.iter().filter(|s| close(**s, colour, range)).count();
    let mut dist: Vec<i32> = samples.iter().map(|s| distance(*s, colour)).collect();
    dist.sort_unstable();
    let spread = dist[(dist.len() * 9 / 10).min(dist.len() - 1)];
    Edge { colour: Rgba(colour), uniformity: matching as f32 / samples.len().max(1) as f32, spread }
}

fn distance(a: [u8; 4], b: [u8; 4]) -> i32 {
    if a[3] < 16 && b[3] < 16 {
        return 0;
    }
    (0..4).map(|i| (i32::from(a[i]) - i32::from(b[i])).abs()).max().unwrap_or(0)
}

fn tolerance(pct: u8) -> i32 {
    (i32::from(pct.min(100)) * 255 / 100).max(2)
}

/// The smallest rectangle holding everything that isn't backdrop, or `None` when the edges
/// aren't a backdrop or nothing would be removed.
pub fn detect(px: &RgbaImage, tolerance_pct: u8) -> Option<(Rect, Edge)> {
    let (w, h) = px.dimensions();
    if w < 3 || h < 3 {
        return None;
    }
    let edge = edge_colour(px, tolerance_pct);
    if edge.uniformity < MIN_UNIFORMITY {
        return None;
    }
    // Content is whatever differs from the backdrop by more than the backdrop varies itself.
    let tol = tolerance(tolerance_pct).max(edge.spread + 6);
    // A row or column counts as content only when enough of it differs, so dust and
    // JPEG noise don't stop the trim.
    let min_row = (w / 400).max(2) as usize;
    let min_col = (h / 400).max(2) as usize;
    let differs = |x: u32, y: u32| !close(px.get_pixel(x, y).0, edge.colour.0, tol);
    let row_has = |y: u32| (0..w).filter(|&x| differs(x, y)).take(min_row).count() >= min_row;
    let col_has = |x: u32, top: u32, bottom: u32| (top..=bottom).filter(|&y| differs(x, y)).take(min_col).count() >= min_col;

    let top = (0..h).find(|&y| row_has(y))?;
    let bottom = (0..h).rev().find(|&y| row_has(y))?;
    let left = (0..w).find(|&x| col_has(x, top, bottom))?;
    let right = (0..w).rev().find(|&x| col_has(x, top, bottom))?;
    let rect = Rect { x: left, y: top, w: right - left + 1, h: bottom - top + 1 };
    if rect.w == w && rect.h == h {
        return None;
    }
    Some((rect, edge))
}

/// The median colour of the backdrop-like pixels on an image's border: after a trim, the
/// backdrop right next to the subject, which is what padding sits beside.
pub fn border_backdrop(px: &RgbaImage, like: Rgba, tol: i32) -> Option<Rgba> {
    let (w, h) = px.dimensions();
    let border = (0..w).flat_map(|x| [(x, 0), (x, h - 1)]).chain((0..h).flat_map(|y| [(0, y), (w - 1, y)]));
    let near: Vec<[u8; 4]> = border.map(|(x, y)| px.get_pixel(x, y).0).filter(|p| distance(*p, like.0) <= tol).collect();
    if near.len() < (2 * (w + h) / 5) as usize {
        return None;
    }
    let median = |i: usize| {
        let mut v: Vec<u8> = near.iter().map(|s| s[i]).collect();
        v.sort_unstable();
        v[v.len() / 2]
    };
    Some(Rgba([median(0), median(1), median(2), median(3)]))
}

/// Grow a rectangle by `margin` pixels on every side, kept inside the image.
pub fn with_margin(r: Rect, margin: u32, w: u32, h: u32) -> Rect {
    let x = r.x.saturating_sub(margin);
    let y = r.y.saturating_sub(margin);
    Rect { x, y, w: (r.x + r.w + margin).min(w) - x, h: (r.y + r.h + margin).min(h) - y }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn product_shot() -> RgbaImage {
        // A dark 40×60 product at (30, 20) on an off-white sweep with light noise.
        RgbaImage::from_fn(120, 100, |x, y| {
            if (30..70).contains(&x) && (20..80).contains(&y) {
                image::Rgba([40, 30, 20, 255])
            } else {
                let n = ((x * 7 + y * 13) % 5) as u8;
                image::Rgba([248 - n, 247 - n, 244 - n, 255])
            }
        })
    }

    #[test]
    fn finds_the_product() {
        let (r, edge) = detect(&product_shot(), 4).unwrap();
        assert_eq!(r, Rect { x: 30, y: 20, w: 40, h: 60 });
        assert!(edge.uniformity > 0.95);
        assert!(edge.colour.0[0] >= 244);
    }

    #[test]
    fn leaves_busy_photos_alone() {
        let busy = RgbaImage::from_fn(80, 80, |x, y| image::Rgba([(x * 3) as u8, (y * 3) as u8, ((x * y) % 255) as u8, 255]));
        assert!(detect(&busy, 4).is_none());
    }

    #[test]
    fn ignores_specks_of_dust() {
        let mut px = product_shot();
        px.put_pixel(5, 5, image::Rgba([0, 0, 0, 255]));
        let (r, _) = detect(&px, 4).unwrap();
        assert_eq!(r.x, 30);
    }

    #[test]
    fn trims_on_a_gradient_sweep() {
        // A studio sweep shading from 200 to 230 grey, with a dark product in the middle.
        let px = RgbaImage::from_fn(200, 160, |x, y| {
            if (80..120).contains(&x) && (50..120).contains(&y) {
                image::Rgba([20, 20, 20, 255])
            } else {
                let g = 200 + ((x + y) * 30 / 360) as u8;
                image::Rgba([g, g, g, 255])
            }
        });
        let (r, edge) = detect(&px, 4).unwrap();
        assert_eq!(r, Rect { x: 80, y: 50, w: 40, h: 70 });
        assert!(edge.spread > 10);
    }

    #[test]
    fn trims_transparent_padding() {
        let px = RgbaImage::from_fn(50, 50, |x, y| if (10..20).contains(&x) && (5..45).contains(&y) { image::Rgba([255, 0, 0, 255]) } else { image::Rgba([0, 0, 0, 0]) });
        let (r, _) = detect(&px, 4).unwrap();
        assert_eq!(r, Rect { x: 10, y: 5, w: 10, h: 40 });
    }

    #[test]
    fn margin_stays_inside() {
        assert_eq!(with_margin(Rect { x: 2, y: 2, w: 10, h: 10 }, 5, 14, 14), Rect { x: 0, y: 0, w: 14, h: 14 });
    }
}
