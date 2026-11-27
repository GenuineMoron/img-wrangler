// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Sean Hawkey

//! Finder's own right-click menu, through the Finder Sync extension inside Wrangler.app
//! (apps/desktop/src-tauri/finder). The app and the extension share one folder:
//!
//! - `menu.json`: what the extension shows. Written here, by `write_menu`.
//! - `requests/<id>.json`: what was chosen, on which files. Written by the extension, read once
//!   by the app with `take_requests`.
//!
//! The extension wakes the app by opening `img-wrangler://finder`. The address carries nothing:
//! the app acts only on request files, so a web page that opens the address can't make it run.
//! macOS decides whether an extension loads (`pluginkit`), and people can turn one off in
//! System Settings, so `state` asks, and `turn_on` only overrules a choice when told to.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use serde::{Deserialize, Serialize};

use crate::{Error, Result, Setup, OPEN_LABEL};

/// The address the extension opens to wake the app.
pub const SCHEME: &str = "img-wrangler";
/// The extension's bundle identifier: the app's, plus `.finder`.
pub const EXTENSION_ID: &str = "com.genuinemoron.wrangler.finder";
/// A request older than this was never picked up: the app wasn't there. Don't surprise anyone
/// by running it at the next launch.
const FRESH: Duration = Duration::from_secs(60);

/// `~/Library/Application Support/img-wrangler/finder`, the folder the extension's sandbox may
/// use (the same path is in FinderSync.swift and finder.entitlements).
pub fn dir() -> PathBuf {
    dirs::config_dir().unwrap_or_else(|| crate::home().join("Library/Application Support")).join("img-wrangler/finder")
}

#[derive(Serialize)]
struct Menu<'a> {
    extensions: &'a [String],
    open: &'a str,
    recipes: Vec<Entry<'a>>,
}

#[derive(Serialize)]
struct Entry<'a> {
    id: &'a str,
    label: &'a str,
}

/// The menu file's text.
pub fn menu_json(setup: &Setup) -> String {
    let menu = Menu { extensions: &setup.extensions, open: OPEN_LABEL, recipes: setup.actions.iter().map(|a| Entry { id: &a.id, label: &a.label }).collect() };
    serde_json::to_string_pretty(&menu).expect("plain strings")
}

pub fn write_menu(dir: &Path, setup: &Setup) -> Result<PathBuf> {
    fs::create_dir_all(dir.join("requests")).map_err(|e| Error::Io(dir.into(), e))?;
    let path = dir.join("menu.json");
    fs::write(&path, menu_json(setup)).map_err(|e| Error::Io(path.clone(), e))?;
    Ok(path)
}

/// Remove the menu, so the extension shows nothing, and any requests left behind.
pub fn remove_menu(dir: &Path) -> Result<()> {
    let menu = dir.join("menu.json");
    if menu.exists() {
        fs::remove_file(&menu).map_err(|e| Error::Io(menu.clone(), e))?;
    }
    let _ = fs::remove_dir_all(dir.join("requests"));
    Ok(())
}

/// Something chosen in Finder's menu.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Request {
    /// The recipe to run at once. None loads the files into the main window.
    #[serde(default)]
    pub recipe: Option<String>,
    pub paths: Vec<PathBuf>,
}

/// Every request waiting, oldest first, each handed over once: the file is removed as it's
/// read. Stale and unreadable ones are dropped, and so are paths that aren't absolute.
pub fn take_requests(dir: &Path) -> Vec<Request> {
    let requests = dir.join("requests");
    let Ok(entries) = fs::read_dir(&requests) else { return Vec::new() };
    let mut found: Vec<(SystemTime, Request)> = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().is_none_or(|x| x != "json") {
            continue;
        }
        let written = entry.metadata().and_then(|m| m.modified()).unwrap_or(SystemTime::UNIX_EPOCH);
        let text = fs::read_to_string(&path);
        // Claimed by removing it: if another copy of the app got there first, leave it to them.
        if fs::remove_file(&path).is_err() {
            continue;
        }
        if written.elapsed().is_ok_and(|age| age > FRESH) {
            continue;
        }
        let Ok(mut request) = text.map_err(|_| ()).and_then(|t| serde_json::from_str::<Request>(&t).map_err(|_| ())) else { continue };
        request.paths.retain(|p| p.is_absolute());
        if !request.paths.is_empty() {
            found.push((written, request));
        }
    }
    found.sort_by_key(|(t, _)| *t);
    found.into_iter().map(|(_, r)| r).collect()
}

