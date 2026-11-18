// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Sean Hawkey

//! Modules: one user-facing feature each. Pixel modules implement `Stage` and run in a fixed
//! order, so a recipe can never be assembled in a broken sequence. See docs/MODULES.md.

pub mod rotate;
pub mod sharpen;
pub mod size;
pub mod watermark;

use std::sync::atomic::{AtomicBool, Ordering};

use serde::{Deserialize, Serialize};

pub use rotate::Rotate;
pub use sharpen::Sharpen;
pub use size::{Anchor, Backdrop, Pad, Size, Trim};
pub use watermark::Watermark;

use crate::error::{Error, Result};
use crate::img::{Dims, Img};

/// A predicted output size. `exact` is false when content decides it (trim).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub struct Planned {
    pub dims: Dims,
    pub exact: bool,
}

/// Run-time context passed through the stages of one image.
#[derive(Default)]
pub struct Cx<'a> {
    pub cancel: Option<&'a AtomicBool>,
}

impl Cx<'_> {
    pub fn check(&self) -> Result<()> {
        match self.cancel {
            Some(c) if c.load(Ordering::Relaxed) => Err(Error::Cancelled),
            _ => Ok(()),
        }
    }
}

pub trait Stage: Send + Sync {
    fn id(&self) -> &'static str;
    /// Position in the pipeline (see the table in docs/ARCHITECTURE.md).
    fn order(&self) -> u16;
    /// Whether this stage changes anything with its current settings.
    fn active(&self) -> bool;
    /// Predict the output size without decoding pixels.
    fn plan(&self, input: Planned) -> Planned;
    fn apply(&self, img: Img, cx: &mut Cx) -> Result<Img>;
}

/// Optional modules. Add a feature by adding a variant and one arm in `stage`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum AddOn {
    Rotate(Rotate),
    Sharpen(Sharpen),
    Watermark(Watermark),
}

impl AddOn {
    pub fn stage(&self) -> &dyn Stage {
        match self {
            AddOn::Rotate(m) => m,
            AddOn::Sharpen(m) => m,
            AddOn::Watermark(m) => m,
        }
    }
}

/// The active stages of a recipe, in run order.
pub fn stages<'a>(size: &'a Size, addons: &'a [AddOn]) -> Vec<&'a dyn Stage> {
    let mut v: Vec<&dyn Stage> = std::iter::once(size as &dyn Stage).chain(addons.iter().map(AddOn::stage)).filter(|s| s.active()).collect();
    v.sort_by_key(|s| s.order());
    v
}

pub fn plan(stages: &[&dyn Stage], dims: Dims) -> Planned {
    stages.iter().fold(Planned { dims, exact: true }, |p, s| s.plan(p))
}

pub fn apply(stages: &[&dyn Stage], mut img: Img, cx: &mut Cx) -> Result<Img> {
    for s in stages {
        cx.check()?;
        img = s.apply(img, cx)?;
    }
    Ok(img)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stages_run_in_fixed_order_and_skip_inactive() {
        let size = Size { long_side: Some(1000), ..Size::default() };
        let addons = vec![AddOn::Sharpen(Sharpen::default()), AddOn::Rotate(Rotate { turn: 90, ..Rotate::default() }), AddOn::Rotate(Rotate { enabled: false, ..Rotate::default() })];
        let ids: Vec<_> = stages(&size, &addons).iter().map(|s| s.id()).collect();
        assert_eq!(ids, ["rotate", "size", "sharpen"]);
    }

    #[test]
    fn plan_follows_rotation_then_size() {
        let size = Size { long_side: Some(1000), ..Size::default() };
        let addons = vec![AddOn::Rotate(Rotate { turn: 90, ..Rotate::default() })];
        let p = plan(&stages(&size, &addons), Dims::new(4000, 3000));
        assert_eq!(p.dims, Dims::new(750, 1000));
    }
}
