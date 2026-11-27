// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Sean Hawkey

//! The app's own settings, in settings.json beside the recipes. Recipes hold everything about
//! the images; this holds how the app itself behaves.

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};
use wrangler_engine::presets;

use crate::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    /// Keep the hat in the menu bar or tray, with Wrangler running when its window closes.
    pub menu_bar: bool,
    /// Look for a new version now and then.
    pub auto_update: bool,
    /// A recipe to run on images dropped on the menu bar icon. Without one they open in the
    /// main window.
    pub drop_runs: Option<String>,
    /// Wrangler has put itself in the file managers' right-click menus once. After that, the
    /// menus are only kept current where they still are, so removing them sticks.
    pub menus_set_up: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self { menu_bar: true, auto_update: true, drop_runs: None, menus_set_up: false }
    }
}

fn path(app: &AppHandle) -> std::path::PathBuf {
    app.state::<AppState>().dirs.config.join("settings.json")
}

pub fn load(app: &AppHandle) -> Settings {
    std::fs::read(path(app)).ok().and_then(|b| serde_json::from_slice(&b).ok()).unwrap_or_default()
}

pub fn save(app: &AppHandle, s: &Settings) -> Result<(), String> {
    let p = path(app);
    if let Some(dir) = p.parent() {
        std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    }
    std::fs::write(&p, serde_json::to_vec_pretty(s).map_err(|e| e.to_string())?).map_err(|e| e.to_string())
}

/// The program a right-click menu should start. An AppImage runs from a mount that is gone once
/// it quits, so its menus start the AppImage file itself.
fn program() -> Result<std::path::PathBuf, String> {
    let image = std::env::var_os("APPIMAGE").map(std::path::PathBuf::from).filter(|p| cfg!(target_os = "linux") && p.is_file());
    match image {
        Some(image) => Ok(image),
        None => std::env::current_exe().map_err(|e| e.to_string()),
    }
}

/// What the right-click menus are built from: this copy of the app and the recipes that are on.
/// `asked` says the person asked for the menus just now (see `wrangler_integrate::Setup`).
pub fn menu_setup(app: &AppHandle, asked: bool) -> Result<wrangler_integrate::Setup, String> {
    Ok(wrangler_integrate::Setup {
        program: program()?,
        actions: menu_recipes(app).into_iter().map(|(id, label)| wrangler_integrate::QuickAction { id, label }).collect(),
        extensions: wrangler_engine::codec::Format::extensions().into_iter().map(String::from).collect(),
        asked,
    })
}

/// The recipes the menus offer, with their names: the ones switched on, in their order.
pub fn menu_recipes(app: &AppHandle) -> Vec<(String, String)> {
    presets::enabled(&app.state::<AppState>().dirs.recipes()).into_iter().map(|e| (e.id, e.name)).collect()
}

/// A recipe changed, so the menus that name recipes are out of date: the tray's now, and the
/// file managers' in the background (rewriting Finder's takes a moment).
pub fn menus_changed(app: &AppHandle) {
    crate::tray::refresh(app);
    let app = app.clone();
    std::thread::spawn(move || {
        use wrangler_integrate as wi;
        let Ok(setup) = menu_setup(&app, false) else { return };
        for t in wi::targets().into_iter().filter(|t| wi::installed(*t)) {
            if let Err(e) = wi::install(t, &setup) {
                eprintln!("{}: {e}", t.label());
            }
        }
    });
}
