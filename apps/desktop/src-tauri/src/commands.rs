// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Sean Hawkey

//! Commands the front end calls. Each is a thin wrapper over the engine; anything slow runs on
//! a blocking thread so the window never stalls.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tauri::ipc::Channel;
use tauri::{AppHandle, Manager, State};
use wrangler_engine::codec::encode::{encoder_for, EncodeOptions, EncodeQuality};
use wrangler_engine::codec::{decode_path, Format, FormatInfo};
use wrangler_engine::estimate::Estimate;
use wrangler_engine::journal::{RunInfo, UndoReport};
use wrangler_engine::modules::Cx;
use wrangler_engine::plan::{Plan, PlanCx};
use wrangler_engine::presets::{self, RecipeEntry};
use wrangler_engine::recipe::Select;
use wrangler_engine::run::{Event, RunOptions, Summary};
use wrangler_engine::scan::SourceFile;
use wrangler_engine::{Dims, Recipe};
use wrangler_integrate as wi;

use crate::launch::Launch;
use crate::AppState;

type Res<T> = Result<T, String>;

fn err(e: impl std::fmt::Display) -> String {
    e.to_string()
}

async fn blocking<T: Send + 'static>(f: impl FnOnce() -> Res<T> + Send + 'static) -> Res<T> {
    tauri::async_runtime::spawn_blocking(f).await.map_err(err)?
}

// ---------- Formats and recipes ----------

#[tauri::command]
pub fn formats() -> Vec<FormatInfo> {
    wrangler_engine::codec::formats().to_vec()
}

#[tauri::command]
pub fn recipes(state: State<AppState>) -> Vec<RecipeEntry> {
    presets::list(&state.dirs.recipes())
}

/// A saved or built-in recipe; `"new"` gives the defaults.
#[tauri::command]
pub fn load_recipe(state: State<AppState>, id: String) -> Res<Recipe> {
    presets::load(&state.dirs.recipes(), &id).map_err(err)
}

#[tauri::command]
pub fn save_recipe(state: State<AppState>, recipe: Recipe) -> Res<String> {
    recipe.validate().map_err(err)?;
    presets::save(&state.dirs.recipes(), &recipe).map_err(err)
}

#[tauri::command]
pub fn delete_recipe(state: State<AppState>, id: String) -> Res<()> {
    presets::delete(&state.dirs.recipes(), &id).map_err(err)
}

// ---------- The batch ----------

/// Add dropped files and folders to the batch (or replace it), returning the whole batch.
#[tauri::command]
pub async fn scan(app: AppHandle, paths: Vec<PathBuf>, select: Select, append: bool) -> Res<Vec<SourceFile>> {
    let found = blocking(move || wrangler_engine::scan::collect(&paths, &select).map_err(err)).await?;
    let state = app.state::<AppState>();
    let mut sources = state.sources.lock();
    if !append {
        sources.clear();
    }
    for f in found {
        if !sources.iter().any(|s| s.path == f.path) {
            sources.push(f);
        }
    }
    Ok(sources.clone())
}

/// Drop files from the batch; an empty list clears it.
#[tauri::command]
pub fn forget(state: State<AppState>, paths: Vec<PathBuf>) -> Vec<SourceFile> {
    let mut sources = state.sources.lock();
    if paths.is_empty() {
        sources.clear();
    } else {
        sources.retain(|s| !paths.contains(&s.path));
    }
    sources.clone()
}

#[tauri::command]
pub async fn plan(app: AppHandle, recipe: Recipe) -> Res<Plan> {
    let sources = app.state::<AppState>().sources.lock().clone();
    blocking(move || Ok(wrangler_engine::plan::build(&sources, &recipe, &PlanCx::default()))).await
}

/// Sampled output size. A newer estimate cancels one still running.
#[tauri::command]
pub async fn estimate(app: AppHandle, recipe: Recipe, samples: Option<usize>) -> Res<Estimate> {
    let state = app.state::<AppState>();
    let cancel = Arc::new(AtomicBool::new(false));
    if let Some(old) = state.estimating.lock().replace(cancel.clone()) {
        old.store(true, Ordering::Relaxed);
    }
    let sources = state.sources.lock().clone();
    blocking(move || {
        let plan = wrangler_engine::plan::build(&sources, &recipe, &PlanCx::default());
        let out = wrangler_engine::estimate::estimate(&plan, &recipe, samples.unwrap_or(8), &cancel);
        if cancel.load(Ordering::Relaxed) { Err("superseded".into()) } else { Ok(out) }
    })
    .await
}

// ---------- Preview ----------

