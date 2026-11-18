// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Sean Hawkey

//! Run history and undo. Each run writes an append-only log as it goes, so even a crash leaves
//! a record of every original that was moved. Undo only reverses files that are unchanged.

use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

use parking_lot::Mutex;
use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Entry {
    /// A file this run wrote.
    Created { path: PathBuf, size: u64 },
    /// An original moved out of the way before being replaced.
    Stored { original: PathBuf, stored: PathBuf },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub struct RunInfo {
    pub id: String,
    pub recipe: String,
    pub started: String,
    pub finished: Option<String>,
    pub files: u32,
    pub bytes_in: u64,
    pub bytes_out: u64,
    pub undone: bool,
}

pub struct Journal {
    pub id: String,
    dir: PathBuf,
    log: Mutex<File>,
    stored: Mutex<u32>,
}

fn now() -> String {
    chrono::Local::now().to_rfc3339()
}

fn read_info(dir: &Path) -> Option<RunInfo> {
    serde_json::from_slice(&fs::read(dir.join("run.json")).ok()?).ok()
}

fn write_info(dir: &Path, info: &RunInfo) -> Result<()> {
    let tmp = dir.join("run.json.tmp");
    fs::write(&tmp, serde_json::to_vec_pretty(info).expect("serialisable")).map_err(|e| Error::io(&tmp, e))?;
    fs::rename(&tmp, dir.join("run.json")).map_err(|e| Error::io(dir, e))
}

impl Journal {
    pub fn begin(runs: &Path, recipe: &str) -> Result<Journal> {
        let id = format!("{}-{}", chrono::Local::now().format("%Y%m%d-%H%M%S"), &uuid::Uuid::new_v4().simple().to_string()[..6]);
        let dir = runs.join(&id);
        fs::create_dir_all(dir.join("originals")).map_err(|e| Error::io(&dir, e))?;
        write_info(&dir, &RunInfo { id: id.clone(), recipe: recipe.into(), started: now(), finished: None, files: 0, bytes_in: 0, bytes_out: 0, undone: false })?;
        let log = OpenOptions::new().create(true).append(true).open(dir.join("entries.jsonl")).map_err(|e| Error::io(&dir, e))?;
        Ok(Journal { id, dir, log: Mutex::new(log), stored: Mutex::new(0) })
    }

    fn append(&self, e: &Entry) -> Result<()> {
        let mut line = serde_json::to_vec(e).expect("serialisable");
        line.push(b'\n');
        let mut f = self.log.lock();
        f.write_all(&line).and_then(|_| f.flush()).map_err(|err| Error::io(&self.dir, err))
    }

    /// Move an original into the undo store. Uses the store when it's on the same drive, and a
    /// hidden `.wrangler-undo` folder beside the file otherwise, so this is always a fast rename.
    pub fn store(&self, original: &Path) -> Result<PathBuf> {
        let n = {
            let mut s = self.stored.lock();
            *s += 1;
            *s
        };
        let name = format!("{n:06}-{}", original.file_name().and_then(|s| s.to_str()).unwrap_or("file"));
        let primary = self.dir.join("originals").join(&name);
        let stored = match fs::rename(original, &primary) {
            Ok(()) => primary,
            Err(_) => {
                let beside = original.parent().unwrap_or(Path::new(".")).join(".wrangler-undo").join(&self.id);
                fs::create_dir_all(&beside).map_err(|e| Error::io(&beside, e))?;
                let target = beside.join(&name);
                fs::rename(original, &target).map_err(|e| Error::io(original, e))?;
                target
            }
        };
        self.append(&Entry::Stored { original: original.into(), stored: stored.clone() })?;
        Ok(stored)
    }

    pub fn created(&self, path: &Path) -> Result<()> {
        let size = fs::metadata(path).map(|m| m.len()).unwrap_or(0);
        self.append(&Entry::Created { path: path.into(), size })
    }

    pub fn finish(&self, files: u32, bytes_in: u64, bytes_out: u64) -> Result<()> {
        let mut info = read_info(&self.dir).ok_or_else(|| Error::Other("run record missing".into()))?;
        info.finished = Some(now());
        (info.files, info.bytes_in, info.bytes_out) = (files, bytes_in, bytes_out);
        write_info(&self.dir, &info)
    }
}

fn entries(dir: &Path) -> Vec<Entry> {
    let Ok(f) = File::open(dir.join("entries.jsonl")) else { return Vec::new() };
    BufReader::new(f).lines().map_while(|l| l.ok()).filter_map(|l| serde_json::from_str(&l).ok()).collect()
}

#[derive(Debug, Default, Serialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub struct UndoReport {
    pub restored: u32,
    pub removed: u32,
    /// Files left alone, with why.
    pub skipped: Vec<(PathBuf, String)>,
}

