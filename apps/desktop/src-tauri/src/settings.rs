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
    /// Recipes offered in the right-click menus and the menu bar menu.
    pub pins: Vec<String>,
    /// Keep the hat in the menu bar or tray, with Wrangler running when its window closes.
    pub menu_bar: bool,
    /// Look for a new version now and then.
    pub auto_update: bool,
    /// The recipe that runs on images dropped on the menu bar icon.
    pub drop_recipe: String,
    /// Wrangler has put itself in the file managers' right-click menus once. After that, the
    /// menus are only kept current where they still are, so removing them sticks.
    pub menus_set_up: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self { pins: vec!["web-ready".into(), "just-compress".into(), "shopify-product".into()], menu_bar: true, auto_update: true, drop_recipe: "web-ready".into(), menus_set_up: false }
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

/// What the right-click menus are built from: this copy of the app and the pinned recipes.
/// `asked` says the person asked for the menus just now (see `wrangler_integrate::Setup`).
pub fn menu_setup(app: &AppHandle, s: &Settings, asked: bool) -> Result<wrangler_integrate::Setup, String> {
    Ok(wrangler_integrate::Setup {
        program: program()?,
        actions: pinned(app, s).into_iter().map(|(id, label)| wrangler_integrate::QuickAction { id, label }).collect(),
        extensions: wrangler_engine::codec::Format::extensions().into_iter().map(String::from).collect(),
        asked,
    })
}

/// The pinned recipes that still exist, with their names.
pub fn pinned(app: &AppHandle, s: &Settings) -> Vec<(String, String)> {
    let dir = app.state::<AppState>().dirs.recipes();
    s.pins.iter().filter_map(|id| presets::load(&dir, id).ok().map(|r| (id.clone(), r.name))).collect()
}
