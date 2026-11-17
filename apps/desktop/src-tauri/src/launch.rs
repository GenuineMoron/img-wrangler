// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Sean Hawkey

//! How the app was asked to start: plain, `open <paths…>`, or `quick --recipe <id> <paths…>`
//! (what the file-manager actions run). Also handles Finder's "Open With" style bare paths.

use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Launch {
    Plain,
    Open(Vec<PathBuf>),
    Quick { recipe: String, paths: Vec<PathBuf> },
}

pub fn parse(args: &[String]) -> Launch {
    let mut it = args.iter().skip(1).filter(|a| !a.starts_with("-psn_"));
    match it.next().map(String::as_str) {
        None => Launch::Plain,
        Some("open") => Launch::Open(it.map(PathBuf::from).collect()),
        Some("quick") => {
            let mut recipe = String::from("just-compress");
            let mut paths = Vec::new();
            while let Some(a) = it.next() {
                if a == "--recipe" {
                    if let Some(r) = it.next() {
                        recipe = r.clone();
                    }
                } else {
                    paths.push(PathBuf::from(a));
                }
            }
            Launch::Quick { recipe, paths }
        }
        Some(first) => Launch::Open(std::iter::once(first).chain(it.map(String::as_str)).map(PathBuf::from).collect()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(a: &[&str]) -> Vec<String> {
        std::iter::once("wrangler").chain(a.iter().copied()).map(String::from).collect()
    }

    #[test]
    fn parses_launch_modes() {
        assert_eq!(parse(&args(&[])), Launch::Plain);
        assert_eq!(parse(&args(&["open", "/a.jpg"])), Launch::Open(vec!["/a.jpg".into()]));
        assert_eq!(parse(&args(&["quick", "--recipe", "web-ready", "/a.jpg", "/b"])), Launch::Quick { recipe: "web-ready".into(), paths: vec!["/a.jpg".into(), "/b".into()] });
        assert_eq!(parse(&args(&["/x.png"])), Launch::Open(vec!["/x.png".into()]));
        assert_eq!(parse(&args(&["-psn_0_123"])), Launch::Plain);
    }
}
