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
}

impl Default for Settings {
    fn default() -> Self {
        Self { pins: vec!["web-ready".into(), "just-compress".into(), "shopify-product".into()], menu_bar: true, auto_update: true, drop_recipe: "web-ready".into() }
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

/// The pinned recipes that still exist, with their names.
pub fn pinned(app: &AppHandle, s: &Settings) -> Vec<(String, String)> {
    let dir = app.state::<AppState>().dirs.recipes();
    s.pins.iter().filter_map(|id| presets::load(&dir, id).ok().map(|r| (id.clone(), r.name))).collect()
}
