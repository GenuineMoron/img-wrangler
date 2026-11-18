// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Sean Hawkey

//! Wrangler in the menu bar (macOS) or the system tray (Windows, Linux): the hat and its menu.
//! On a Mac, images dropped on the hat run straight away, and the small unit opens under it
//! with the results and Undo.
//!
//! The menu is what someone reaches for without opening the window: run a pinned recipe on some
//! images, choose what a drop on the hat does, take back the last run, and the app's own
//! business. It's rebuilt whenever one of those changes, so its labels are always current.
//!
//! The icons are made from icons/tray/*.svg with the CLI:
//!   img-wrangler run icons/tray/menubar.svg --size 66 --format png --look lossless --to …
//!   img-wrangler run icons/tray/tray.svg --size 64 --format png --look lossless --to …

use std::path::PathBuf;
use std::time::Duration;

use tauri::image::Image;
use tauri::menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem, Submenu};
use tauri::tray::TrayIconBuilder;
use tauri::{AppHandle, Emitter, Manager, PhysicalPosition, WebviewUrl, WebviewWindowBuilder};

use crate::launch::Launch;
use crate::{settings, AppState};

pub const ID: &str = "wrangler";
const QUICK: (f64, f64) = (320.0, 410.0);

/// Whether this system can show a tray icon. On Linux the tray needs the AppIndicator library,
/// and the tray code ends the app if it's missing, so look first.
pub fn available() -> bool {
    #[cfg(target_os = "linux")]
    {
        ["libayatana-appindicator3.so.1", "libappindicator3.so.1", "libayatana-appindicator3.so", "libappindicator3.so"]
            .iter()
            .any(|name| unsafe { libloading::Library::new(name) }.is_ok())
    }
    #[cfg(not(target_os = "linux"))]
    true
}

fn images(n: u32) -> String {
    if n == 1 { "1 image".into() } else { format!("{n} images") }
}

fn menu(app: &AppHandle) -> tauri::Result<Menu<tauri::Wry>> {
    let s = settings::load(app);
    let pinned = settings::pinned(app, &s);
    let menu = Menu::new(app)?;
    menu.append(&MenuItem::with_id(app, "open", "Open Wrangler", true, None::<&str>)?)?;
    menu.append(&PredefinedMenuItem::separator(app)?)?;

    // Run a pinned recipe on images chosen now.
    for (id, name) in &pinned {
        menu.append(&MenuItem::with_id(app, format!("run:{id}"), format!("{name} on images…"), true, None::<&str>)?)?;
    }
    // Only a Mac can drop on a menu bar icon.
    if cfg!(target_os = "macos") && !pinned.is_empty() {
        let drop = Submenu::with_id(app, "drop", "Images dropped on the hat", true)?;
        for (id, name) in &pinned {
            drop.append(&CheckMenuItem::with_id(app, format!("drop:{id}"), name, true, *id == s.drop_recipe, None::<&str>)?)?;
        }
        menu.append(&drop)?;
    }
    menu.append(&PredefinedMenuItem::separator(app)?)?;

    // Take back the last run, from anywhere.
    let last = wrangler_engine::journal::history(&app.state::<AppState>().dirs.runs()).into_iter().find(|r| !r.undone && r.files > 0);
    match last {
        Some(r) => menu.append(&MenuItem::with_id(app, format!("undo:{}", r.id), format!("Undo “{}” on {}", r.recipe, images(r.files)), true, None::<&str>)?)?,
        None => menu.append(&MenuItem::with_id(app, "undo", "Nothing to undo", false, None::<&str>)?)?,
    }
    menu.append(&PredefinedMenuItem::separator(app)?)?;

    let update = crate::updates::found(app).map(|u| format!("Update to {}…", u.version));
    menu.append(&MenuItem::with_id(app, "updates", update.as_deref().unwrap_or("Check for updates…"), true, None::<&str>)?)?;
    menu.append(&MenuItem::with_id(app, "settings", "Settings…", true, None::<&str>)?)?;
    menu.append(&PredefinedMenuItem::separator(app)?)?;
    menu.append(&MenuItem::with_id(app, "quit", "Quit Wrangler", true, None::<&str>)?)?;
    Ok(menu)
}

