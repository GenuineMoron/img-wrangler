// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Sean Hawkey

//! The plan: for every source, where its output goes, what it is called, and how big it will
//! be, worked out without reading pixels. This is what the app shows before anything runs.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::codec::{probe, Format, Probe};
use crate::img::Dims;
use crate::modules::{self, Planned};
use crate::naming::{NameCx, Namer};
use crate::process::output_format;
use crate::recipe::{DestinationMode, OnConflict, Recipe};
use crate::scan::SourceFile;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Action {
    /// Decode, change and encode.
    Process,
    /// Copy or move the file unchanged (rename-only recipes).
    Copy,
    Skip { reason: String },
}

#[derive(Debug, Clone, Serialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub struct PlanItem {
    pub id: u32,
    pub source: PathBuf,
    pub probe: Option<Probe>,
    pub output: PathBuf,
    pub format: Format,
    pub planned: Option<Planned>,
    pub action: Action,
    /// The name was changed to avoid a clash (-2, -3…).
    pub renamed_for_clash: bool,
    /// Writing replaces the source file itself.
    pub in_place: bool,
}

#[derive(Debug, Clone, Serialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub struct Plan {
    pub items: Vec<PlanItem>,
    pub clashes: u32,
    pub bytes_in: u64,
    /// In `list` naming: files the list has no name for (they keep theirs).
    pub unlisted: u32,
}

/// Things a plan needs from the world, passed in so tests are repeatable.
pub struct PlanCx {
    /// Today's date for `{date}` when a file's own date is unknown.
    pub today: String,
    /// Whether a path already exists on disk.
    pub exists: Box<dyn Fn(&Path) -> bool + Send + Sync>,
}

impl Default for PlanCx {
    fn default() -> Self {
        Self { today: chrono::Local::now().format("%Y-%m-%d").to_string(), exists: Box::new(|p| p.exists()) }
    }
}

fn file_date(path: &Path, fallback: &str) -> String {
    std::fs::metadata(path)
        .and_then(|m| m.modified())
        .map(|t| chrono::DateTime::<chrono::Local>::from(t).format("%Y-%m-%d").to_string())
        .unwrap_or_else(|_| fallback.to_string())
}

fn taken_date(path: &Path) -> Option<String> {
    let file = std::fs::File::open(path).ok()?;
    let exif = exif::Reader::new().read_from_container(&mut std::io::BufReader::new(file)).ok()?;
    let f = exif.get_field(exif::Tag::DateTimeOriginal, exif::In::PRIMARY)?;
    let text = f.display_value().to_string();
    Some(text.get(..10)?.replace(':', "-"))
}

fn with_suffix(path: &Path, n: u32) -> PathBuf {
    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("image");
    let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
    path.with_file_name(if ext.is_empty() { format!("{stem}-{n}") } else { format!("{stem}-{n}.{ext}") })
}