#[derive(Debug, Clone, Serialize)]
pub struct Preview {
    /// Token for `preview://localhost/<token>`.
    pub before: String,
    pub after: String,
    pub before_bytes: u64,
    pub after_bytes: u64,
    pub source_dims: Dims,
    pub dims: Dims,
    pub format: Format,
    pub quality: Option<u8>,
    pub score: Option<f64>,
    pub kept_original: bool,
    pub over_limit: bool,
    pub downscaled: bool,
}

/// Bytes a webview can show: the file itself when it can, otherwise a WebP copy.
fn displayable(bytes: Vec<u8>, format: Format, path: Option<&Path>) -> Res<(Vec<u8>, &'static str)> {
    if format.browser_displayable() {
        return Ok((bytes, format.mime()));
    }
    let img = match path {
        Some(p) => decode_path(p),
        None => wrangler_engine::codec::decode_bytes(&bytes, Some(format)),
    }
    .map_err(err)?;
    let webp = encoder_for(Format::Webp).map_err(err)?.encode(&img.px, img.has_transparency(), EncodeQuality::Lossy(92), &EncodeOptions { fast: true, ..EncodeOptions::default() }).map_err(err)?;
    Ok((webp, Format::Webp.mime()))
}

/// Process one image exactly as the run would, without writing anything.
#[tauri::command]
pub async fn preview(app: AppHandle, path: PathBuf, recipe: Recipe) -> Res<Preview> {
    let state_dirs_ok = app.state::<AppState>().sources.lock().iter().any(|s| s.path == path);
    if !state_dirs_ok {
        return Err("not in the batch".into());
    }
    let (p, before_store, after_store) = blocking(move || {
        let original = std::fs::read(&path).map_err(err)?;
        let source = Format::sniff(&original[..original.len().min(64)]).or_else(|| Format::from_path(&path)).ok_or("unknown format")?;
        let source_dims = wrangler_engine::codec::probe(&path).ok().and_then(|p| p.dims).unwrap_or(Dims::new(0, 0));
        let out = wrangler_engine::process::process_path(&path, &recipe, &mut Cx { cancel: None }).map_err(err)?;
        let before_bytes = original.len() as u64;
        let after_bytes = out.bytes.len() as u64;
        let before = displayable(original, source, Some(&path))?;
        let after = displayable(out.bytes, out.format, None)?;
        let token = || uuid::Uuid::new_v4().simple().to_string();
        let p = Preview {
            before: token(),
            after: token(),
            before_bytes,
            after_bytes,
            source_dims,
            dims: out.dims,
            format: out.format,
            quality: out.quality,
            score: out.score,
            kept_original: out.kept_original,
            over_limit: out.over_limit,
            downscaled: out.downscaled,
        };
        Ok((p, before, after))
    })
    .await?;
    let state = app.state::<AppState>();
    let mut previews = state.previews.lock();
    previews.push((p.before.clone(), before_store.0, before_store.1));
    previews.push((p.after.clone(), after_store.0, after_store.1));
    let excess = previews.len().saturating_sub(8);
    previews.drain(..excess);
    Ok(p)
}

// ---------- Running ----------

#[tauri::command]
pub async fn run(app: AppHandle, recipe: Recipe, on_event: Channel<Event>) -> Res<Summary> {
    let state = app.state::<AppState>();
    recipe.validate().map_err(err)?;
    state.cancel.store(false, Ordering::Relaxed);
    let cancel = state.cancel.clone();
    let sources = state.sources.lock().clone();
    let runs = state.dirs.runs();
    blocking(move || {
        let plan = wrangler_engine::plan::build(&sources, &recipe, &PlanCx::default());
        let opts = RunOptions { threads: 0, memory_budget: memory_budget(), runs_dir: runs };
        wrangler_engine::run::run(&plan, &recipe, &opts, &cancel, &|e| {
            let _ = on_event.send(e);
        })
        .map_err(err)
    })
    .await
}

/// Memory for images in flight: a few large photos decoded at once.
fn memory_budget() -> u64 {
    2 * 1024 * 1024 * 1024
}

#[derive(Debug, Clone, Serialize)]
pub struct QuickResult {
    pub recipe: Recipe,
    pub summary: Summary,
    /// Where the files went, for "Show in folder".
    pub first_output: Option<PathBuf>,
}

/// Run a saved recipe on paths from the file manager, apart from the main window's batch.
#[tauri::command]
pub async fn quick_run(app: AppHandle, recipe: String, paths: Vec<PathBuf>, on_event: Channel<Event>) -> Res<QuickResult> {
    let state = app.state::<AppState>();
    let dirs = state.dirs.clone();
    state.cancel.store(false, Ordering::Relaxed);
    let cancel = state.cancel.clone();
    blocking(move || {
        let recipe = presets::load(&dirs.recipes(), &recipe).map_err(err)?;
        let sources = wrangler_engine::scan::collect(&paths, &recipe.select).map_err(err)?;
        let plan = wrangler_engine::plan::build(&sources, &recipe, &PlanCx::default());
        let first_output = plan.items.first().map(|i| i.output.clone());
        let opts = RunOptions { threads: 0, memory_budget: memory_budget(), runs_dir: dirs.runs() };
        let summary = wrangler_engine::run::run(&plan, &recipe, &opts, &cancel, &|e| {
            let _ = on_event.send(e);
        })
        .map_err(err)?;
        Ok(QuickResult { recipe, summary, first_output })
    })
    .await
}

