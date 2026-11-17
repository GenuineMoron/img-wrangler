// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Sean Hawkey

//! The Wrangler image engine. See docs/ARCHITECTURE.md for the map.

pub mod codec;
pub mod colour;
pub mod error;
pub mod estimate;
pub mod img;
pub mod journal;
pub mod modules;
pub mod naming;
pub mod ops;
pub mod paths;
pub mod plan;
pub mod presets;
pub mod process;
pub mod quality;
pub mod recipe;
pub mod run;
pub mod scan;
pub mod thumbs;

pub use error::{Error, Result};
pub use img::{Dims, Img};
pub use recipe::Recipe;

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