pub fn build(app: &AppHandle) -> tauri::Result<()> {
    let icon = if cfg!(target_os = "macos") { Image::from_bytes(include_bytes!("../icons/tray/menubar.png"))? } else { Image::from_bytes(include_bytes!("../icons/tray/tray.png"))? };
    let tray = TrayIconBuilder::with_id(ID)
        .icon(icon)
        .icon_as_template(true)
        .tooltip("Wrangler")
        .menu(&menu(app)?)
        .show_menu_on_left_click(true)
        .on_menu_event(|app, e| handle(app, e.id().as_ref()))
        .build(app)?;
    #[cfg(target_os = "macos")]
    {
        let app = app.clone();
        let _ = tray.with_inner_tray_icon(move |inner| crate::macos::accept_drops(inner.ns_status_item(), app));
    }
    #[cfg(not(target_os = "macos"))]
    let _ = tray;
    Ok(())
}

/// Rebuild the menu after something it shows has changed.
pub fn refresh(app: &AppHandle) {
    if let (Some(tray), Ok(m)) = (app.tray_by_id(ID), menu(app)) {
        let _ = tray.set_menu(Some(m));
    }
}

/// A word beside the hat for a moment (macOS and Linux show it; Windows has no room for it).
fn flash(app: &AppHandle, text: &str) {
    let Some(tray) = app.tray_by_id(ID) else { return };
    let _ = tray.set_title(Some(text));
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(2500));
        let _ = tray.set_title(None::<&str>);
    });
}

fn handle(app: &AppHandle, id: &str) {
    match id {
        "open" => crate::show_main(app),
        "updates" => {
            crate::show_main(app);
            let _ = app.emit_to("main", "menu://updates", ());
        }
        "settings" => {
            crate::show_main(app);
            let _ = app.emit_to("main", "menu://settings", ());
        }
        "quit" => app.exit(0),
        run if run.starts_with("run:") => {
            let recipe = run[4..].to_string();
            let app = app.clone();
            use tauri_plugin_dialog::DialogExt;
            app.clone().dialog().file().set_title("Choose images").pick_files(move |files| {
                let paths: Vec<PathBuf> = files.unwrap_or_default().into_iter().filter_map(|f| f.into_path().ok()).collect();
                if !paths.is_empty() {
                    crate::deliver(&app, Launch::Quick { recipe, paths });
                }
            });
        }
        drop if drop.starts_with("drop:") => {
            let mut s = settings::load(app);
            s.drop_recipe = drop[5..].to_string();
            let _ = settings::save(app, &s);
            refresh(app);
        }
        undo if undo.starts_with("undo:") => {
            let (app, run) = (app.clone(), undo[5..].to_string());
            std::thread::spawn(move || {
                let done = wrangler_engine::journal::undo(&app.state::<AppState>().dirs.runs(), &run).is_ok();
                flash(&app, if done { "Undone" } else { "Couldn't undo" });
                refresh(&app);
            });
        }
        _ => {}
    }
}

/// The small unit's window, made when first needed. It shows itself once drawn.
pub fn open_quick(app: &AppHandle) -> Option<tauri::WebviewWindow> {
    if let Some(w) = app.get_webview_window("quick") {
        return Some(w);
    }
    WebviewWindowBuilder::new(app, "quick", WebviewUrl::App("index.html#quick".into()))
        .title("Wrangler")
        .inner_size(QUICK.0, QUICK.1)
        .resizable(false)
        .decorations(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .visible(false)
        .center()
        .background_color(tauri::window::Color(0xe9, 0xe8, 0xe4, 0xff))
        .build()
        .ok()
}

/// Put the small unit just under the hat, as a menu would open.
#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
pub fn place_under_icon(app: &AppHandle) {
    let (Some(w), Some(rect)) = (app.get_webview_window("quick"), app.tray_by_id(ID).and_then(|t| t.rect().ok().flatten())) else { return };
    let scale = w.scale_factor().unwrap_or(1.0);
    let (pos, size) = (rect.position.to_physical::<f64>(scale), rect.size.to_physical::<f64>(scale));
    let x = pos.x + size.width / 2.0 - QUICK.0 * scale / 2.0;
    let y = pos.y + size.height + 6.0 * scale;
    let _ = w.set_position(PhysicalPosition::new(x.max(0.0), y.max(0.0)));
}