pub fn build(sources: &[SourceFile], recipe: &Recipe, cx: &PlanCx) -> Plan {
    let stages = modules::stages(&recipe.size, &recipe.addons);
    let namer = Namer::new(&recipe.rename);
    let mut unlisted = 0;
    let wants_taken = recipe.rename.pattern.contains("{taken") || recipe.rename.list.contains("{taken");
    let sources_set: HashSet<&Path> = sources.iter().map(|s| s.path.as_path()).collect();
    let mut taken_paths: HashMap<PathBuf, u32> = HashMap::new();
    let mut items = Vec::with_capacity(sources.len());
    let mut clashes = 0;
    let mut bytes_in = 0;
    let mut index = 0;

    for (id, src) in sources.iter().enumerate() {
        let id = id as u32;
        let probe = probe(&src.path);
        let Ok(probe) = probe else {
            let reason = probe.err().map(|e| e.to_string()).unwrap_or_default();
            items.push(PlanItem { id, source: src.path.clone(), probe: None, output: src.path.clone(), format: Format::from_path(&src.path).unwrap_or(Format::Jpeg), planned: None, action: Action::Skip { reason }, renamed_for_clash: false, in_place: false });
            continue;
        };
        bytes_in += probe.bytes;
        let format = output_format(recipe, probe.format);
        let planned = probe.dims.map(|d| modules::plan(&stages, d));
        let date = file_date(&src.path, &cx.today);
        let taken = if wants_taken { taken_date(&src.path) } else { None };
        let stem = src.path.file_stem().and_then(|s| s.to_str()).unwrap_or("image");
        let folder = src.path.parent().and_then(|p| p.file_name()).and_then(|s| s.to_str()).unwrap_or("");
        if recipe.rename.mode == crate::recipe::RenameMode::List && !namer.listed(stem, index) {
            unlisted += 1;
        }
        let name = namer.render(&NameCx { stem, index, dims: planned.map_or(Dims::new(0, 0), |p| p.dims), folder, date: &date, taken: taken.as_deref(), ext: format.extension() });
        index += 1;
        let file_name = format!("{name}.{}", format.extension());
        let dir = src.path.parent().unwrap_or(Path::new("."));
        let d = &recipe.destination;
        let mut output = match d.mode {
            DestinationMode::Replace => dir.join(&file_name),
            DestinationMode::Beside => dir.join(&file_name),
            DestinationMode::Subfolder => dir.join(d.folder.as_deref().unwrap_or("Wrangled")).join(&file_name),
            DestinationMode::Folder => {
                let base = PathBuf::from(d.folder.as_deref().unwrap_or("Wrangled"));
                base.join(src.rel.parent().unwrap_or(Path::new(""))).join(&file_name)
            }
        };
        if d.mode == DestinationMode::Beside && output == src.path {
            output = output.with_file_name(format!("{name}{}.{}", d.suffix, format.extension()));
        }
        let in_place = d.mode == DestinationMode::Replace && output == src.path;

        // Clashes: with another output of this batch, or with a file already there that
        // isn't this item's own source.
        let mut renamed_for_clash = false;
        let mut skip = None;
        let clash = |p: &Path, taken: &HashMap<PathBuf, u32>| taken.contains_key(p) || (p != src.path && (cx.exists)(p) && !(d.mode == DestinationMode::Replace && sources_set.contains(p)));
        if clash(&output, &taken_paths) {
            clashes += 1;
            match d.on_conflict {
                OnConflict::Number => {
                    let mut n = 2;
                    while clash(&with_suffix(&output, n), &taken_paths) {
                        n += 1;
                    }
                    output = with_suffix(&output, n);
                    renamed_for_clash = true;
                }
                OnConflict::Skip => skip = Some(format!("{} already exists", output.file_name().and_then(|s| s.to_str()).unwrap_or(""))),
                OnConflict::Overwrite => {}
            }
        }
        taken_paths.insert(output.clone(), id);
        let action = match skip {
            Some(reason) => Action::Skip { reason },
            None if recipe.is_passthrough() => Action::Copy,
            None => Action::Process,
        };
        items.push(PlanItem { id, source: src.path.clone(), probe: Some(probe), output, format, planned, action, renamed_for_clash, in_place });
    }
    Plan { items, clashes, bytes_in, unlisted }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codec::encode::{encoder_for, EncodeOptions, EncodeQuality};
    use crate::modules::Size;
    use crate::recipe::{Destination, OutputFormat, Rename, RenameMode};
    use image::RgbaImage;

    fn write_jpeg(p: &Path, w: u32, h: u32) {
        let px = RgbaImage::from_pixel(w, h, image::Rgba([120, 120, 120, 255]));
        std::fs::write(p, encoder_for(Format::Jpeg).unwrap().encode(&px, false, EncodeQuality::Lossy(80), &EncodeOptions::default()).unwrap()).unwrap();
    }

    fn sources(dir: &Path, names: &[&str]) -> Vec<SourceFile> {
        names.iter().map(|n| {
            let p = dir.join(n);
            write_jpeg(&p, 600, 400);
            SourceFile { path: p, root: dir.into(), rel: PathBuf::from(n) }
        }).collect()
    }

    fn cx() -> PlanCx {
        PlanCx { today: "2026-11-17".into(), exists: Box::new(|p| p.exists()) }
    }

    #[test]
    fn plans_names_paths_and_sizes() {
        let d = tempfile::tempdir().unwrap();
        let src = sources(d.path(), &["DSC_1.jpg", "DSC_2.jpg"]);
        let mut r = Recipe { size: Size { long_side: Some(300), ..Size::default() }, ..Recipe::default() };
        r.format.id = OutputFormat::To(Format::Webp);
        r.rename = Rename { mode: RenameMode::Pattern, pattern: "shop-{n:00}".into(), ..Rename::default() };
        let plan = build(&src, &r, &cx());
        assert_eq!(plan.items[0].output, d.path().join("Wrangled/shop-01.webp"));
        assert_eq!(plan.items[1].output, d.path().join("Wrangled/shop-02.webp"));
        assert_eq!(plan.items[0].planned.unwrap().dims, Dims::new(300, 200));
        assert_eq!(plan.items[0].action, Action::Process);
    }

    #[test]
    fn clashes_get_numbered() {
        let d = tempfile::tempdir().unwrap();
        let src = sources(d.path(), &["a.jpg", "b.jpg"]);
        let r = Recipe { rename: Rename { mode: RenameMode::Pattern, pattern: "same".into(), ..Rename::default() }, ..Recipe::default() };
        let plan = build(&src, &r, &cx());
        assert_eq!(plan.items[1].output.file_name().unwrap(), "same-2.jpg");
        assert!(plan.items[1].renamed_for_clash);
        assert_eq!(plan.clashes, 1);
    }

    #[test]
    fn replace_in_place_and_existing_files() {
        let d = tempfile::tempdir().unwrap();
        let src = sources(d.path(), &["a.jpg"]);
        std::fs::write(d.path().join("a.webp"), b"x").unwrap();
        let mut r = Recipe { destination: Destination { mode: crate::recipe::DestinationMode::Replace, ..Destination::default() }, ..Recipe::default() };
        let same = build(&src, &r, &cx());
        assert!(same.items[0].in_place);
        r.format.id = OutputFormat::To(Format::Webp);
        let converted = build(&src, &r, &cx());
        assert_eq!(converted.items[0].output.file_name().unwrap(), "a-2.webp");
    }

    #[test]
    fn unreadable_files_are_skipped_with_a_reason() {
        let d = tempfile::tempdir().unwrap();
        let p = d.path().join("notes.jpg");
        std::fs::write(&p, b"not an image").unwrap();
        let plan = build(&[SourceFile { path: p.clone(), root: d.path().into(), rel: "notes.jpg".into() }], &Recipe::default(), &cx());
        assert!(matches!(plan.items[0].action, Action::Skip { .. }) || plan.items[0].planned.is_none());
    }
}
