// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Sean Hawkey

//! Size: the core module for size, shape, fit, trimming and padding.
//! One place answers "how big, what shape, and how does the image sit in it".

use serde::{Deserialize, Serialize};

use super::{Cx, Planned, Stage};
use crate::colour::Rgba;
use crate::error::Result;
use crate::img::{Dims, Img};
use crate::ops::frame::{self, Edge, Fit, FrameSpec, Shape};
use crate::ops::resize::{resize, Filter};
use crate::ops::{pad, trim};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[serde(default)]
pub struct Size {
    /// The size in pixels; `None` keeps the original size. With the original shape it measures
    /// `edge`. With any other shape it's the frame's long side.
    pub long_side: Option<u32>,
    /// Which side `long_side` measures when each image keeps its own shape.
    pub edge: Edge,
    /// "original", a ratio such as "1:1" or "4:5", or an exact canvas such as "1200x628".
    #[cfg_attr(feature = "ts", ts(type = "string"))]
    pub shape: Shape,
    pub fit: Fit,
    pub anchor: Anchor,
    /// Allow making images bigger than they are.
    pub upscale: bool,
    /// Cut empty background before fitting. `None` keeps the edges.
    pub trim: Option<Trim>,
    pub pad: Pad,
    pub filter: Filter,
}

