// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Sean Hawkey

//! Turning dropped files and folders into the list of images to process.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use globset::{Glob, GlobSet, GlobSetBuilder};
use serde::Serialize;

use crate::codec::is_supported_path;
use crate::error::{Error, Result};
use crate::recipe::Select;

/// Folders Wrangler writes into; never read back as input.
pub const OUTPUT_FOLDERS: &[&str] = &["Wrangled", ".wrangler-undo"];

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub struct SourceFile {
    pub path: PathBuf,
    /// The dropped folder this came from (or the file's own folder for dropped files).
    pub root: PathBuf,
    /// Path relative to `root`, used to mirror structure into an output folder.
    pub rel: PathBuf,
}

fn globs(patterns: &[String]) -> Result<Option<GlobSet>> {
    if patterns.is_empty() {
        return Ok(None);
    }
    let mut b = GlobSetBuilder::new();
    for p in patterns {
        b.add(Glob::new(&p.to_lowercase()).map_err(|e| Error::Recipe(format!("pattern {p}: {e}")))?);
    }
    b.build().map(Some).map_err(|e| Error::Recipe(e.to_string()))
}

fn hidden(name: &str) -> bool {
    name.starts_with('.')
}

pub fn collect(inputs: &[PathBuf], select: &Select) -> Result<Vec<SourceFile>> {
    let include = globs(&select.include)?;
    let exclude = globs(&select.exclude)?;
    let min_bytes = select.min_kb.unwrap_or(0) * 1024;
    let mut seen = HashSet::new();
    let mut out = Vec::new();

    let mut consider = |path: &Path, root: &Path| {
        if !is_supported_path(path) {
            return;
        }
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("").to_lowercase();
        if include.as_ref().is_some_and(|g| !g.is_match(&name)) || exclude.as_ref().is_some_and(|g| g.is_match(&name)) {
            return;
        }
        if min_bytes > 0 && std::fs::metadata(path).map(|m| m.len()).unwrap_or(0) < min_bytes {
            return;
        }
        let key = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
        if seen.insert(key) {
            out.push(SourceFile { path: path.to_path_buf(), root: root.to_path_buf(), rel: path.strip_prefix(root).unwrap_or(path).to_path_buf() });
        }
    };

    for input in inputs {
        let meta = std::fs::metadata(input).map_err(|e| Error::io(input, e))?;
        if meta.is_file() {
            let root = input.parent().unwrap_or(Path::new("."));
            consider(input, root);
            continue;
        }
        let depth = if select.recursive { usize::MAX } else { 1 };
        let walker = walkdir::WalkDir::new(input).max_depth(depth).follow_links(false).into_iter().filter_entry(|e| {
            let n = e.file_name().to_str().unwrap_or("");
            e.depth() == 0 || !(hidden(n) || (e.file_type().is_dir() && OUTPUT_FOLDERS.contains(&n)))
        });
        for entry in walker.flatten() {
            if entry.file_type().is_file() {
                consider(entry.path(), input);
            }
        }
    }
    out.sort_by(|a, b| natord::compare_ignore_case(&a.path.to_string_lossy(), &b.path.to_string_lossy()));
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn touch(p: &Path, bytes: usize) {
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, vec![0u8; bytes]).unwrap();
    }

    #[test]
    fn walks_filters_and_sorts_naturally() {
        let d = tempfile::tempdir().unwrap();
        for n in ["img10.jpg", "img2.jpg", "img1.png", "notes.txt", ".hidden.jpg", "sub/deep.webp", "Wrangled/out.jpg"] {
            touch(&d.path().join(n), 10);
        }
        let flat = collect(&[d.path().into()], &Select::default()).unwrap();
        let names: Vec<_> = flat.iter().map(|f| f.rel.to_string_lossy().into_owned()).collect();
        assert_eq!(names, ["img1.png", "img2.jpg", "img10.jpg"]);

        let deep = collect(&[d.path().into()], &Select { recursive: true, ..Select::default() }).unwrap();
        assert!(deep.iter().any(|f| f.rel == Path::new("sub/deep.webp")));
        assert!(!deep.iter().any(|f| f.rel.starts_with("Wrangled")));

        let only = collect(&[d.path().into()], &Select { include: vec!["*.JPG".into()], ..Select::default() }).unwrap();
        assert_eq!(only.len(), 2);
    }

    #[test]
    fn skips_small_files_and_duplicates() {
        let d = tempfile::tempdir().unwrap();
        touch(&d.path().join("tiny.jpg"), 10);
        touch(&d.path().join("big.jpg"), 4096);
        let f = d.path().join("big.jpg");
        let got = collect(&[d.path().into(), f], &Select { min_kb: Some(2), ..Select::default() }).unwrap();
        assert_eq!(got.len(), 1);
    }
}
