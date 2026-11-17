// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Sean Hawkey

//! Reading and writing image files.

pub mod decode;
pub mod encode;
mod format;
pub mod metadata;

pub use decode::{decode_bytes, decode_path, probe, Probe};
pub use format::{formats, is_supported_path, supported_extensions, Format, FormatInfo};
