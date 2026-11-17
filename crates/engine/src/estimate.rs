// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Sean Hawkey

//! Predicting output sizes: fully process a small, spread-out sample, then scale by each file's
//! planned pixel count. Shown in the app as "≈" until the run gives real numbers.

use std::collections::HashMap;
use std::sync::atomic::AtomicBool;

use rayon::prelude::*;
use serde::Serialize;

use crate::codec::Format;
use crate::modules::Cx;
use crate::plan::{Action, Plan};
use crate::process::process_path;
use crate::recipe::Recipe;

#[derive(Debug, Clone, Serialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub struct Estimate {
    pub sampled: u32,
    /// Estimated output bytes for each plan item, in plan order.
    pub per_item: Vec<Option<u64>>,
    pub bytes_out: u64,
}

pub fn estimate(plan: &Plan, recipe: &Recipe, max_samples: usize, cancel: &AtomicBool) -> Estimate {
    let candidates: Vec<usize> = plan.items.iter().enumerate().filter(|(_, i)| i.action == Action::Process && i.planned.is_some()).map(|(n, _)| n).collect();
    let step = (candidates.len() / max_samples.max(1)).max(1);
    let picks: Vec<usize> = candidates.iter().step_by(step).take(max_samples).copied().collect();

    // Bytes per output pixel, by output format, from the sample.
    let measured: Vec<(Format, f64, usize, u64)> = picks
        .par_iter()
        .filter_map(|&n| {
            let item = &plan.items[n];
            let out = process_path(&item.source, recipe, &mut Cx { cancel: Some(cancel) }).ok()?;
            Some((out.format, out.bytes.len() as f64 / out.dims.area().max(1) as f64, n, out.bytes.len() as u64))
        })
        .collect();
    let mut by_format: HashMap<Format, Vec<f64>> = HashMap::new();
    for (f, bpp, _, _) in &measured {
        by_format.entry(*f).or_default().push(*bpp);
    }
    let all: Vec<f64> = measured.iter().map(|m| m.1).collect();
    let mean = |v: &[f64]| (!v.is_empty()).then(|| v.iter().sum::<f64>() / v.len() as f64);
    let exact: HashMap<usize, u64> = measured.iter().map(|m| (m.2, m.3)).collect();

    let per_item: Vec<Option<u64>> = plan
        .items
        .iter()
        .enumerate()
        .map(|(n, item)| {
            let src = item.probe.as_ref().map(|p| p.bytes);
            match &item.action {
                Action::Skip { .. } => None,
                Action::Copy => src,
                Action::Process => {
                    if let Some(b) = exact.get(&n) {
                        return Some(*b);
                    }
                    let bpp = by_format.get(&item.format).and_then(|v| mean(v)).or_else(|| mean(&all))?;
                    let est = (bpp * item.planned?.dims.area() as f64) as u64;
                    // Never bigger: a same-format, same-size result is capped at the source.
                    let same = item.probe.as_ref().is_some_and(|p| p.format == item.format && p.dims == item.planned.map(|x| x.dims));
                    Some(if same && recipe.quality.never_bigger { est.min(src.unwrap_or(est)) } else { est })
                }
            }
        })
        .collect();
    Estimate { sampled: measured.len() as u32, bytes_out: per_item.iter().flatten().sum(), per_item }
}
