// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Sean Hawkey

//! Running a plan: in parallel, within a memory budget, with atomic writes, progress events,
//! cancellation, and a journal for undo.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
use std::time::Instant;

use parking_lot::{Condvar, Mutex};
use rayon::prelude::*;
use serde::Serialize;

use crate::error::{Error, Result};
use crate::journal::Journal;
use crate::modules::Cx;
use crate::plan::{Action, Plan, PlanItem};
use crate::process::process_path;
use crate::recipe::{DestinationMode, OnConflict, Recipe};

#[derive(Debug, Clone)]
pub struct RunOptions {
    /// Worker threads; 0 uses every core.
    pub threads: usize,
    /// Rough cap on memory used by images in flight, in bytes.
    pub memory_budget: u64,
    pub runs_dir: PathBuf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Done,
    /// The original was kept because the new file wouldn't have been smaller.
    Kept,
    /// Saved, but still over the size limit.
    OverLimit,
    Skipped,
    Failed,
}

#[derive(Debug, Clone, Serialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub struct ItemReport {
    pub id: u32,
    pub status: Status,
    pub bytes_in: u64,
    pub bytes_out: u64,
    pub output: Option<PathBuf>,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub struct Summary {
    pub run_id: String,
    pub total: u32,
    pub done: u32,
    pub kept: u32,
    pub over_limit: u32,
    pub skipped: u32,
    pub failed: u32,
    pub bytes_in: u64,
    pub bytes_out: u64,
    pub seconds: f64,
    pub cancelled: bool,
}

#[derive(Debug, Clone, Serialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Event {
    Started { run_id: String, total: u32 },
    Item(ItemReport),
    Finished(Summary),
}

/// A counting semaphore weighted by bytes, so a few huge images can't exhaust memory while
/// many small ones still run in parallel.
struct Budget {
    free: Mutex<u64>,
    cv: Condvar,
    total: u64,
}

impl Budget {
    fn take(&self, want: u64) -> u64 {
        let want = want.min(self.total);
        let mut free = self.free.lock();
        while *free < want {
            self.cv.wait(&mut free);
        }
        *free -= want;
        want
    }
    fn give(&self, n: u64) {
        *self.free.lock() += n;
        self.cv.notify_all();
    }
}

fn cost(item: &PlanItem) -> u64 {
    let src = item.probe.as_ref().and_then(|p| p.dims).map_or(25_000_000, |d| d.area());
    let out = item.planned.map_or(src, |p| p.dims.area());
    // Decoded source, working copy and output, at 4 bytes a pixel.
    (src * 2 + out) * 4
}

fn temp_path(target: &Path) -> PathBuf {
    let name = target.file_name().and_then(|s| s.to_str()).unwrap_or("file");
    target.with_file_name(format!(".{name}.wrangler-{}", &uuid::Uuid::new_v4().simple().to_string()[..8]))
}

/// Write `bytes` to the item's output: temp file first, originals into the undo store, then an
/// atomic rename into place.
fn write(item: &PlanItem, bytes: &[u8], recipe: &Recipe, journal: &Journal) -> Result<PathBuf> {
    let target = &item.output;
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent).map_err(|e| Error::io(parent, e))?;
    }
    let source_meta = fs::metadata(&item.source).ok();
    let tmp = temp_path(target);
    fs::write(&tmp, bytes).map_err(|e| Error::io(&tmp, e))?;
    let finish = || -> Result<()> {
        if target.exists() {
            if *target == item.source || recipe.destination.on_conflict == OnConflict::Overwrite || recipe.destination.mode == DestinationMode::Replace {
                journal.store(target)?;
            } else {
                return Err(Error::Other(format!("{} appeared while running", target.display())));
            }
        }
        fs::rename(&tmp, target).map_err(|e| Error::io(target, e))?;
        // Replace mode with a new name or format: the original goes to the undo store too.
        if recipe.destination.mode == DestinationMode::Replace && *target != item.source && item.source.exists() {
            journal.store(&item.source)?;
        }
        journal.created(target)?;
        if recipe.destination.keep_dates
            && let Some(modified) = source_meta.as_ref().and_then(|m| m.modified().ok())
                && let Ok(f) = fs::File::options().write(true).open(target) {
                    let _ = f.set_times(fs::FileTimes::new().set_modified(modified));
                }
        Ok(())
    };
    finish().inspect_err(|_| {
        let _ = fs::remove_file(&tmp);
    })?;
    Ok(target.clone())
}

