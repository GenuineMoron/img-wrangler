// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Sean Hawkey

//! Geometry for size, shape and fit. Pure maths: given the content's size and the settings,
//! where does everything go. Planning uses this without touching pixels.

use serde::{Deserialize, Serialize};

use crate::img::Dims;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub struct Rect {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}

/// The output's shape.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Shape {
    /// Keep each image's own shape.
    #[default]
    Original,
    /// A ratio such as 1:1 or 4:5; its long side is the size setting.
    Ratio(u32, u32),
    /// An exact canvas in pixels, such as 1200×628.
    Exact(u32, u32),
}

impl Shape {
    pub fn parse(text: &str) -> Option<Shape> {
        let t = text.trim().to_ascii_lowercase();
        if t.is_empty() || t == "original" || t == "orig" {
            return Some(Shape::Original);
        }
        let num = |s: &str| s.trim().parse::<u32>().ok().filter(|v| *v > 0);
        if let Some((a, b)) = t.split_once(':') {
            return Some(Shape::Ratio(num(a)?, num(b)?));
        }
        if let Some((a, b)) = t.split_once(['x', '×']) {
            return Some(Shape::Exact(num(a)?, num(b)?));
        }
        None
    }
}

impl std::fmt::Display for Shape {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Shape::Original => f.write_str("original"),
            Shape::Ratio(a, b) => write!(f, "{a}:{b}"),
            Shape::Exact(w, h) => write!(f, "{w}x{h}"),
        }
    }
}

impl Serialize for Shape {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for Shape {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let text = String::deserialize(d)?;
        Shape::parse(&text).ok_or_else(|| serde::de::Error::custom(format!("not a shape: {text} (use original, 1:1, 4:5 or 1200x628)")))
    }
}

/// How an image meets a shape that isn't its own.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[serde(rename_all = "snake_case")]
pub enum Fit {
    /// Fill the frame and cut off what spills over.
    Crop,
    /// Fit inside the frame and fill the rest with a backdrop.
    #[default]
    Pad,
    /// Fit inside the frame with no padding; the output keeps the image's shape.
    Inside,
}

/// Which side the size measures, when each image keeps its own shape.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[serde(rename_all = "snake_case")]
pub enum Edge {
    /// The longer side, whichever way round the image is.
    #[default]
    Long,
    Short,
    Width,
    Height,
}

