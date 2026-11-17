// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Sean Hawkey

//! Built-in recipes and the user's saved ones.

use std::fs;
use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::error::{Error, Result};
use crate::naming::slug;
use crate::recipe::Recipe;

const BUILT_IN: &[(&str, &str)] = &[
    ("web-ready", include_str!("../presets/web-ready.toml")),
    ("just-compress", include_str!("../presets/just-compress.toml")),
    ("shopify-product", include_str!("../presets/shopify-product.toml")),
    ("amazon-main-image", include_str!("../presets/amazon-main-image.toml")),
    ("email-safe", include_str!("../presets/email-safe.toml")),
    ("social-card", include_str!("../presets/social-card.toml")),
    ("square-thumbnails", include_str!("../presets/square-thumbnails.toml")),
];

#[derive(Debug, Clone, Serialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub struct RecipeEntry {
    pub id: String,
    pub name: String,
    pub description: String,
    pub built_in: bool,
}

pub fn built_in() -> Vec<(String, Recipe)> {
    BUILT_IN.iter().map(|(id, text)| ((*id).to_string(), Recipe::from_toml(text).expect("built-in recipes are valid"))).collect()
}

/// Built-ins first, then the user's recipes. A user recipe with a built-in's id replaces it.
pub fn list(dir: &Path) -> Vec<RecipeEntry> {
    let mut out: Vec<RecipeEntry> = built_in().into_iter().map(|(id, r)| RecipeEntry { id, name: r.name, description: r.description, built_in: true }).collect();
    let mut users: Vec<RecipeEntry> = fs::read_dir(dir)
        .map(|d| {
            d.flatten()
                .filter(|e| e.path().extension().is_some_and(|x| x == "toml"))
                .filter_map(|e| {
                    let id = e.path().file_stem()?.to_str()?.to_string();
                    let r = Recipe::from_toml(&fs::read_to_string(e.path()).ok()?).ok()?;
                    Some(RecipeEntry { id, name: r.name, description: r.description, built_in: false })
                })
                .collect()
        })
        .unwrap_or_default();
    users.sort_by(|a, b| natord::compare_ignore_case(&a.name, &b.name));
    for u in users {
        match out.iter_mut().find(|e| e.id == u.id) {
            Some(existing) => *existing = u,
            None => out.push(u),
        }
    }
    out
}

/// Load by id, by name, or from a path to a `.toml` file. `new` gives the defaults.
pub fn load(dir: &Path, key: &str) -> Result<Recipe> {
    if key == "new" {
        return Ok(Recipe::default());
    }
    let as_path = Path::new(key);
    if as_path.extension().is_some_and(|x| x == "toml") && as_path.exists() {
        return Recipe::from_toml(&fs::read_to_string(as_path).map_err(|e| Error::io(as_path, e))?);
    }
    let user = dir.join(format!("{}.toml", slug(key)));
    if user.exists() {
        return Recipe::from_toml(&fs::read_to_string(&user).map_err(|e| Error::io(&user, e))?);
    }
    built_in()
        .into_iter()
        .find(|(id, r)| *id == slug(key) || r.name.eq_ignore_ascii_case(key))
        .map(|(_, r)| r)
        .ok_or_else(|| Error::Recipe(format!("no recipe called \"{key}\"")))
}

pub fn save(dir: &Path, recipe: &Recipe) -> Result<String> {
    recipe.validate()?;
    fs::create_dir_all(dir).map_err(|e| Error::io(dir, e))?;
    let id = slug(&recipe.name);
    let id = if id.is_empty() { "recipe".to_string() } else { id };
    let path: PathBuf = dir.join(format!("{id}.toml"));
    fs::write(&path, recipe.to_toml()?).map_err(|e| Error::io(&path, e))?;
    Ok(id)
}

pub fn delete(dir: &Path, id: &str) -> Result<()> {
    let path = dir.join(format!("{}.toml", slug(id)));
    fs::remove_file(&path).map_err(|e| Error::io(&path, e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn built_ins_are_valid_and_listed() {
        assert_eq!(built_in().len(), BUILT_IN.len());
        let d = tempfile::tempdir().unwrap();
        assert_eq!(list(d.path()).len(), BUILT_IN.len());
        assert_eq!(load(d.path(), "Shopify product").unwrap().name, "Shopify product");
    }

    #[test]
    fn saves_overrides_and_deletes() {
        let d = tempfile::tempdir().unwrap();
        let mut r = load(d.path(), "web-ready").unwrap();
        r.description = "mine".into();
        let id = save(d.path(), &r).unwrap();
        assert_eq!(id, "web-ready");
        assert_eq!(list(d.path()).iter().find(|e| e.id == "web-ready").unwrap().description, "mine");
        delete(d.path(), &id).unwrap();
        assert!(load(d.path(), "nope").is_err());
    }
}
