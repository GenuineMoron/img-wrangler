// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Sean Hawkey

//! Pixel operations as pure functions. They know nothing about recipes, files or progress;
//! modules compose them.

pub mod frame;
pub mod pad;
pub mod resize;
pub mod rotate;
pub mod sharpen;
pub mod trim;