impl Edge {
    /// The length of this side of an image.
    pub fn of(self, d: Dims) -> u32 {
        match self {
            Edge::Long => d.w.max(d.h),
            Edge::Short => d.w.min(d.h),
            Edge::Width => d.w,
            Edge::Height => d.h,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FrameSpec {
    /// The size in pixels: of `edge` when the shape is the original, of the long side otherwise.
    pub long_side: Option<u32>,
    pub edge: Edge,
    pub shape: Shape,
    pub fit: Fit,
    /// Where the image sits (pad) or what is kept (crop): 0 = start, 1 = end, per axis.
    pub anchor: (f32, f32),
    /// Share of the frame the content fills when padding, 0-1.
    pub fill: f32,
    pub upscale: bool,
}

impl Default for FrameSpec {
    fn default() -> Self {
        Self { long_side: None, edge: Edge::Long, shape: Shape::Original, fit: Fit::Pad, anchor: (0.5, 0.5), fill: 1.0, upscale: false }
    }
}

/// Where everything goes.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Layout {
    /// Final output size.
    pub canvas: Dims,
    /// Region of the content kept (crop fit), in content pixels.
    pub crop: Option<Rect>,
    /// Size the (cropped) content is scaled to.
    pub scaled: Dims,
    /// Top-left of the scaled content on the canvas.
    pub offset: (u32, u32),
}

fn round(v: f64) -> u32 {
    (v.round() as u32).max(1)
}

/// The frame for a ratio with the given long side.
fn ratio_frame(a: u32, b: u32, long: u32) -> Dims {
    if a >= b { Dims::new(long, round(f64::from(long) * f64::from(b) / f64::from(a))) } else { Dims::new(round(f64::from(long) * f64::from(a) / f64::from(b)), long) }
}

pub fn layout(content: Dims, spec: &FrameSpec) -> Layout {
    let (cw, ch) = (f64::from(content.w), f64::from(content.h));
    let fill = f64::from(spec.fill.clamp(0.05, 1.0));
    let (ax, ay) = (f64::from(spec.anchor.0.clamp(0.0, 1.0)), f64::from(spec.anchor.1.clamp(0.0, 1.0)));

    // Original shape: a plain resize, by whichever side the size measures.
    let frame = match spec.shape {
        Shape::Original => {
            let s = spec.long_side.map_or(1.0, |l| f64::from(l) / f64::from(spec.edge.of(content).max(1)));
            let s = if spec.upscale { s } else { s.min(1.0) };
            let d = Dims::new(round(cw * s), round(ch * s));
            return Layout { canvas: d, crop: None, scaled: d, offset: (0, 0) };
        }
        Shape::Exact(w, h) => Dims::new(w.max(1), h.max(1)),
        Shape::Ratio(a, b) => match spec.long_side {
            Some(l) => ratio_frame(a, b, l),
            // No size given: the frame follows the content.
            None => match spec.fit {
                Fit::Crop => {
                    let r = f64::from(a) / f64::from(b);
                    if cw / ch > r { Dims::new(round(ch * r), content.h) } else { Dims::new(content.w, round(cw / r)) }
                }
                _ => {
                    let r = f64::from(a) / f64::from(b);
                    let (bw, bh) = (cw / fill, ch / fill);
                    if bw / bh > r { Dims::new(round(bw), round(bw / r)) } else { Dims::new(round(bh * r), round(bh)) }
                }
            },
        },
    };
    let (fw, fh) = (f64::from(frame.w), f64::from(frame.h));

    match spec.fit {
        Fit::Crop => {
            let mut s = (fw / cw).max(fh / ch);
            let mut canvas = frame;
            if !spec.upscale && s > 1.0 {
                // Not enough pixels to fill the frame: keep the shape, shrink the frame.
                canvas = Dims::new(round(fw / s), round(fh / s));
                s = 1.0;
            }
            let (kw, kh) = (f64::from(canvas.w) / s, f64::from(canvas.h) / s);
            let crop = Rect { x: ((cw - kw) * ax).round().max(0.0) as u32, y: ((ch - kh) * ay).round().max(0.0) as u32, w: round(kw).min(content.w), h: round(kh).min(content.h) };
            Layout { canvas, crop: Some(crop), scaled: canvas, offset: (0, 0) }
        }
        Fit::Pad => {
            let mut s = (fw * fill / cw).min(fh * fill / ch);
            if !spec.upscale {
                s = s.min(1.0);
            }
            let scaled = Dims::new(round(cw * s).min(frame.w), round(ch * s).min(frame.h));
            let offset = (((fw - f64::from(scaled.w)) * ax).round() as u32, ((fh - f64::from(scaled.h)) * ay).round() as u32);
            Layout { canvas: frame, crop: None, scaled, offset }
        }
        Fit::Inside => {
            let mut s = (fw / cw).min(fh / ch);
            if !spec.upscale {
                s = s.min(1.0);
            }
            let d = Dims::new(round(cw * s), round(ch * s));
            Layout { canvas: d, crop: None, scaled: d, offset: (0, 0) }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PHOTO: Dims = Dims::new(6000, 4000);

    fn spec(long: Option<u32>, shape: Shape, fit: Fit) -> FrameSpec {
        FrameSpec { long_side: long, shape, fit, ..FrameSpec::default() }
    }

    #[test]
    fn original_shape_resizes_long_side() {
        let l = layout(PHOTO, &spec(Some(2048), Shape::Original, Fit::Pad));
        assert_eq!(l.canvas, Dims::new(2048, 1365));
        assert_eq!(l.scaled, l.canvas);
    }

    #[test]
    fn original_shape_resizes_by_the_side_asked_for() {
        let portrait = Dims::new(4000, 6000);
        let by = |edge, d| layout(d, &FrameSpec { edge, ..spec(Some(1080), Shape::Original, Fit::Pad) }).canvas;
        assert_eq!(by(Edge::Short, PHOTO), Dims::new(1620, 1080));
        assert_eq!(by(Edge::Short, portrait), Dims::new(1080, 1620));
        assert_eq!(by(Edge::Width, PHOTO), Dims::new(1080, 720));
        assert_eq!(by(Edge::Width, portrait), Dims::new(1080, 1620));
        assert_eq!(by(Edge::Height, PHOTO), Dims::new(1620, 1080));
        assert_eq!(by(Edge::Height, portrait), Dims::new(720, 1080));
    }

    #[test]
    fn never_upscales_unless_asked() {
        let small = Dims::new(800, 600);
        assert_eq!(layout(small, &spec(Some(2048), Shape::Original, Fit::Pad)).canvas, small);
        let up = FrameSpec { upscale: true, ..spec(Some(1600), Shape::Original, Fit::Pad) };
        assert_eq!(layout(small, &up).canvas, Dims::new(1600, 1200));
    }

    #[test]
    fn square_pad_centres_at_fill() {
        let s = FrameSpec { fill: 0.85, ..spec(Some(2048), Shape::Ratio(1, 1), Fit::Pad) };
        let l = layout(PHOTO, &s);
        assert_eq!(l.canvas, Dims::new(2048, 2048));
        assert_eq!(l.scaled.w, 1741); // 2048 × 0.85, the limiting side
        assert_eq!(l.offset.0, 154); // (2048 − 1741) ÷ 2, rounded
        assert_eq!(l.offset.1, 444); // (2048 − 1161) ÷ 2, rounded
    }

    #[test]
    fn square_crop_keeps_centre() {
        let l = layout(PHOTO, &spec(Some(1000), Shape::Ratio(1, 1), Fit::Crop));
        assert_eq!(l.canvas, Dims::new(1000, 1000));
        assert_eq!(l.crop, Some(Rect { x: 1000, y: 0, w: 4000, h: 4000 }));
    }

    #[test]
    fn crop_anchor_top() {
        let tall = Dims::new(1000, 3000);
        let s = FrameSpec { anchor: (0.5, 0.0), ..spec(Some(1000), Shape::Ratio(1, 1), Fit::Crop) };
        assert_eq!(layout(tall, &s).crop, Some(Rect { x: 0, y: 0, w: 1000, h: 1000 }));
    }

    #[test]
    fn crop_without_upscale_shrinks_frame() {
        let small = Dims::new(600, 400);
        let l = layout(small, &spec(Some(2048), Shape::Ratio(1, 1), Fit::Crop));
        assert_eq!(l.canvas, Dims::new(400, 400));
    }

    #[test]
    fn exact_canvas_for_social_cards() {
        let l = layout(PHOTO, &spec(None, Shape::Exact(1200, 628), Fit::Crop));
        assert_eq!(l.canvas, Dims::new(1200, 628));
    }

    #[test]
    fn inside_keeps_own_shape() {
        let l = layout(PHOTO, &spec(None, Shape::Exact(1000, 1000), Fit::Inside));
        assert_eq!(l.canvas, Dims::new(1000, 667));
    }

    #[test]
    fn shapes_parse_and_print() {
        for (text, shape) in [("original", Shape::Original), ("1:1", Shape::Ratio(1, 1)), ("4:5", Shape::Ratio(4, 5)), ("1200x628", Shape::Exact(1200, 628))] {
            assert_eq!(Shape::parse(text), Some(shape));
            assert_eq!(shape.to_string(), text);
        }
        assert_eq!(Shape::parse("0:1"), None);
        assert_eq!(Shape::parse("square"), None);
    }

    #[test]
    fn portrait_ratio_frames() {
        assert_eq!(ratio_frame(4, 5, 2000), Dims::new(1600, 2000));
        assert_eq!(ratio_frame(16, 9, 1920), Dims::new(1920, 1080));
    }
}
