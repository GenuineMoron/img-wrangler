// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Sean Hawkey

//! New versions. Wrangler looks at genuinemoron.github.io/img-wrangler/latest.json shortly after
//! it starts and every few hours (unless that's turned off), and says so when there's something
//! newer, with its changelog. Installing downloads the update, checks its signature against the
//! key built into the app, installs it and restarts. An Arch package can't be updated from inside
//! (makepkg doesn't run as root), so there the app offers the install command instead.

use std::time::Duration;

use parking_lot::Mutex;
use serde::Serialize;
use tauri::ipc::Channel;
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_updater::{Update, UpdaterExt};

pub const COMMAND: &str = "curl -fsSL https://genuinemoron.github.io/img-wrangler/install | sh";

#[derive(Debug, Clone, Serialize)]
pub struct UpdateInfo {
    pub version: String,
    pub current: String,
    pub date: Option<String>,
    /// The changelog for the new version, in Markdown.
    pub notes: String,
    /// When the app can't update itself: the command that does it.
    pub manual: Option<String>,
}

#[derive(Default)]
pub struct Updates {
    found: Mutex<Option<(UpdateInfo, Update)>>,
}

/// Installed by pacman (the Arch package), which only the package manager may replace.
fn from_pacman() -> bool {
    cfg!(target_os = "linux") && std::fs::read_dir("/var/lib/pacman/local").is_ok_and(|d| d.flatten().any(|e| e.file_name().to_string_lossy().starts_with("img-wrangler-bin-")))
}

pub async fn check(app: &AppHandle) -> Result<Option<UpdateInfo>, String> {
    let update = app.updater().map_err(|e| e.to_string())?.check().await.map_err(|e| e.to_string())?;
    let state = app.state::<Updates>();
    let Some(u) = update else {
        *state.found.lock() = None;
        return Ok(None);
    };
    let info = UpdateInfo {
        version: u.version.clone(),
        current: u.current_version.clone(),
        date: u.date.map(|d| d.date().to_string()),
        notes: u.body.clone().unwrap_or_default(),
        manual: from_pacman().then(|| COMMAND.to_string()),
    };
    *state.found.lock() = Some((info.clone(), u));
    crate::tray::refresh(app);
    Ok(Some(info))
}

/// What the last look found.
pub fn found(app: &AppHandle) -> Option<UpdateInfo> {
    app.state::<Updates>().found.lock().as_ref().map(|(i, _)| i.clone())
}

/// Look now and then, and tell the main window when there's something new.
pub fn watch(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        tokio_sleep(Duration::from_secs(8)).await;
        loop {
            if crate::settings::load(&app).auto_update
                && let Ok(Some(info)) = check(&app).await
            {
                let _ = app.emit_to("main", "update://available", info);
            }
            tokio_sleep(Duration::from_secs(6 * 60 * 60)).await;
        }
    });
}

async fn tokio_sleep(d: Duration) {
    let _ = tauri::async_runtime::spawn_blocking(move || std::thread::sleep(d)).await;
}

#[tauri::command]
pub async fn update_check(app: AppHandle) -> Result<Option<UpdateInfo>, String> {
    check(&app).await
}

/// What the last look found, for a window that opens after it.
#[tauri::command]
pub fn update_found(app: AppHandle) -> Option<UpdateInfo> {
    found(&app)
}

#[derive(Clone, Serialize)]
pub struct Progress {
    pub done: u64,
    pub total: Option<u64>,
}

/// Download, verify and install the update found, then restart into it.
#[tauri::command]
pub async fn update_install(app: AppHandle, on_progress: Channel<Progress>) -> Result<(), String> {
    let Some((info, update)) = app.state::<Updates>().found.lock().take() else { return Err("no update to install".into()) };
    if info.manual.is_some() {
        return Err("this copy is updated by its package manager".into());
    }
    let mut done = 0u64;
    update
        .download_and_install(
            |chunk, total| {
                done += chunk as u64;
                let _ = on_progress.send(Progress { done, total });
            },
            || {},
        )
        .await
        .map_err(|e| e.to_string())?;
    app.restart();
}
