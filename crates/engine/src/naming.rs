// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Sean Hawkey

//! Output names: patterns with tokens, find and replace, case, and names that are safe on
//! every operating system.

use regex::RegexBuilder;

use crate::img::Dims;
use crate::recipe::{Case, Rename, RenameMode, Replacement};

/// What a pattern can refer to for one file.
#[derive(Debug, Clone)]
pub struct NameCx<'a> {
    /// The original file name without its extension.
    pub stem: &'a str,
    /// Position in the batch, from 0.
    pub index: u32,
    pub dims: Dims,
    /// Name of the folder the file is in.
    pub folder: &'a str,
    /// File modified date, `YYYY-MM-DD`.
    pub date: &'a str,
    /// Date the photo was taken (EXIF), `YYYY-MM-DD`, when known.
    pub taken: Option<&'a str>,
    /// Extension of the output file, without the dot.
    pub ext: &'a str,
}

/// The output name without extension.
pub fn render(r: &Rename, cx: &NameCx) -> String {
    let base = apply_replacements(cx.stem, &r.replace);
    let name = match r.mode {
        RenameMode::Keep => cx.stem.to_string(),
        RenameMode::Replace => base,
        RenameMode::Pattern => expand(&r.pattern, &base, r, cx),
    };
    sanitise(&apply_case(&name, r.case))
}

fn apply_replacements(stem: &str, list: &[Replacement]) -> String {
    list.iter().filter(|x| !x.find.is_empty()).fold(stem.to_string(), |acc, x| {
        let pattern = if x.regex { x.find.clone() } else { regex::escape(&x.find) };
        match RegexBuilder::new(&pattern).case_insensitive(x.ignore_case).build() {
            Ok(re) => re.replace_all(&acc, x.with.as_str()).into_owned(),
            Err(_) => acc,
        }
    })
}

fn expand(pattern: &str, base: &str, r: &Rename, cx: &NameCx) -> String {
    let mut out = String::with_capacity(pattern.len() + 16);
    let mut rest = pattern;
    while let Some(open) = rest.find('{') {
        out.push_str(&rest[..open]);
        let Some(close) = rest[open..].find('}') else {
            out.push_str(&rest[open..]);
            return out;
        };
        let token = &rest[open + 1..open + close];
        out.push_str(&token_value(token, base, r, cx).unwrap_or_else(|| format!("{{{token}}}")));
        rest = &rest[open + close + 1..];
    }
    out.push_str(rest);
    out
}

fn token_value(token: &str, base: &str, r: &Rename, cx: &NameCx) -> Option<String> {
    let (head, filter) = token.split_once('|').map_or((token, None), |(h, f)| (h, Some(f)));
    let (key, pad) = head.split_once(':').map_or((head, None), |(k, p)| (k, Some(p)));
    let value = match key.trim() {
        "name" => base.to_string(),
        "n" => {
            let n = r.start + cx.index * r.step.max(1);
            let width = pad.map(|p| p.len()).unwrap_or(0);
            format!("{n:0width$}")
        }
        "w" => cx.dims.w.to_string(),
        "h" => cx.dims.h.to_string(),
        "date" => cx.date.to_string(),
        "taken" => cx.taken.unwrap_or(cx.date).to_string(),
        "folder" => cx.folder.to_string(),
        "ext" => cx.ext.to_string(),
        _ => return None,
    };
    Some(match filter.map(str::trim) {
        Some("slug") => slug(&value),
        Some("lower") => value.to_lowercase(),
        Some("upper") => value.to_uppercase(),
        Some("title") => title(&value),
        _ => value,
    })
}

fn apply_case(s: &str, case: Case) -> String {
    match case {
        Case::Keep => s.to_string(),
        Case::Lower => s.to_lowercase(),
        Case::Upper => s.to_uppercase(),
        Case::Title => title(s),
        Case::Slug => slug(s),
    }
}