impl Default for Size {
    fn default() -> Self {
        Self { long_side: None, edge: Edge::Long, shape: Shape::Original, fit: Fit::Pad, anchor: Anchor::Center, upscale: false, trim: None, pad: Pad::default(), filter: Filter::Lanczos3 }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[serde(default)]
pub struct Trim {
    /// How different a pixel must be from the backdrop to count as subject, in percent.
    pub tolerance: u8,
    /// Pixels of backdrop to keep around the subject.
    pub margin: u32,
}

impl Default for Trim {
    fn default() -> Self {
        Self { tolerance: 4, margin: 0 }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[serde(default)]
pub struct Pad {
    pub backdrop: Backdrop,
    /// Percentage of the frame the image fills on its limiting side.
    pub fill: f32,
}

impl Default for Pad {
    fn default() -> Self {
        Self { backdrop: Backdrop::Auto, fill: 100.0 }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[serde(rename_all = "snake_case")]
pub enum Anchor {
    #[default]
    Center,
    Top,
    Bottom,
    Left,
    Right,
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

impl Anchor {
    pub fn fractions(self) -> (f32, f32) {
        match self {
            Anchor::Center => (0.5, 0.5),
            Anchor::Top => (0.5, 0.0),
            Anchor::Bottom => (0.5, 1.0),
            Anchor::Left => (0.0, 0.5),
            Anchor::Right => (1.0, 0.5),
            Anchor::TopLeft => (0.0, 0.0),
            Anchor::TopRight => (1.0, 0.0),
            Anchor::BottomLeft => (0.0, 1.0),
            Anchor::BottomRight => (1.0, 1.0),
        }
    }
}

/// What fills empty space when padding.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum Backdrop {
    /// The image's own edge colour when its edges are a plain backdrop; white otherwise.
    #[default]
    Auto,
    Transparent,
    Colour(Rgba),
}

impl Serialize for Backdrop {
    fn serialize<S: serde::Serializer>(&self, s: S) -> std::result::Result<S::Ok, S::Error> {
        match self {
            Backdrop::Auto => s.serialize_str("auto"),
            Backdrop::Transparent => s.serialize_str("transparent"),
            Backdrop::Colour(c) => s.serialize_str(&c.to_hex()),
        }
    }
}

impl<'de> Deserialize<'de> for Backdrop {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> std::result::Result<Self, D::Error> {
        let text = String::deserialize(d)?;
        match text.trim().to_ascii_lowercase().as_str() {
            "auto" => Ok(Backdrop::Auto),
            "transparent" | "clear" | "none" => Ok(Backdrop::Transparent),
            "white" => Ok(Backdrop::Colour(Rgba::WHITE)),
            "black" => Ok(Backdrop::Colour(Rgba::BLACK)),
            other => Rgba::from_hex(other).map(Backdrop::Colour).ok_or_else(|| serde::de::Error::custom(format!("not a colour: {other}"))),
        }
    }
}

#[cfg(feature = "ts")]
impl ts_rs::TS for Backdrop {
    type WithoutGenerics = Self;
    type OptionInnerType = Self;
    fn name(_: &ts_rs::Config) -> String {
        "string".into()
    }
    fn inline(_: &ts_rs::Config) -> String {
        "string".into()
    }
}

impl Size {
    fn spec(&self) -> FrameSpec {
        FrameSpec { long_side: self.long_side, edge: self.edge, shape: self.shape, fit: self.fit, anchor: self.anchor.fractions(), fill: self.pad.fill / 100.0, upscale: self.upscale }
    }

    pub fn validate(&self) -> std::result::Result<(), String> {
        if self.long_side == Some(0) {
            return Err("size must be above 0 px".into());
        }
        if !(5.0..=100.0).contains(&self.pad.fill) {
            return Err("the image must fill between 5% and 100% of the frame".into());
        }
        if let Some(t) = self.trim
            && t.tolerance > 50 {
                return Err("trim tolerance is a percentage up to 50".into());
            }
        match self.shape {
            Shape::Ratio(a, b) | Shape::Exact(a, b) if a == 0 || b == 0 => Err("shape sides must be above 0".into()),
            _ => Ok(()),
        }
    }
}

impl Stage for Size {
    fn id(&self) -> &'static str {
        "size"
    }
    fn order(&self) -> u16 {
        30
    }
    fn active(&self) -> bool {
        self.long_side.is_some() || self.shape != Shape::Original || self.trim.is_some()
    }
    fn plan(&self, input: Planned) -> Planned {
        Planned { dims: frame::layout(input.dims, &self.spec()).canvas, exact: input.exact && self.trim.is_none() }
    }
    fn apply(&self, mut img: Img, cx: &mut Cx) -> Result<Img> {
        let mut edge = None;
        if let Some(t) = self.trim
            && let Some((rect, mut e)) = trim::detect(&img.px, t.tolerance) {
                let r = trim::with_margin(rect, t.margin, img.px.width(), img.px.height());
                img.px = image::imageops::crop_imm(&img.px, r.x, r.y, r.w, r.h).to_image();
                // Sweeps are lighter near the subject than at the photo's edges: match padding
                // to the backdrop that is actually next to the subject.
                if let Some(c) = trim::border_backdrop(&img.px, e.colour, e.spread + 18) {
                    e.colour = c;
                }
                edge = Some(e);
            }
        cx.check()?;
        let l = frame::layout(img.dims(), &self.spec());
        if let Some(c) = l.crop
            && (c.w, c.h) != img.px.dimensions() {
                img.px = image::imageops::crop_imm(&img.px, c.x, c.y, c.w, c.h).to_image();
            }
        img.px = resize(&img.px, l.scaled.w, l.scaled.h, self.filter);
        if l.canvas != l.scaled {
            let colour = match self.pad.backdrop {
                Backdrop::Colour(c) => c,
                Backdrop::Transparent => Rgba::CLEAR,
                Backdrop::Auto => {
                    let e = *edge.get_or_insert_with(|| trim::edge_colour(&img.px, 4));
                    if e.uniformity >= 0.6 { e.colour } else { Rgba::WHITE }
                }
            };
            if colour.0[3] < 255 {
                img.alpha = true;
            }
            // Fade leftover photo backdrop into the pad colour; the subject stays crisp.
            // Images with real transparency already blend.
            let feather = if img.has_transparency() { 0 } else { (l.scaled.short() as f32 * 0.04).clamp(2.0, 64.0) as u32 };
            let like = edge.map(|e| (e.colour, e.spread + 18));
            img.px = pad::composite_feathered(&img.px, l.canvas, l.offset, colour, feather, like);
        }
        Ok(img)
    }
}

/// Predicted output for planning without a full pipeline.
pub fn predict(size: &Size, dims: Dims) -> Dims {
    frame::layout(dims, &size.spec()).canvas
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codec::Format;
    use image::RgbaImage;

    fn product() -> Img {
        Img::new(RgbaImage::from_fn(300, 200, |x, y| if (100..200).contains(&x) && (40..160).contains(&y) { image::Rgba([30, 30, 30, 255]) } else { image::Rgba([240, 230, 220, 255]) }), Format::Jpeg)
    }

    #[test]
    fn trim_then_square_pad_on_sampled_backdrop() {
        let size = Size { long_side: Some(500), shape: Shape::Ratio(1, 1), fit: Fit::Pad, trim: Some(Trim::default()), pad: Pad { backdrop: Backdrop::Auto, fill: 80.0 }, upscale: true, ..Size::default() };
        let out = size.apply(product(), &mut Cx::default()).unwrap();
        assert_eq!(out.dims(), Dims::new(500, 500));
        // The product (100×120) fills 80% of the height: 400 px tall.
        assert_eq!(out.px.get_pixel(250, 250).0, [30, 30, 30, 255]);
        assert_eq!(out.px.get_pixel(250, 55).0, [30, 30, 30, 255]); // product spans y 50-450
        assert_eq!(out.px.get_pixel(250, 20).0, [240, 230, 220, 255]);
        assert_eq!(out.px.get_pixel(5, 5).0, [240, 230, 220, 255]);
    }

    #[test]
    fn transparent_backdrop_marks_alpha() {
        let size = Size { long_side: Some(100), shape: Shape::Ratio(1, 1), pad: Pad { backdrop: Backdrop::Transparent, fill: 100.0 }, ..Size::default() };
        let out = size.apply(product(), &mut Cx::default()).unwrap();
        assert!(out.has_transparency());
    }

    #[test]
    fn plan_is_inexact_when_trimming() {
        let size = Size { long_side: Some(1000), trim: Some(Trim::default()), ..Size::default() };
        assert!(!size.plan(Planned { dims: Dims::new(2000, 1000), exact: true }).exact);
    }

    #[test]
    fn default_is_inactive() {
        assert!(!Size::default().active());
    }
}