pub fn undo(runs: &Path, id: &str) -> Result<UndoReport> {
    let dir = runs.join(id);
    let mut info = read_info(&dir).ok_or_else(|| Error::Other(format!("no run called {id}")))?;
    if info.undone {
        return Err(Error::Other("this run was already undone".into()));
    }
    let mut report = UndoReport::default();
    for e in entries(&dir).into_iter().rev() {
        match e {
            Entry::Created { path, size } => match fs::metadata(&path) {
                Ok(m) if m.len() == size => match fs::remove_file(&path) {
                    Ok(()) => report.removed += 1,
                    Err(err) => report.skipped.push((path, err.to_string())),
                },
                Ok(_) => report.skipped.push((path, "changed since the run, left as is".into())),
                Err(_) => {}
            },
            Entry::Stored { original, stored } => {
                if original.exists() {
                    report.skipped.push((original, "a newer file is there now, so the original stays in the undo store".into()));
                } else if fs::rename(&stored, &original).is_ok() || (fs::copy(&stored, &original).is_ok() && fs::remove_file(&stored).is_ok()) {
                    report.restored += 1;
                } else {
                    report.skipped.push((original, "could not move the original back".into()));
                }
            }
        }
    }
    info.undone = true;
    write_info(&dir, &info)?;
    Ok(report)
}

/// Runs, newest first.
pub fn history(runs: &Path) -> Vec<RunInfo> {
    let mut out: Vec<RunInfo> = fs::read_dir(runs).map(|d| d.flatten().filter_map(|e| read_info(&e.path())).collect()).unwrap_or_default();
    out.sort_by(|a, b| b.started.cmp(&a.started));
    out
}

/// Delete runs (and their stored originals) older than `days`. Returns how many went.
pub fn prune(runs: &Path, days: u32) -> u32 {
    let cutoff = chrono::Local::now() - chrono::Duration::days(i64::from(days));
    let mut gone = 0;
    for info in history(runs) {
        let old = chrono::DateTime::parse_from_rfc3339(&info.started).is_ok_and(|t| t < cutoff);
        if !old {
            continue;
        }
        let dir = runs.join(&info.id);
        for e in entries(&dir) {
            if let Entry::Stored { stored, .. } = e {
                let _ = fs::remove_file(&stored);
                if let Some(parent) = stored.parent().filter(|p| p.ends_with(&info.id) && p.parent().is_some_and(|g| g.ends_with(".wrangler-undo"))) {
                    let _ = fs::remove_dir(parent);
                    let _ = fs::remove_dir(parent.parent().expect("checked"));
                }
            }
        }
        if fs::remove_dir_all(&dir).is_ok() {
            gone += 1;
        }
    }
    gone
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn undo_restores_originals_and_removes_outputs() {
        let d = tempfile::tempdir().unwrap();
        let runs = d.path().join("runs");
        let photo = d.path().join("photo.jpg");
        fs::write(&photo, b"original").unwrap();

        let j = Journal::begin(&runs, "test").unwrap();
        j.store(&photo).unwrap();
        fs::write(&photo, b"compressed").unwrap();
        j.created(&photo).unwrap();
        let extra = d.path().join("photo.webp");
        fs::write(&extra, b"new").unwrap();
        j.created(&extra).unwrap();
        j.finish(1, 8, 10).unwrap();

        let r = undo(&runs, &j.id).unwrap();
        assert_eq!((r.restored, r.removed), (1, 2));
        assert_eq!(fs::read(&photo).unwrap(), b"original");
        assert!(!extra.exists());
        assert!(undo(&runs, &j.id).is_err(), "can't undo twice");
        assert!(history(&runs)[0].undone);
    }

    #[test]
    fn undo_leaves_changed_files_alone() {
        let d = tempfile::tempdir().unwrap();
        let runs = d.path().join("runs");
        let out = d.path().join("out.jpg");
        let j = Journal::begin(&runs, "test").unwrap();
        fs::write(&out, b"from the run").unwrap();
        j.created(&out).unwrap();
        fs::write(&out, b"edited by someone afterwards").unwrap();
        let r = undo(&runs, &j.id).unwrap();
        assert_eq!(r.removed, 0);
        assert_eq!(r.skipped.len(), 1);
        assert!(out.exists());
    }

    #[test]
    fn prune_keeps_recent_runs() {
        let d = tempfile::tempdir().unwrap();
        Journal::begin(d.path(), "recent").unwrap();
        assert_eq!(prune(d.path(), 30), 0);
        assert_eq!(history(d.path()).len(), 1);
    }
}