/// `Blue Mug (Large)` → `blue-mug-large`. Accents are transliterated, not dropped.
pub fn slug(s: &str) -> String {
    let ascii = deunicode::deunicode(s).to_lowercase();
    let mut out = String::with_capacity(ascii.len());
    for c in ascii.chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c);
        } else if !out.ends_with('-') {
            out.push('-');
        }
    }
    out.trim_matches('-').to_string()
}

pub fn title(s: &str) -> String {
    s.split_inclusive([' ', '-', '_'])
        .map(|w| {
            let mut c = w.chars();
            c.next().map_or(String::new(), |f| f.to_uppercase().chain(c.flat_map(char::to_lowercase)).collect())
        })
        .collect()
}

/// A name every file system accepts: no path separators or reserved characters, no trailing
/// dots or spaces, and not one of Windows's reserved device names.
pub fn sanitise(name: &str) -> String {
    let mut s: String = name
        .chars()
        .map(|c| if c.is_control() || matches!(c, '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|') { '-' } else { c })
        .collect();
    s = s.trim().trim_end_matches(['.', ' ']).to_string();
    if s.is_empty() {
        s = "image".into();
    }
    let upper = s.to_ascii_uppercase();
    let reserved = ["CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8", "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9"];
    if reserved.contains(&upper.as_str()) {
        s.push('_');
    }
    // Leave room for an extension and a -2 suffix under the common 255-byte limit.
    while s.len() > 240 {
        s.pop();
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cx(stem: &str, index: u32) -> NameCx<'_> {
        NameCx { stem, index, dims: Dims::new(2048, 1365), folder: "spring-catalogue", date: "2026-11-17", taken: Some("2026-10-02"), ext: "webp" }
    }

    fn pattern(p: &str) -> Rename {
        Rename { mode: RenameMode::Pattern, pattern: p.into(), ..Rename::default() }
    }

    #[test]
    fn numbers_with_padding() {
        let r = pattern("spring-24-{n:000}");
        assert_eq!(render(&r, &cx("DSC_4471", 0)), "spring-24-001");
        assert_eq!(render(&r, &cx("DSC_4471", 1283)), "spring-24-1284");
        let r = Rename { start: 10, step: 10, ..pattern("{n}") };
        assert_eq!(render(&r, &cx("x", 2)), "30");
    }

    #[test]
    fn tokens_and_filters() {
        assert_eq!(render(&pattern("{name|slug}-{w}x{h}"), &cx("Blue Mug (Large)", 0)), "blue-mug-large-2048x1365");
        assert_eq!(render(&pattern("{folder}_{taken}"), &cx("a", 0)), "spring-catalogue_2026-10-02");
        assert_eq!(render(&pattern("{unknown}-{n}"), &cx("a", 0)), "{unknown}-1");
    }

    #[test]
    fn find_and_replace() {
        let r = Rename { mode: RenameMode::Replace, replace: vec![Replacement { find: "DSC_".into(), with: "spring-".into(), ..Default::default() }], ..Rename::default() };
        assert_eq!(render(&r, &cx("DSC_4471", 0)), "spring-4471");
        let re = Rename { mode: RenameMode::Replace, replace: vec![Replacement { find: r"_v\d+$".into(), with: "".into(), regex: true, ..Default::default() }], ..Rename::default() };
        assert_eq!(render(&re, &cx("board_oak_v2", 0)), "board_oak");
    }

    #[test]
    fn keep_mode_ignores_pattern() {
        assert_eq!(render(&Rename { mode: RenameMode::Keep, pattern: "x-{n}".into(), ..Rename::default() }, &cx("DSC_1", 0)), "DSC_1");
    }

    #[test]
    fn case_and_slug() {
        assert_eq!(render(&Rename { case: Case::Slug, ..pattern("{name}") }, &cx("Café Crème_01", 0)), "cafe-creme-01");
        assert_eq!(title("blue mug-large"), "Blue Mug-Large");
    }

    #[test]
    fn names_are_safe_everywhere() {
        assert_eq!(sanitise("a/b:c*?"), "a-b-c--");
        assert_eq!(sanitise("CON"), "CON_");
        assert_eq!(sanitise("name. "), "name");
        assert_eq!(sanitise("   "), "image");
    }
}
