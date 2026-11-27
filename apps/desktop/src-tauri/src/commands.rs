// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Sean Hawkey

//! Commands the front end calls. Each is a thin wrapper over the engine; anything slow runs on
//! a blocking thread so the window never stalls.

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use serde::Serialize;
use tauri::ipc::Channel;
use tauri::{AppHandle, Manager, State};
use wrangler_engine::codec::{Format, FormatInfo};
use wrangler_engine::estimate::Estimate;
use wrangler_engine::journal::{RunInfo, UndoReport};
use wrangler_engine::plan::{Plan, PlanCx};
use wrangler_engine::presets::{self, RecipeEntry};
use wrangler_engine::recipe::Select;
use wrangler_engine::run::{Event, RunOptions, Summary};
use wrangler_engine::scan::SourceFile;
use wrangler_engine::Recipe;
use wrangler_integrate as wi;

use crate::launch::Launch;
use crate::{settings, AppState};

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

/// Every file extension Wrangler reads, for the file pickers.
#[tauri::command]
pub fn image_extensions() -> Vec<&'static str> {
    Format::extensions()
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

/// Every change to the recipes below also brings the menus that name them up to date.
fn shelf<T>(app: &AppHandle, change: impl FnOnce(&std::path::Path) -> wrangler_engine::Result<T>) -> Res<T> {
    let done = change(&app.state::<AppState>().dirs.recipes()).map_err(err)?;
    settings::menus_changed(app);
    Ok(done)
}

#[tauri::command]
pub fn save_recipe(app: AppHandle, recipe: Recipe) -> Res<String> {
    shelf(&app, |dir| presets::save(dir, &recipe))
}

/// Save under a new name, leaving the recipe it came from as it was.
#[tauri::command]
pub fn save_recipe_as(app: AppHandle, recipe: Recipe, from: String) -> Res<String> {
    shelf(&app, |dir| presets::save_as(dir, &recipe, &from))
}

#[tauri::command]
pub fn delete_recipe(app: AppHandle, id: String) -> Res<()> {
    shelf(&app, |dir| presets::delete(dir, &id))
}

#[tauri::command]
pub fn rename_recipe(app: AppHandle, id: String, name: String) -> Res<String> {
    shelf(&app, |dir| presets::rename(dir, &id, &name))
}

#[tauri::command]
pub fn duplicate_recipe(app: AppHandle, id: String) -> Res<String> {
    shelf(&app, |dir| presets::duplicate(dir, &id))
}

/// Put a changed starter recipe back as it came.
#[tauri::command]
pub fn reset_recipe(app: AppHandle, id: String) -> Res<()> {
    shelf(&app, |dir| presets::reset(dir, &id))
}

/// Bring back the starter recipes that were deleted.
#[tauri::command]
pub fn restore_recipes(app: AppHandle) -> Res<()> {
    shelf(&app, presets::restore)
}

#[tauri::command]
pub fn set_recipe_on(app: AppHandle, id: String, on: bool) -> Res<()> {
    shelf(&app, |dir| presets::set_on(dir, &id, on))
}

