// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Sean Hawkey

//! The Wrangler desktop host: windows, commands over the engine, image protocols, the menu bar
//! icon, updates, and launch modes for the file-manager actions.

mod commands;
mod launch;
#[cfg(target_os = "macos")]
mod macos;
mod pictures;
mod settings;
mod tray;
mod updates;

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use parking_lot::Mutex;
use percent_encoding::percent_decode_str;
use tauri::http::{Response, StatusCode};
use tauri::{Emitter, Manager};
use wrangler_engine::paths::AppDirs;
use wrangler_engine::scan::SourceFile;

pub use launch::{parse, Launch};

/// Everything the commands share.
pub struct AppState {
    pub dirs: AppDirs,
    /// Files in the current batch.
    pub sources: Mutex<Vec<SourceFile>>,
    /// Previews and the pictures behind `preview://<token>`.
    pub pictures: pictures::Pictures,
    pub cancel: Arc<AtomicBool>,
    /// Cancels the estimate still running when a newer one starts.
    pub estimating: Mutex<Option<Arc<AtomicBool>>>,
    /// Paths that arrived from the file manager before the window was ready, or between
    /// per-file launches (Explorer starts one process per selected file).
    pub pending: Mutex<Option<Launch>>,
    /// Set when started as `selftest <images> <report.json>`.
    pub selftest: Option<commands::SelfTestRun>,
    /// The main window should be on screen. False after a quick action or a start at login,
    /// until someone asks for the window.
    pub show_main: AtomicBool,
    /// The small unit opened under the menu bar icon, so it hides when it loses focus, as a
    /// menu would.
    pub popover: AtomicBool,
    /// The menu bar icon is up, so closing a window leaves Wrangler running.
    pub in_menu_bar: AtomicBool,
}

fn respond(bytes: Vec<u8>, mime: &str) -> Response<Vec<u8>> {
    Response::builder().header("Content-Type", mime).header("Cache-Control", "max-age=31536000, immutable").body(bytes).expect("valid response")
}

fn not_found() -> Response<Vec<u8>> {
    Response::builder().status(StatusCode::NOT_FOUND).body(Vec::new()).expect("valid response")
}

/// Merge a new launch into one waiting to be handled, so a multi-file Explorer selection
/// (one process per file) becomes one batch.
fn merge(pending: &mut Option<Launch>, next: Launch) {
    *pending = match (pending.take(), next) {
        (Some(Launch::Quick { recipe, mut paths }), Launch::Quick { recipe: r2, paths: p2 }) if recipe == r2 => {
            paths.extend(p2);
            Some(Launch::Quick { recipe, paths })
        }
        (Some(Launch::Open(mut a)), Launch::Open(b)) => {
            a.extend(b);
            Some(Launch::Open(a))
        }
        // Opening the app again never drops paths still waiting.
        (waiting, Launch::Plain) => waiting.or(Some(Launch::Plain)),
        (_, next) => Some(next),
    };
}

/// Bring the main window up (and, on a Mac, the Dock icon with it).
pub fn show_main(app: &tauri::AppHandle) {
    app.state::<AppState>().show_main.store(true, Ordering::Relaxed);
    #[cfg(target_os = "macos")]
    let _ = app.set_dock_visibility(true);
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.unminimize();
        let _ = w.show();
        let _ = w.set_focus();
    }
}

pub fn set_popover(app: &tauri::AppHandle, on: bool) {
    app.state::<AppState>().popover.store(on, Ordering::Relaxed);
}

/// Images dropped on the menu bar icon: they run with the chosen recipe, and the small unit
/// opens under the icon with the results.
#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
pub fn drop_on_icon(app: &tauri::AppHandle, paths: Vec<PathBuf>) {
    // Into the main window, unless Settings names a recipe to run on them there and then.
    let runs = settings::load(app).drop_runs.filter(|id| settings::menu_recipes(app).iter().any(|r| r.0 == *id));
    match runs {
        Some(recipe) => {
            deliver(app, Launch::Quick { recipe, paths });
            set_popover(app, true);
            tray::place_under_icon(app);
        }
        None => deliver(app, Launch::Open(paths)),
    }
}

pub(crate) fn deliver(app: &tauri::AppHandle, launch: Launch) {
    let state = app.state::<AppState>();
    merge(&mut state.pending.lock(), launch.clone());
    match launch {
        Launch::Quick { .. } => {
            set_popover(app, false);
            // A new window shows itself once drawn. One already made may be hidden (clicked away
            // from): bring it back for the new run.
            let existed = app.get_webview_window("quick").is_some();
            if let Some(w) = tray::open_quick(app)
                && existed
            {
                let _ = w.show();
                let _ = w.set_focus();
            }
            let _ = app.emit_to("quick", "launch://pending", ());
        }
        Launch::Open(_) => {
            show_main(app);
            let _ = app.emit_to("main", "launch://pending", ());
        }
        Launch::Plain => show_main(app),
    }
}