fn run_item(item: &PlanItem, recipe: &Recipe, journal: &Journal, cancel: &AtomicBool) -> ItemReport {
    let bytes_in = item.probe.as_ref().map_or(0, |p| p.bytes);
    let report = |status, bytes_out, output: Option<PathBuf>, message: Option<String>| ItemReport { id: item.id, status, bytes_in, bytes_out, output, message };
    match &item.action {
        Action::Skip { reason } => return report(Status::Skipped, 0, None, Some(reason.clone())),
        Action::Copy => {
            return match fs::read(&item.source).map_err(|e| Error::io(&item.source, e)).and_then(|b| {
                if item.output == item.source { Ok(item.source.clone()) } else { write(item, &b, recipe, journal) }
            }) {
                Ok(p) => report(Status::Done, bytes_in, Some(p), None),
                Err(e) => report(Status::Failed, 0, None, Some(e.to_string())),
            };
        }
        Action::Process => {}
    }
    let mut cx = Cx { cancel: Some(cancel) };
    let outcome = match process_path(&item.source, recipe, &mut cx) {
        Ok(o) => o,
        Err(Error::Cancelled) => return report(Status::Skipped, 0, None, Some("stopped".into())),
        Err(e) => return report(Status::Failed, 0, None, Some(e.to_string())),
    };
    // Kept in place: nothing to write at all.
    if outcome.kept_original && item.in_place {
        return report(Status::Kept, bytes_in, Some(item.source.clone()), Some("already smaller than the result".into()));
    }
    match write(item, &outcome.bytes, recipe, journal) {
        Ok(p) => {
            let status = if outcome.kept_original { Status::Kept } else if outcome.over_limit { Status::OverLimit } else { Status::Done };
            let msg = if outcome.over_limit { Some("couldn't reach the size limit".into()) } else if outcome.downscaled { Some("shrunk to fit the size limit".into()) } else { None };
            report(status, outcome.bytes.len() as u64, Some(p), msg)
        }
        Err(e) => report(Status::Failed, 0, None, Some(e.to_string())),
    }
}