/// The extension inside the app a program belongs to, when there is one.
pub fn appex_for(program: &Path) -> Option<PathBuf> {
    // <Name>.app/Contents/MacOS/<program>
    let macos = program.parent()?;
    let app = macos.parent()?.parent()?;
    let appex = app.join("Contents/PlugIns/WranglerFinder.appex");
    (macos.ends_with("Contents/MacOS") && app.extension().is_some_and(|e| e == "app") && appex.is_dir()).then_some(appex)
}

/// Where the extension stands with macOS.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    /// macOS doesn't know of it.
    Unknown,
    /// Known, and no one has chosen: Finder doesn't load it yet.
    Undecided,
    /// Turned off by the person, in System Settings.
    Off,
    /// Finder loads it.
    On,
}

/// Reads `pluginkit -m -i <id>`: one line per copy it knows, starting `+` (in use), `-` (turned
/// off) or blank (no choice made).
pub fn parse_state(listing: &str) -> State {
    let mut state = State::Unknown;
    for line in listing.lines().filter(|l| l.contains(EXTENSION_ID)) {
        state = match line.trim_start().chars().next() {
            Some('+') => return State::On,
            Some('-') => State::Off,
            _ if state == State::Unknown => State::Undecided,
            _ => state,
        };
    }
    state
}

#[cfg(target_os = "macos")]
fn pluginkit(args: &[&str]) -> Option<String> {
    let out = std::process::Command::new("/usr/bin/pluginkit").args(args).output().ok()?;
    Some(String::from_utf8_lossy(&out.stdout).into_owned())
}

#[cfg(target_os = "macos")]
pub fn state() -> State {
    pluginkit(&["-m", "-i", EXTENSION_ID]).map_or(State::Unknown, |l| parse_state(&l))
}

#[cfg(not(target_os = "macos"))]
pub fn state() -> State {
    State::Unknown
}

/// Make sure macOS knows the extension and Finder loads it. A choice to turn it off is left
/// alone unless `overrule` is set (the person just asked for the menu). Returns where it stands.
#[cfg(target_os = "macos")]
pub fn turn_on(appex: &Path, overrule: bool) -> State {
    if state() == State::Unknown {
        // macOS finds extensions of apps in /Applications by itself. Anywhere else, tell it.
        if let Some(app) = appex.ancestors().nth(3) {
            let lsregister = "/System/Library/Frameworks/CoreServices.framework/Frameworks/LaunchServices.framework/Support/lsregister";
            let _ = std::process::Command::new(lsregister).arg("-f").arg("-R").arg(app).status();
        }
        pluginkit(&["-a", &appex.to_string_lossy()]);
    }
    match state() {
        State::Undecided => {
            pluginkit(&["-e", "use", "-i", EXTENSION_ID]);
        }
        State::Off if overrule => {
            pluginkit(&["-e", "use", "-i", EXTENSION_ID]);
        }
        _ => {}
    }
    state()
}