/// The right-click menus, without asking. The first time Wrangler runs it puts itself in every
/// file manager it finds, so it's there from the start. After that it only keeps the menus that
/// are still there in step with this copy of the app (its path, the pinned recipes, fixes to how
/// they start it), so menus someone removed stay removed.
fn keep_menus_current(app: &tauri::AppHandle) {
    use wrangler_integrate as wi;
    let mut s = settings::load(app);
    let first_time = !s.menus_set_up;
    let targets: Vec<_> = wi::targets().into_iter().filter(|t| first_time || wi::installed(*t)).collect();
    if let Ok(setup) = settings::menu_setup(app, false) {
        for t in targets {
            let _ = wi::install(t, &setup);
        }
    }
    if first_time {
        s.menus_set_up = true;
        let _ = settings::save(app, &s);
    }
}

/// What was chosen in Finder's right-click menu since we last looked: each runs, or loads into
/// the main window. True when there was something.
fn finder_requests(app: &tauri::AppHandle) -> bool {
    use wrangler_integrate::finder;
    let requests = finder::take_requests(&finder::dir());
    let any = !requests.is_empty();
    for r in requests {
        deliver(app, match r.recipe {
            Some(recipe) => Launch::Quick { recipe, paths: r.paths },
            None => Launch::Open(r.paths),
        });
    }
    any
}

