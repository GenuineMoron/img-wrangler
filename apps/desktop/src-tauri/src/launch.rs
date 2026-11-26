// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Sean Hawkey

//! How the app was asked to start: plain, `open <paths…>`, or `quick --recipe <id> <paths…>`
//! (what the file-manager actions run). Also handles Finder's "Open With" style bare paths.

use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Launch {
    Plain,
    Open(Vec<PathBuf>),
    Quick { recipe: String, paths: Vec<PathBuf> },
}

/// `cwd` is the folder the launch came from. File managers that run scripts (Files, Caja) pass
/// names relative to it, and so does anyone in a terminal. A launch handed on to the copy
/// already running has to use the sender's folder, not that copy's.
pub fn parse(args: &[String], cwd: &Path) -> Launch {
    let path = |a: &str| if Path::new(a).is_absolute() { PathBuf::from(a) } else { cwd.join(a) };
    let mut it = args.iter().skip(1).filter(|a| !a.starts_with("-psn_") && *a != "--hidden");
    match it.next().map(String::as_str) {
        None => Launch::Plain,
        Some("open") => Launch::Open(it.map(|a| path(a)).collect()),
        Some("quick") => {
            let mut recipe = String::from("just-compress");
            let mut paths = Vec::new();
            while let Some(a) = it.next() {
                if a == "--recipe" {
                    if let Some(r) = it.next() {
                        recipe = r.clone();
                    }
                } else {
                    paths.push(path(a));
                }
            }
            Launch::Quick { recipe, paths }
        }
        Some(first) => Launch::Open(std::iter::once(first).chain(it.map(String::as_str)).map(path).collect()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(a: &[&str]) -> Vec<String> {
        std::iter::once("wrangler").chain(a.iter().copied()).map(String::from).collect()
    }

    // An absolute path on any system, as Windows wants a drive letter.
    fn abs(name: &str) -> String {
        std::env::temp_dir().join(name).to_string_lossy().into_owned()
    }

    #[test]
    fn parses_launch_modes() {
        let cwd = std::env::temp_dir().join("from-here");
        let (a, b) = (abs("a.jpg"), abs("b"));
        let (a, b) = (a.as_str(), b.as_str());
        assert_eq!(parse(&args(&[]), &cwd), Launch::Plain);
        assert_eq!(parse(&args(&["open", a]), &cwd), Launch::Open(vec![a.into()]));
        assert_eq!(parse(&args(&["quick", "--recipe", "web-ready", a, b]), &cwd), Launch::Quick { recipe: "web-ready".into(), paths: vec![a.into(), b.into()] });
        assert_eq!(parse(&args(&[a]), &cwd), Launch::Open(vec![a.into()]));
        assert_eq!(parse(&args(&["-psn_0_123"]), &cwd), Launch::Plain);
        assert_eq!(parse(&args(&["--hidden"]), &cwd), Launch::Plain);
    }

    /// Files and Caja run their scripts in the folder with bare file names.
    #[test]
    fn names_are_taken_from_the_folder_the_launch_came_from() {
        let cwd = std::env::temp_dir().join("shoot");
        assert_eq!(parse(&args(&["quick", "--recipe", "web-ready", "two words.jpg"]), &cwd), Launch::Quick { recipe: "web-ready".into(), paths: vec![cwd.join("two words.jpg")] });
        assert_eq!(parse(&args(&["open", "a.jpg"]), &cwd), Launch::Open(vec![cwd.join("a.jpg")]));
    }
}