#[cfg(not(target_os = "macos"))]
pub fn turn_on(_appex: &Path, _overrule: bool) -> State {
    State::Unknown
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::QuickAction;

    fn setup() -> Setup {
        Setup {
            program: "/Applications/Wrangler.app/Contents/MacOS/img-wrangler-app".into(),
            actions: vec![QuickAction { id: "compress-for-web".into(), label: "Compress for web".into() }],
            extensions: vec!["jpg".into(), "png".into()],
            asked: true,
        }
    }

    #[test]
    fn menu_lists_recipes_and_extensions() {
        let v: serde_json::Value = serde_json::from_str(&menu_json(&setup())).unwrap();
        assert_eq!(v["recipes"][0]["id"], "compress-for-web");
        assert_eq!(v["recipes"][0]["label"], "Compress for web");
        assert_eq!(v["extensions"], serde_json::json!(["jpg", "png"]));
        assert_eq!(v["open"], "Open in Wrangler…");
    }

    #[test]
    fn requests_are_taken_once_and_checked() {
        let dir = tempfile::tempdir().unwrap();
        write_menu(dir.path(), &setup()).unwrap();
        let requests = dir.path().join("requests");
        let abs = if cfg!(windows) { r"C:\photos\a.jpg" } else { "/photos/a.jpg" };
        fs::write(requests.join("1.json"), serde_json::json!({ "recipe": "compress-for-web", "paths": [abs, "relative/b.jpg"] }).to_string()).unwrap();
        fs::write(requests.join("2.json"), serde_json::json!({ "paths": [abs] }).to_string()).unwrap();
        fs::write(requests.join("3.json"), "not json").unwrap();
        fs::write(requests.join("4.tmp"), "half written").unwrap();

        let mut taken = take_requests(dir.path());
        taken.sort_by_key(|r| r.recipe.is_none());
        assert_eq!(taken, vec![Request { recipe: Some("compress-for-web".into()), paths: vec![abs.into()] }, Request { recipe: None, paths: vec![abs.into()] }]);
        assert!(take_requests(dir.path()).is_empty(), "each request is handed over once");
        assert!(requests.join("4.tmp").exists(), "a request still being written is left alone");
    }

    #[test]
    fn stale_requests_are_dropped() {
        let dir = tempfile::tempdir().unwrap();
        let requests = dir.path().join("requests");
        fs::create_dir_all(&requests).unwrap();
        let old = requests.join("old.json");
        fs::write(&old, r#"{"paths":["/a.jpg"]}"#).unwrap();
        let then = SystemTime::now() - Duration::from_secs(600);
        fs::File::options().write(true).open(&old).unwrap().set_modified(then).unwrap();
        assert!(take_requests(dir.path()).is_empty());
        assert!(!old.exists());
    }

    #[test]
    fn reads_what_pluginkit_says() {
        assert_eq!(parse_state(""), State::Unknown);
        assert_eq!(parse_state("     com.genuinemoron.wrangler.finder(0.1.0)\n"), State::Undecided);
        assert_eq!(parse_state("+    com.genuinemoron.wrangler.finder(0.1.0)\n"), State::On);
        assert_eq!(parse_state("-    com.genuinemoron.wrangler.finder(0.1.0)\n"), State::Off);
        // Two copies known, one in use.
        assert_eq!(parse_state("-    com.genuinemoron.wrangler.finder(0.1.0)\n+    com.genuinemoron.wrangler.finder(0.1.1)\n"), State::On);
        assert_eq!(parse_state("+    com.other.app.finder(1.0)\n"), State::Unknown);
    }

    #[test]
    fn finds_the_extension_inside_the_app() {
        let dir = tempfile::tempdir().unwrap();
        let app = dir.path().join("Wrangler.app");
        fs::create_dir_all(app.join("Contents/PlugIns/WranglerFinder.appex")).unwrap();
        fs::create_dir_all(app.join("Contents/MacOS")).unwrap();
        let program = app.join("Contents/MacOS/img-wrangler-app");
        assert_eq!(appex_for(&program), Some(app.join("Contents/PlugIns/WranglerFinder.appex")));
        assert_eq!(appex_for(Path::new("/usr/local/bin/img-wrangler")), None);
    }
}
