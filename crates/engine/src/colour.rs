// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Sean Hawkey

//! Colours, and conversion of embedded colour profiles to sRGB.

use image::RgbaImage;
use moxcms::{ColorProfile, Layout, TransformOptions};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Rgba(pub [u8; 4]);

impl Rgba {
    pub const WHITE: Rgba = Rgba([255, 255, 255, 255]);
    pub const BLACK: Rgba = Rgba([0, 0, 0, 255]);
    pub const CLEAR: Rgba = Rgba([0, 0, 0, 0]);

    /// Accepts `#rgb`, `#rrggbb` and `#rrggbbaa`, with or without the `#`.
    pub fn from_hex(text: &str) -> Option<Rgba> {
        let hex = text.trim().trim_start_matches('#');
        let digits: Vec<u8> = hex.chars().map(|c| c.to_digit(16).map(|d| d as u8)).collect::<Option<_>>()?;
        match digits.len() {
            3 => Some(Rgba([digits[0] * 17, digits[1] * 17, digits[2] * 17, 255])),
            6 | 8 => {
                let byte = |i: usize| digits[i] * 16 + digits[i + 1];
                let alpha = if digits.len() == 8 { byte(6) } else { 255 };
                Some(Rgba([byte(0), byte(2), byte(4), alpha]))
            }
            _ => None,
        }
    }

    pub fn to_hex(self) -> String {
        let [r, g, b, a] = self.0;
        if a == 255 { format!("#{r:02x}{g:02x}{b:02x}") } else { format!("#{r:02x}{g:02x}{b:02x}{a:02x}") }
    }

    pub fn is_opaque(self) -> bool {
        self.0[3] == 255
    }
}

impl Default for Rgba {
    fn default() -> Self {
        Rgba::WHITE
    }
}

impl Serialize for Rgba {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&self.to_hex())
    }
}

impl<'de> Deserialize<'de> for Rgba {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let text = String::deserialize(d)?;
        match text.trim().to_ascii_lowercase().as_str() {
            "white" => Ok(Rgba::WHITE),
            "black" => Ok(Rgba::BLACK),
            other => Rgba::from_hex(other).ok_or_else(|| serde::de::Error::custom(format!("not a colour: {other}"))),
        }
    }
}

/// Convert pixels tagged with an ICC profile to sRGB in place.
/// Returns false (and leaves pixels alone) for sRGB or unreadable profiles.
pub fn convert_to_srgb(image: &mut RgbaImage, icc: &[u8]) -> bool {
    let Ok(source) = ColorProfile::new_from_slice(icc) else { return false };
    let srgb = ColorProfile::new_srgb();
    if looks_like_srgb(&source) {
        return false;
    }
    let Ok(transform) = source.create_transform_8bit(Layout::Rgba, &srgb, Layout::Rgba, TransformOptions::default())
    else {
        return false;
    };
    let src = image.as_raw().clone();
    transform.transform(&src, image.as_mut()).is_ok()
}

fn looks_like_srgb(profile: &ColorProfile) -> bool {
    let srgb = ColorProfile::new_srgb();
    let close = |a: moxcms::Xyzd, b: moxcms::Xyzd| {
        (a.x - b.x).abs() < 0.002 && (a.y - b.y).abs() < 0.002 && (a.z - b.z).abs() < 0.002
    };
    close(profile.red_colorant, srgb.red_colorant)
        && close(profile.green_colorant, srgb.green_colorant)
        && close(profile.blue_colorant, srgb.blue_colorant)
        && profile.red_trc == srgb.red_trc
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_hex() {
        assert_eq!(Rgba::from_hex("#fff"), Some(Rgba::WHITE));
        assert_eq!(Rgba::from_hex("FFFFFF"), Some(Rgba::WHITE));
        assert_eq!(Rgba::from_hex("#00000080"), Some(Rgba([0, 0, 0, 128])));
        assert_eq!(Rgba::from_hex("#12"), None);
        assert_eq!(Rgba::from_hex("#zzzzzz"), None);
        assert_eq!(Rgba([18, 52, 86, 255]).to_hex(), "#123456");
    }
}
