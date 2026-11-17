// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Sean Hawkey

//! The Wrangler desktop host: windows, commands over the engine, image protocols, and launch
//! modes for the file-manager actions.

mod commands;
mod launch;

use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use parking_lot::Mutex;
use percent_encoding::percent_decode_str;
use tauri::http::{Response, StatusCode};
use tauri::{Emitter, Manager, WebviewUrl, WebviewWindowBuilder};
use wrangler_engine::paths::AppDirs;
use wrangler_engine::scan::SourceFile;

pub use launch::{parse, Launch};

/// Everything the commands share.
pub struct AppState {
    pub dirs: AppDirs,
    /// Files in the current batch.
    pub sources: Mutex<Vec<SourceFile>>,
    /// Encoded previews served on `preview://<token>`, newest last; a few are kept.
    pub previews: Mutex<Vec<(String, Vec<u8>, &'static str)>>,
    pub cancel: Arc<AtomicBool>,
    /// Cancels the estimate still running when a newer one starts.
    pub estimating: Mutex<Option<Arc<AtomicBool>>>,
    /// Paths that arrived from the file manager before the window was ready, or between
    /// per-file launches (Explorer starts one process per selected file).
    pub pending: Mutex<Option<Launch>>,
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

pub(crate) fn deliver(app: &tauri::AppHandle, launch: Launch) {
    let state = app.state::<AppState>();
    merge(&mut state.pending.lock(), launch.clone());
    match launch {
        Launch::Quick { .. } => {
            if app.get_webview_window("quick").is_none() {
                let _ = WebviewWindowBuilder::new(app, "quick", WebviewUrl::App("index.html#quick".into()))
                    .title("Wrangler")
                    .inner_size(320.0, 410.0)
                    .resizable(false)
                    .decorations(false)
                    .always_on_top(true)
                    .background_color(tauri::window::Color(0xe9, 0xe8, 0xe4, 0xff))
                    .build();
            }
            let _ = app.emit_to("quick", "launch://pending", ());
        }
        Launch::Open(_) => {
            if let Some(w) = app.get_webview_window("main") {
                let _ = w.show();
                let _ = w.set_focus();
            }
            let _ = app.emit_to("main", "launch://pending", ());
        }
        Launch::Plain => {
            if let Some(w) = app.get_webview_window("main") {
                let _ = w.show();
                let _ = w.set_focus();
            }
        }
    }
}

pub fn run() {
    let args: Vec<String> = std::env::args().collect();
    let first = parse(&args);
    let state = AppState {
        dirs: AppDirs::detect(),
        sources: Mutex::new(Vec::new()),
        previews: Mutex::new(Vec::new()),
        cancel: Arc::new(AtomicBool::new(false)),
        estimating: Mutex::new(None),
        pending: Mutex::new(None),
    };
    let thumbs_dir: PathBuf = state.dirs.thumbs();

    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, argv, _cwd| deliver(app, parse(&argv))))
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .manage(state)
        // thumb://localhost/<url-encoded path>: a cached WebP thumbnail of a file in the batch.
        .register_asynchronous_uri_scheme_protocol("thumb", move |ctx, request, responder| {
            let app = ctx.app_handle().clone();
            let dir = thumbs_dir.clone();
            let raw = request.uri().path().trim_start_matches('/').to_string();
            std::thread::spawn(move || {
                let path = PathBuf::from(percent_decode_str(&raw).decode_utf8_lossy().into_owned());
                let known = app.state::<AppState>().sources.lock().iter().any(|s| s.path == path);
                let res = if known {
                    wrangler_engine::thumbs::thumbnail(&path, 384, &dir).ok().and_then(|t| std::fs::read(t).ok()).map_or_else(not_found, |b| respond(b, "image/webp"))
                } else {
                    not_found()
                };
                responder.respond(res);
            });
        })
        .register_uri_scheme_protocol("preview", |ctx, request| {
            let token = request.uri().path().trim_matches('/').to_string();
            let state = ctx.app_handle().state::<AppState>();
            let found = state.previews.lock().iter().find(|(t, _, _)| *t == token).map(|(_, b, m)| (b.clone(), *m));
            found.map_or_else(not_found, |(b, m)| respond(b, m))
        })
        .invoke_handler(tauri::generate_handler![
            commands::formats,
            commands::recipes,
            commands::load_recipe,
            commands::save_recipe,
            commands::delete_recipe,
            commands::scan,
            commands::forget,
            commands::plan,
            commands::estimate,
            commands::preview,
            commands::run,
            commands::quick_run,
            commands::open_main,
            commands::cancel,
            commands::undo,
            commands::history,
            commands::take_launch,
            commands::integrations,
            commands::set_integrations,
        ])
        .setup(move |app| {
            let handle = app.handle().clone();
            // Keep the undo store tidy: drop runs older than 30 days.
            let runs = app.state::<AppState>().dirs.runs();
            std::thread::spawn(move || wrangler_engine::journal::prune(&runs, 30));
            deliver(&handle, first.clone());
            Ok(())
        })
        // A quick action opens only the small window; when it closes and the main window was
        // never shown, the app has nothing left to do.
        .on_window_event(|window, event| {
            if window.label() == "quick" && matches!(event, tauri::WindowEvent::Destroyed) {
                let app = window.app_handle();
                let main_visible = app.get_webview_window("main").and_then(|w| w.is_visible().ok()).unwrap_or(false);
                if !main_visible {
                    app.exit(0);
                }
            }
        })
        .build(tauri::generate_context!())
        .expect("Wrangler failed to start")
        .run(|_app, _event| {
            // macOS hands over files from Open With and the Dock icon as an event, not arguments.
            #[cfg(target_os = "macos")]
            if let tauri::RunEvent::Opened { urls } = _event {
                let paths: Vec<PathBuf> = urls.iter().filter_map(|u| u.to_file_path().ok()).collect();
                if !paths.is_empty() {
                    deliver(_app, Launch::Open(paths));
                }
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
