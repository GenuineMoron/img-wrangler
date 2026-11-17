// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Sean Hawkey

//! Quarter turns and mirror flips.

use image::{imageops, RgbaImage};

pub fn turn(px: &RgbaImage, degrees: u32) -> RgbaImage {
    match degrees % 360 {
        90 => imageops::rotate90(px),
        180 => imageops::rotate180(px),
        270 => imageops::rotate270(px),
        _ => px.clone(),
    }
}

pub fn flip(px: RgbaImage, horizontal: bool, vertical: bool) -> RgbaImage {
    let px = if horizontal { imageops::flip_horizontal(&px) } else { px };
    if vertical { imageops::flip_vertical(&px) } else { px }
}