/// Load paths into the main window (from the quick window's Open key).
#[tauri::command]
pub fn open_main(app: AppHandle, paths: Vec<PathBuf>) {
    crate::deliver(&app, Launch::Open(paths));
}

#[tauri::command]
pub fn cancel(state: State<AppState>) {
    state.cancel.store(true, Ordering::Relaxed);
    if let Some(e) = state.estimating.lock().as_ref() {
        e.store(true, Ordering::Relaxed);
    }
}

#[tauri::command]
pub async fn undo(app: AppHandle, id: String) -> Res<UndoReport> {
    let runs = app.state::<AppState>().dirs.runs();
    blocking(move || wrangler_engine::journal::undo(&runs, &id).map_err(err)).await
}

#[tauri::command]
pub fn history(state: State<AppState>) -> Vec<RunInfo> {
    wrangler_engine::journal::history(&state.dirs.runs())
}

// ---------- Launch from the file manager ----------

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "mode", rename_all = "snake_case")]
pub enum LaunchPayload {
    Open { paths: Vec<PathBuf> },
    Quick { recipe: String, paths: Vec<PathBuf> },
}

/// Paths the app was opened with, once. Each window asks for its own kind ("open" for the main
/// window, "quick" for the quick window) when it starts and whenever `launch://pending` fires,
/// so the hidden main window can't take a quick action's files.
#[tauri::command]
pub fn take_launch(state: State<AppState>, kind: String) -> Option<LaunchPayload> {
    let mut pending = state.pending.lock();
    let mine = match pending.as_ref()? {
        Launch::Plain => {
            *pending = None;
            return None;
        }
        Launch::Open(_) => kind == "open",
        Launch::Quick { .. } => kind == "quick",
    };
    if !mine {
        return None;
    }
    match pending.take()? {
        Launch::Open(paths) => Some(LaunchPayload::Open { paths }),
        Launch::Quick { recipe, paths } => Some(LaunchPayload::Quick { recipe, paths }),
        Launch::Plain => None,
    }
}

// ---------- File-manager integration ----------

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Settings {
    /// Recipes offered directly in the right-click menu.
    pub pins: Vec<String>,
}

fn settings_path(state: &AppState) -> PathBuf {
    state.dirs.config.join("settings.json")
}

fn load_settings(state: &AppState) -> Settings {
    std::fs::read(settings_path(state)).ok().and_then(|b| serde_json::from_slice(&b).ok()).unwrap_or_else(|| Settings { pins: vec!["web-ready".into(), "just-compress".into(), "shopify-product".into()] })
}

#[derive(Debug, Clone, Serialize)]
pub struct Integration {
    pub target: wi::Target,
    pub label: &'static str,
    pub installed: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct Integrations {
    pub targets: Vec<Integration>,
    pub pins: Vec<String>,
}

#[tauri::command]
pub fn integrations(state: State<AppState>) -> Integrations {
    Integrations { targets: wi::targets().into_iter().map(|t| Integration { target: t, label: t.label(), installed: wi::installed(t) }).collect(), pins: load_settings(&state).pins }
}

/// Install (with these pinned recipes) or remove the right-click actions everywhere.
#[tauri::command]
pub fn set_integrations(state: State<AppState>, install: bool, pins: Vec<String>) -> Res<Integrations> {
    let settings = Settings { pins };
    std::fs::create_dir_all(&state.dirs.config).map_err(err)?;
    std::fs::write(settings_path(&state), serde_json::to_vec_pretty(&settings).map_err(err)?).map_err(err)?;
    let mut problems = Vec::new();
    if install {
        let program = std::env::current_exe().map_err(err)?;
        let actions = settings
            .pins
            .iter()
            .filter_map(|id| presets::load(&state.dirs.recipes(), id).ok().map(|r| wi::QuickAction { id: id.clone(), label: r.name }))
            .collect();
        let setup = wi::Setup { program, actions };
        for t in wi::targets() {
            if let Err(e) = wi::install(t, &setup) {
                problems.push(format!("{}: {e}", t.label()));
            }
        }
    } else {
        for t in wi::targets() {
            if let Err(e) = wi::remove(t) {
                problems.push(format!("{}: {e}", t.label()));
            }
        }
    }
    if problems.is_empty() { Ok(integrations(state)) } else { Err(problems.join("\n")) }
}