pub fn run(plan: &Plan, recipe: &Recipe, opts: &RunOptions, cancel: &AtomicBool, on_event: &(dyn Fn(Event) + Sync)) -> Result<Summary> {
    let started = Instant::now();
    let journal = Journal::begin(&opts.runs_dir, &recipe.name)?;
    let total = plan.items.len() as u32;
    on_event(Event::Started { run_id: journal.id.clone(), total });

    let budget = Budget { free: Mutex::new(opts.memory_budget), cv: Condvar::new(), total: opts.memory_budget };
    let counts: [AtomicU32; 5] = Default::default();
    let (bytes_in, bytes_out) = (AtomicU64::new(0), AtomicU64::new(0));
    let pool = rayon::ThreadPoolBuilder::new().num_threads(opts.threads).thread_name(|i| format!("wrangler-{i}")).build().map_err(|e| Error::Other(e.to_string()))?;

    pool.install(|| {
        plan.items.par_iter().for_each(|item| {
            let r = if cancel.load(Ordering::Relaxed) {
                ItemReport { id: item.id, status: Status::Skipped, bytes_in: 0, bytes_out: 0, output: None, message: Some("stopped".into()) }
            } else {
                let held = budget.take(cost(item));
                let r = run_item(item, recipe, &journal, cancel);
                budget.give(held);
                r
            };
            counts[r.status as usize].fetch_add(1, Ordering::Relaxed);
            if matches!(r.status, Status::Done | Status::Kept | Status::OverLimit) {
                bytes_in.fetch_add(r.bytes_in, Ordering::Relaxed);
                bytes_out.fetch_add(r.bytes_out, Ordering::Relaxed);
            }
            on_event(Event::Item(r));
        });
    });

    let c = |s: Status| counts[s as usize].load(Ordering::Relaxed);
    let summary = Summary {
        run_id: journal.id.clone(),
        total,
        done: c(Status::Done),
        kept: c(Status::Kept),
        over_limit: c(Status::OverLimit),
        skipped: c(Status::Skipped),
        failed: c(Status::Failed),
        bytes_in: bytes_in.into_inner(),
        bytes_out: bytes_out.into_inner(),
        seconds: started.elapsed().as_secs_f64(),
        cancelled: cancel.load(Ordering::Relaxed),
    };
    journal.finish(summary.done + summary.kept + summary.over_limit, summary.bytes_in, summary.bytes_out)?;
    on_event(Event::Finished(summary.clone()));
    Ok(summary)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codec::encode::{encoder_for, EncodeOptions, EncodeQuality};
    use crate::codec::Format;
    use crate::journal::undo;
    use crate::modules::Size;
    use crate::plan::{build, PlanCx};
    use crate::recipe::{Destination, OutputFormat};
    use crate::scan::collect;
    use image::RgbaImage;

    fn folder() -> tempfile::TempDir {
        let d = tempfile::tempdir().unwrap();
        for (i, name) in ["a.jpg", "b.jpg", "c.jpg"].iter().enumerate() {
            let px = RgbaImage::from_fn(900, 600, |x, y| image::Rgba([((x + i as u32 * 40) % 256) as u8, (y % 256) as u8, 120, 255]));
            fs::write(d.path().join(name), encoder_for(Format::Jpeg).unwrap().encode(&px, false, EncodeQuality::Lossy(95), &EncodeOptions::default()).unwrap()).unwrap();
        }
        d
    }

    fn opts(d: &Path) -> RunOptions {
        RunOptions { threads: 2, memory_budget: 64 << 20, runs_dir: d.join("runs") }
    }

    #[test]
    fn converts_into_a_subfolder() {
        let d = folder();
        let mut r = Recipe { size: Size { long_side: Some(300), ..Size::default() }, ..Recipe::default() };
        r.format.id = OutputFormat::To(Format::Webp);
        let plan = build(&collect(&[d.path().into()], &r.select).unwrap(), &r, &PlanCx::default());
        let events = Mutex::new(Vec::new());
        let s = run(&plan, &r, &opts(d.path()), &AtomicBool::new(false), &|e| events.lock().push(e)).unwrap();
        assert_eq!(s.done, 3);
        assert!(d.path().join("Wrangled/a.webp").exists());
        assert!(d.path().join("a.jpg").exists(), "originals untouched");
        assert!(s.bytes_out < s.bytes_in);
        assert!(matches!(events.lock().last(), Some(Event::Finished(_))));
    }

    #[test]
    fn replace_in_place_then_undo() {
        let d = folder();
        let before = fs::read(d.path().join("a.jpg")).unwrap();
        let r = Recipe { size: Size { long_side: Some(300), ..Size::default() }, destination: Destination { mode: DestinationMode::Replace, ..Destination::default() }, ..Recipe::default() };
        let plan = build(&collect(&[d.path().into()], &r.select).unwrap(), &r, &PlanCx::default());
        let o = opts(d.path());
        let s = run(&plan, &r, &o, &AtomicBool::new(false), &|_| {}).unwrap();
        assert_eq!(s.done, 3);
        assert_ne!(fs::read(d.path().join("a.jpg")).unwrap(), before);
        undo(&o.runs_dir, &s.run_id).unwrap();
        assert_eq!(fs::read(d.path().join("a.jpg")).unwrap(), before);
        // No temp files left behind.
        assert!(fs::read_dir(d.path()).unwrap().flatten().all(|e| !e.file_name().to_string_lossy().contains(".wrangler-")));
    }

    #[test]
    fn convert_in_place_moves_original_to_undo() {
        let d = folder();
        let mut r = Recipe { destination: Destination { mode: DestinationMode::Replace, ..Destination::default() }, ..Recipe::default() };
        r.format.id = OutputFormat::To(Format::Png);
        let plan = build(&collect(&[d.path().into()], &r.select).unwrap(), &r, &PlanCx::default());
        let o = opts(d.path());
        let s = run(&plan, &r, &o, &AtomicBool::new(false), &|_| {}).unwrap();
        assert!(d.path().join("a.png").exists());
        assert!(!d.path().join("a.jpg").exists());
        undo(&o.runs_dir, &s.run_id).unwrap();
        assert!(d.path().join("a.jpg").exists());
        assert!(!d.path().join("a.png").exists());
    }

    #[test]
    fn cancel_stops_cleanly() {
        let d = folder();
        let r = Recipe { size: Size { long_side: Some(300), ..Size::default() }, ..Recipe::default() };
        let plan = build(&collect(&[d.path().into()], &r.select).unwrap(), &r, &PlanCx::default());
        let s = run(&plan, &r, &opts(d.path()), &AtomicBool::new(true), &|_| {}).unwrap();
        assert!(s.cancelled);
        assert_eq!(s.done, 0);
    }
}