#[tauri::command]
pub fn set_recipe_order(app: AppHandle, ids: Vec<String>) -> Res<()> {
    shelf(&app, |dir| presets::set_order(dir, &ids))
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

/// The selected image as the recipe will make it: a quick draft, or the exact result.
#[tauri::command]
pub async fn preview(app: AppHandle, path: PathBuf, recipe: Recipe, exact: bool) -> Res<crate::pictures::Preview> {
    if !app.state::<AppState>().sources.lock().iter().any(|s| s.path == path) {
        return Err("not in the batch".into());
    }
    blocking(move || app.state::<AppState>().pictures.preview(&path, &recipe, exact)).await
}

/// Stop working on previews of anything but these files under this recipe.
#[tauri::command]
pub fn preview_focus(state: State<AppState>, paths: Vec<PathBuf>, recipe: Recipe) {
    state.pictures.focus(&paths, &recipe);
}

// ---------- Running ----------

/// Run the batch, or just some of it (`only`, from "Run just this one").
#[tauri::command]
pub async fn run(app: AppHandle, recipe: Recipe, only: Option<Vec<PathBuf>>, on_event: Channel<Event>) -> Res<Summary> {
    let state = app.state::<AppState>();
    recipe.validate().map_err(err)?;
    state.cancel.store(false, Ordering::Relaxed);
    let cancel = state.cancel.clone();
    let mut sources = state.sources.lock().clone();
    if let Some(only) = &only {
        sources.retain(|s| only.contains(&s.path));
    }
    let runs = state.dirs.runs();
    let out = blocking(move || {
        let plan = wrangler_engine::plan::build(&sources, &recipe, &PlanCx::default());
        let opts = RunOptions { threads: 0, memory_budget: memory_budget(), runs_dir: runs };
        wrangler_engine::run::run(&plan, &recipe, &opts, &cancel, &|e| {
            let _ = on_event.send(e);
        })
        .map_err(err)
    })
    .await;
    crate::tray::refresh(&app);
    out
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
    let out = blocking(move || {
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
    .await;
    crate::tray::refresh(&app);
    out
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
    let out = blocking(move || wrangler_engine::journal::undo(&runs, &id).map_err(err)).await;
    crate::tray::refresh(&app);
    out
}

/// Open an image in the app the system uses for it (an original, or a result).
#[tauri::command]
pub fn open_image(app: AppHandle, path: PathBuf) -> Res<()> {
    use tauri_plugin_opener::OpenerExt;
    if Format::from_path(&path).is_none() || !path.is_file() {
        return Err("not an image file".into());
    }
    app.opener().open_path(path.to_string_lossy(), None::<&str>).map_err(err)
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

#[derive(Debug, Clone, Serialize)]
pub struct Integration {
    pub target: wi::Target,
    pub label: &'static str,
    pub installed: bool,
    /// Where the menu shows, when that isn't simply the right-click menu.
    pub place: Option<&'static str>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Integrations {
    pub targets: Vec<Integration>,
}

#[tauri::command]
pub fn integrations() -> Integrations {
    Integrations { targets: wi::targets().into_iter().map(|t| Integration { target: t, label: t.label(), installed: wi::installed(t), place: wi::where_it_shows(t) }).collect() }
}

/// Put Wrangler in every file manager's right-click menu, or take it out of them all.
#[tauri::command]
pub fn set_menus(app: AppHandle, on: bool) -> Res<Integrations> {
    let mut problems = Vec::new();
    if on {
        let setup = settings::menu_setup(&app, true)?;
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
    if problems.is_empty() { Ok(integrations()) } else { Err(problems.join("\n")) }
}

// ---------- The app's own settings ----------

#[derive(Debug, Clone, Serialize)]
pub struct AppSettings {
    pub menu_bar: bool,
    /// This system can show a menu bar or tray icon.
    pub menu_bar_available: bool,
    pub auto_update: bool,
    /// Wrangler starts at login, in the menu bar.
    pub login: bool,
    /// The recipe images dropped on the menu bar icon run. None opens them in the window.
    pub drop_runs: Option<String>,
}

#[tauri::command]
pub fn app_settings(app: AppHandle) -> AppSettings {
    use tauri_plugin_autostart::ManagerExt;
    let s = settings::load(&app);
    AppSettings {
        menu_bar: s.menu_bar,
        menu_bar_available: crate::tray::available(),
        auto_update: s.auto_update,
        login: app.autolaunch().is_enabled().unwrap_or(false),
        drop_runs: s.drop_runs,
    }
}

#[tauri::command]
pub fn set_app_settings(app: AppHandle, menu_bar: bool, auto_update: bool, login: bool) -> Res<AppSettings> {
    use std::sync::atomic::Ordering;
    use tauri_plugin_autostart::ManagerExt;
    let mut s = settings::load(&app);
    s.menu_bar = menu_bar;
    s.auto_update = auto_update;
    settings::save(&app, &s)?;
    let state = app.state::<AppState>();
    let shown = state.in_menu_bar.load(Ordering::Relaxed);
    if menu_bar && !shown && crate::tray::available() {
        crate::tray::build(&app).map_err(err)?;
        state.in_menu_bar.store(true, Ordering::Relaxed);
    } else if !menu_bar && shown {
        app.remove_tray_by_id(crate::tray::ID);
        state.in_menu_bar.store(false, Ordering::Relaxed);
    }
    let launcher = app.autolaunch();
    if login != launcher.is_enabled().unwrap_or(false) {
        if login { launcher.enable() } else { launcher.disable() }.map_err(err)?;
    }
    Ok(app_settings(app))
}

#[tauri::command]
pub fn set_drop_recipe(app: AppHandle, id: Option<String>) -> Res<()> {
    let mut s = settings::load(&app);
    s.drop_runs = id;
    settings::save(&app, &s)
}

/// A window has drawn its first frame: put it on screen, unless it's the main window and no one
/// asked for it (a quick action, or a start at login).
#[tauri::command]
pub fn window_ready(window: tauri::WebviewWindow, state: State<AppState>) {
    use std::sync::atomic::Ordering;
    if window.label() != "main" || state.show_main.load(Ordering::Relaxed) {
        let _ = window.show();
        let _ = window.set_focus();
    }
}

// ---------- Self-test ----------

/// `img-wrangler-app selftest <images…> <report.json> [--no-measure] [--no-run] [--no-layout]`: the interface
/// drives itself through the everyday steps, measures them, writes the report and quits. See
/// apps/desktop/src/lib/selftest.
#[derive(Debug, Clone, Serialize)]
pub struct SelfTestRun {
    pub images: Vec<PathBuf>,
    #[serde(skip)]
    pub report: PathBuf,
    pub measure: bool,
    pub run: bool,
    pub layout: bool,
    /// What the webview runs on, for reading the numbers.
    pub host: Vec<(String, String)>,
    /// Effects to switch off (`--fx tex,glow`), to measure what each costs.
    pub fx: Vec<String>,
    /// Full or light effects for this test (`--effects light`), instead of the automatic choice.
    pub effects: Option<String>,
    /// `--update check` looks for a newer version. `--update install` also installs it, which
    /// restarts the app into the new version (and that runs the test again).
    pub update: Option<String>,
}

fn host_info() -> Vec<(String, String)> {
    let env = |k: &str| std::env::var(k).unwrap_or_default();
    let mut h = vec![("os".into(), std::env::consts::OS.into()), ("arch".into(), std::env::consts::ARCH.into()), ("cores".into(), std::thread::available_parallelism().map_or(0, |n| n.get()).to_string())];
    if cfg!(target_os = "linux") {
        h.push(("session".into(), env("XDG_SESSION_TYPE")));
        h.push(("desktop".into(), env("XDG_CURRENT_DESKTOP")));
        h.push(("nvidia".into(), std::path::Path::new("/proc/driver/nvidia/version").exists().to_string()));
        h.push(("WEBKIT_DISABLE_DMABUF_RENDERER".into(), env("WEBKIT_DISABLE_DMABUF_RENDERER")));
        h.push(("WEBKIT_DISABLE_COMPOSITING_MODE".into(), env("WEBKIT_DISABLE_COMPOSITING_MODE")));
    }
    h
}

impl SelfTestRun {
    pub fn from_args(args: &[String]) -> Option<Self> {
        let rest: Vec<&String> = args.iter().skip(1).collect();
        if rest.first().map(|a| a.as_str()) != Some("selftest") {
            return None;
        }
        let (mut paths, mut fx, mut effects, mut update) = (Vec::new(), Vec::new(), None, None);
        let mut it = rest.iter().skip(1);
        while let Some(a) = it.next() {
            if *a == "--fx" {
                fx = it.next().map(|v| v.split(',').map(String::from).collect()).unwrap_or_default();
            } else if *a == "--effects" {
                effects = it.next().map(|v| v.to_string());
            } else if *a == "--update" {
                update = it.next().map(|v| v.to_string());
            } else if !a.starts_with("--") {
                paths.push(PathBuf::from(a));
            }
        }
        let (report, images) = paths.split_last()?;
        Some(Self { images: images.to_vec(), report: report.clone(), measure: !args.iter().any(|a| a == "--no-measure"), run: !args.iter().any(|a| a == "--no-run"), layout: !args.iter().any(|a| a == "--no-layout"), host: host_info(), fx, effects, update })
    }
}

#[tauri::command]
pub fn selftest_config(state: State<AppState>) -> Option<SelfTestRun> {
    state.selftest.clone()
}

#[tauri::command]
pub fn selftest_done(app: AppHandle, report: String) -> Res<()> {
    let state = app.state::<AppState>();
    let Some(t) = &state.selftest else { return Err("not running a self-test".into()) };
    std::fs::write(&t.report, report).map_err(err)?;
    app.exit(0);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(a: &[&str]) -> Vec<String> {
        std::iter::once("img-wrangler-app").chain(a.iter().copied()).map(String::from).collect()
    }

    #[test]
    fn selftest_arguments() {
        assert!(SelfTestRun::from_args(&args(&["open", "/a.jpg"])).is_none());
        let t = SelfTestRun::from_args(&args(&["selftest", "/photos", "/more", "/out/report.json", "--no-run", "--fx", "tex,glow", "--effects", "light", "--update", "check"])).unwrap();
        assert_eq!(t.images, vec![PathBuf::from("/photos"), PathBuf::from("/more")]);
        assert_eq!(t.report, PathBuf::from("/out/report.json"));
        assert!(t.measure && !t.run && t.layout);
        assert_eq!(t.fx, vec!["tex", "glow"]);
        assert_eq!(t.effects.as_deref(), Some("light"));
        assert_eq!(t.update.as_deref(), Some("check"));
        assert!(SelfTestRun::from_args(&args(&["selftest"])).is_none(), "needs at least a report path");
    }
}