pub fn run() {
    let args: Vec<String> = std::env::args().collect();
    let selftest = commands::SelfTestRun::from_args(&args);
    // Started at login: just the menu bar icon until someone asks for the window.
    let hidden = args.iter().any(|a| a == "--hidden");
    let first = if selftest.is_some() || hidden { Launch::Plain } else { parse(&args, &std::env::current_dir().unwrap_or_default()) };
    let state = AppState {
        dirs: AppDirs::detect(),
        sources: Mutex::new(Vec::new()),
        pictures: pictures::Pictures::default(),
        cancel: Arc::new(AtomicBool::new(false)),
        estimating: Mutex::new(None),
        pending: Mutex::new(None),
        selftest,
        show_main: AtomicBool::new(false),
        popover: AtomicBool::new(false),
        in_menu_bar: AtomicBool::new(false),
    };
    let thumbs_dir: PathBuf = state.dirs.thumbs();
    let thumb_pool = pictures::Pool::new(pictures::thumb_threads());

    let mut builder = tauri::Builder::default();
    // One Wrangler at a time: a second launch hands its files to the first, and windows open
    // where they were left. A self-test runs on its own beside any Wrangler already open, and
    // leaves the window size alone.
    if state.selftest.is_none() {
        builder = builder
            // A second start hands its request to this copy, with the folder it came from. A
            // second start at login (`--hidden`) asks for nothing.
            .plugin(tauri_plugin_single_instance::init(|app, argv, cwd| {
                if !argv.iter().any(|a| a == "--hidden") {
                    deliver(app, parse(&argv, std::path::Path::new(&cwd)));
                }
            }))
            // The window comes back where it was left, but whether it shows is ours to say
            // (`window_ready`): left to the plugin, the main window would open behind every
            // quick action and at login.
            .plugin(
                tauri_plugin_window_state::Builder::default()
                    .with_denylist(&["quick"])
                    .with_state_flags(tauri_plugin_window_state::StateFlags::all() - tauri_plugin_window_state::StateFlags::VISIBLE)
                    .build(),
            );
    }
    builder
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_autostart::init(tauri_plugin_autostart::MacosLauncher::LaunchAgent, Some(vec!["--hidden"])))
        .manage(state)
        .manage(updates::Updates::default())
        // thumb://localhost/<url-encoded path>: a cached WebP thumbnail of a file in the batch.
        .register_asynchronous_uri_scheme_protocol("thumb", move |ctx, request, responder| {
            let app = ctx.app_handle().clone();
            let dir = thumbs_dir.clone();
            let raw = request.uri().path().trim_start_matches('/').to_string();
            // ?size=256 on a standard display, 384 on a HiDPI one.
            let size = request.uri().query().and_then(|q| q.strip_prefix("size=")).and_then(|v| v.parse::<u32>().ok()).unwrap_or(384).clamp(128, 512);
            thumb_pool.spawn(move || {
                let path = PathBuf::from(percent_decode_str(&raw).decode_utf8_lossy().into_owned());
                let known = app.state::<AppState>().sources.lock().iter().any(|s| s.path == path);
                let res = if known {
                    wrangler_engine::thumbs::thumbnail(&path, size, &dir).ok().and_then(|t| std::fs::read(t).ok()).map_or_else(not_found, |b| respond(b, "image/webp"))
                } else {
                    not_found()
                };
                responder.respond(res);
            });
        })
        // preview://localhost/<token>: a picture for the monitor. Off the main thread, because an
        // original at full size can be large, or need converting.
        .register_asynchronous_uri_scheme_protocol("preview", |ctx, request, responder| {
            let app = ctx.app_handle().clone();
            let token = request.uri().path().trim_matches('/').to_string();
            std::thread::spawn(move || {
                let found = app.state::<AppState>().pictures.serve(&token);
                responder.respond(found.map_or_else(not_found, |(b, m)| respond(b.as_ref().clone(), m)));
            });
        })
        .invoke_handler(tauri::generate_handler![
            commands::formats,
            commands::image_extensions,
            commands::recipes,
            commands::load_recipe,
            commands::save_recipe,
            commands::delete_recipe,
            commands::scan,
            commands::forget,
            commands::plan,
            commands::estimate,
            commands::preview,
            commands::preview_focus,
            commands::run,
            commands::quick_run,
            commands::open_main,
            commands::open_image,
            commands::cancel,
            commands::undo,
            commands::history,
            commands::take_launch,
            commands::integrations,
            commands::set_menus,
            commands::rename_recipe,
            commands::duplicate_recipe,
            commands::save_recipe_as,
            commands::reset_recipe,
            commands::restore_recipes,
            commands::set_recipe_on,
            commands::set_recipe_order,
            commands::app_settings,
            commands::set_app_settings,
            commands::set_drop_recipe,
            commands::window_ready,
            commands::selftest_config,
            commands::selftest_done,
            updates::update_check,
            updates::update_found,
            updates::update_install,
        ])
        .setup(move |app| {
            let handle = app.handle().clone();
            // Keep the undo store tidy: drop runs older than 30 days.
            let runs = app.state::<AppState>().dirs.runs();
            std::thread::spawn(move || wrangler_engine::journal::prune(&runs, 30));
            let s = settings::load(&handle);
            let selftest = app.state::<AppState>().selftest.is_some();
            if s.menu_bar && tray::available() && tray::build(&handle).is_ok() {
                app.state::<AppState>().in_menu_bar.store(true, Ordering::Relaxed);
            }
            if !selftest {
                updates::watch(handle.clone());
                let h = handle.clone();
                std::thread::spawn(move || keep_menus_current(&h));
            }
            // Started from Finder's menu, there's a request waiting: run that, and leave the main
            // window out of it.
            let from_finder = cfg!(target_os = "macos") && !selftest && finder_requests(&handle);
            if hidden {
                #[cfg(target_os = "macos")]
                let _ = handle.set_dock_visibility(false);
            } else if !from_finder || first != Launch::Plain {
                deliver(&handle, first.clone());
            }
            Ok(())
        })
        .on_window_event(|window, event| {
            let app = window.app_handle();
            let state = app.state::<AppState>();
            let in_menu_bar = state.in_menu_bar.load(Ordering::Relaxed);
            match (window.label(), event) {
                // Opened under the menu bar icon, the small unit behaves like a menu: click away
                // and it goes.
                ("quick", tauri::WindowEvent::Focused(false)) if state.popover.load(Ordering::Relaxed) => {
                    let _ = window.hide();
                }
                // With the icon in the menu bar, closing a window leaves Wrangler there.
                (label, tauri::WindowEvent::CloseRequested { api, .. }) if in_menu_bar => {
                    api.prevent_close();
                    let _ = window.hide();
                    if label == "main" {
                        state.show_main.store(false, Ordering::Relaxed);
                        #[cfg(target_os = "macos")]
                        let _ = app.set_dock_visibility(false);
                    }
                }
                // Without it, a quick action that closes with the main window never shown means
                // there's nothing left to do; and closing the main window ends the app.
                ("quick", tauri::WindowEvent::Destroyed) => {
                    let main_visible = app.get_webview_window("main").and_then(|w| w.is_visible().ok()).unwrap_or(false);
                    if !main_visible {
                        app.exit(0);
                    }
                }
                ("main", tauri::WindowEvent::Destroyed) => app.exit(0),
                _ => {}
            }
        })
        .build(tauri::generate_context!())
        .expect("Wrangler failed to start")
        .run(|_app, _event| {
            #[cfg(target_os = "macos")]
            match _event {
                // Files from Open With and the Dock icon arrive as an event, not arguments. So does
                // the Finder extension's wake-up call, which carries nothing: what was chosen is
                // in its request files.
                tauri::RunEvent::Opened { urls } => {
                    if urls.iter().any(|u| u.scheme() == wrangler_integrate::finder::SCHEME) {
                        finder_requests(_app);
                    }
                    let paths: Vec<PathBuf> = urls.iter().filter_map(|u| u.to_file_path().ok()).collect();
                    if !paths.is_empty() {
                        deliver(_app, Launch::Open(paths));
                    }
                }
                // A click on the Dock icon.
                tauri::RunEvent::Reopen { .. } => show_main(_app),
                _ => {}
            }
        });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn explorer_launches_fold_into_one_batch() {
        let mut p = None;
        merge(&mut p, Launch::Quick { recipe: "web".into(), paths: vec!["/a".into()] });
        merge(&mut p, Launch::Quick { recipe: "web".into(), paths: vec!["/b".into()] });
        assert_eq!(p, Some(Launch::Quick { recipe: "web".into(), paths: vec!["/a".into(), "/b".into()] }));
        merge(&mut p, Launch::Open(vec!["/c".into()]));
        assert_eq!(p, Some(Launch::Open(vec!["/c".into()])));
    }
}
