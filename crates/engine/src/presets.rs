// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Sean Hawkey

//! The recipes someone has: the starter set that comes with Wrangler and the ones they saved,
//! in the order they keep them, each switched on or off.
//!
//! A recipe is a `.toml` file in the recipes folder. The starters are built in, and saving a
//! change to one writes a file of the same name over it. `shelf.json` beside the files holds
//! what isn't in them: the order, which are off, and which starters were deleted.

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::naming::slug;
use crate::recipe::Recipe;

/// The starter recipes: id, file, and whether it starts switched on. The everyday ones are on.
/// The ones for a particular shop or site are there to switch on.
const STARTERS: &[(&str, &str, bool)] = &[
    ("compress-for-web", include_str!("../presets/compress-for-web.toml"), true),
    ("just-compress", include_str!("../presets/just-compress.toml"), true),
    ("convert-to-jpeg", include_str!("../presets/convert-to-jpeg.toml"), true),
    ("1080p-16-9", include_str!("../presets/1080p-16-9.toml"), true),
    ("square-1080", include_str!("../presets/square-1080.toml"), true),
    ("small-for-email", include_str!("../presets/small-for-email.toml"), true),
    ("social-card", include_str!("../presets/social-card.toml"), false),
    ("shopify-product", include_str!("../presets/shopify-product.toml"), false),
    ("amazon-main-image", include_str!("../presets/amazon-main-image.toml"), false),
];

/// The recipe a fresh copy of Wrangler starts on.
pub const FIRST: &str = "compress-for-web";

#[derive(Debug, Clone, Serialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub struct RecipeEntry {
    pub id: String,
    pub name: String,
    pub description: String,
    /// One of the starter recipes that come with Wrangler.
    pub built_in: bool,
    /// A starter that has been changed and saved, so it can be put back as it came.
    pub changed: bool,
    /// Offered in the window, the right-click menus and the menu bar menu.
    pub on: bool,
    /// Every setting, so a list can say what the recipe does.
    pub recipe: Recipe,
}

/// What the recipe files don't hold.
#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(default)]
struct Shelf {
    /// Ids in the order they're kept. Any not listed come after, starters first.
    order: Vec<String>,
    /// Ids switched on or off by hand. Anything else follows its default.
    on: Vec<String>,
    off: Vec<String>,
    /// Starters that were deleted.
    removed: Vec<String>,
}

fn shelf_path(dir: &Path) -> PathBuf {
    dir.join("shelf.json")
}

fn shelf(dir: &Path) -> Shelf {
    fs::read(shelf_path(dir)).ok().and_then(|b| serde_json::from_slice(&b).ok()).unwrap_or_default()
}

fn keep(dir: &Path, shelf: &Shelf) -> Result<()> {
    fs::create_dir_all(dir).map_err(|e| Error::io(dir, e))?;
    let path = shelf_path(dir);
    fs::write(&path, serde_json::to_vec_pretty(shelf).expect("a shelf is plain data")).map_err(|e| Error::io(&path, e))
}

fn file(dir: &Path, id: &str) -> PathBuf {
    dir.join(format!("{id}.toml"))
}

fn starter(id: &str) -> Option<(Recipe, bool)> {
    STARTERS.iter().find(|s| s.0 == id).map(|s| (Recipe::from_toml(s.1).expect("starter recipes are valid"), s.2))
}

pub fn built_in() -> Vec<(String, Recipe)> {
    STARTERS.iter().map(|(id, text, _)| ((*id).to_string(), Recipe::from_toml(text).expect("starter recipes are valid"))).collect()
}

/// The recipes saved in the folder, by name.
fn saved(dir: &Path) -> Vec<(String, Recipe)> {
    let mut out: Vec<(String, Recipe)> = fs::read_dir(dir)
        .map(|d| {
            d.flatten()
                .filter(|e| e.path().extension().is_some_and(|x| x == "toml"))
                .filter_map(|e| {
                    let id = e.path().file_stem()?.to_str()?.to_string();
                    Some((id, Recipe::from_toml(&fs::read_to_string(e.path()).ok()?).ok()?))
                })
                .collect()
        })
        .unwrap_or_default();
    out.sort_by(|a, b| natord::compare_ignore_case(&a.1.name, &b.1.name));
    out
}

/// Every recipe, on or off, in the order they're kept.
pub fn list(dir: &Path) -> Vec<RecipeEntry> {
    let shelf = shelf(dir);
    let mut saved = saved(dir);
    let mut out = Vec::new();
    for (id, text, on) in STARTERS {
        let over = saved.iter().position(|s| s.0 == *id).map(|i| saved.remove(i).1);
        if over.is_none() && shelf.removed.iter().any(|r| r == id) {
            continue;
        }
        let changed = over.is_some();
        let recipe = over.unwrap_or_else(|| Recipe::from_toml(text).expect("starter recipes are valid"));
        out.push(RecipeEntry { id: (*id).to_string(), name: recipe.name.clone(), description: recipe.description.clone(), built_in: true, changed, on: *on, recipe });
    }
    out.extend(saved.into_iter().map(|(id, recipe)| RecipeEntry { id, name: recipe.name.clone(), description: recipe.description.clone(), built_in: false, changed: false, on: true, recipe }));
    for e in &mut out {
        if shelf.on.contains(&e.id) {
            e.on = true;
        } else if shelf.off.contains(&e.id) {
            e.on = false;
        }
    }
    let place = |id: &str| shelf.order.iter().position(|o| o == id).unwrap_or(usize::MAX);
    out.sort_by_key(|e| place(&e.id));
    out
}

/// The recipes that are switched on, in order: what the menus offer.
pub fn enabled(dir: &Path) -> Vec<RecipeEntry> {
    list(dir).into_iter().filter(|e| e.on).collect()
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
    let user = file(dir, &slug(key));
    if user.exists() {
        return Recipe::from_toml(&fs::read_to_string(&user).map_err(|e| Error::io(&user, e))?);
    }
    built_in()
        .into_iter()
        .find(|(id, r)| *id == slug(key) || r.name.eq_ignore_ascii_case(key))
        .map(|(_, r)| r)
        .ok_or_else(|| Error::Recipe(format!("no recipe called \"{key}\"")))
}

/// Save under the recipe's name. Returns its id.
pub fn save(dir: &Path, recipe: &Recipe) -> Result<String> {
    recipe.validate()?;
    fs::create_dir_all(dir).map_err(|e| Error::io(dir, e))?;
    let id = slug(&recipe.name);
    let id = if id.is_empty() { "recipe".to_string() } else { id };
    let path = file(dir, &id);
    fs::write(&path, recipe.to_toml()?).map_err(|e| Error::io(&path, e))?;
    Ok(id)
}

/// Delete a recipe. A starter is remembered as removed, so it stays gone (see `restore`).
pub fn delete(dir: &Path, id: &str) -> Result<()> {
    let id = slug(id);
    let path = file(dir, &id);
    let had_file = path.exists();
    if had_file {
        fs::remove_file(&path).map_err(|e| Error::io(&path, e))?;
    }
    let mut shelf = shelf(dir);
    if starter(&id).is_some() {
        shelf.removed.push(id.clone());
    } else if !had_file {
        return Err(Error::Recipe(format!("no recipe called \"{id}\"")));
    }
    for list in [&mut shelf.order, &mut shelf.on, &mut shelf.off] {
        list.retain(|x| *x != id);
    }
    keep(dir, &shelf)
}

/// Put a changed starter back as it came.
pub fn reset(dir: &Path, id: &str) -> Result<()> {
    let path = file(dir, id);
    if starter(id).is_some() && path.exists() {
        fs::remove_file(&path).map_err(|e| Error::io(&path, e))?;
    }
    Ok(())
}

/// Bring back every starter that was deleted. Changed ones stay as they are.
pub fn restore(dir: &Path) -> Result<()> {
    let mut shelf = shelf(dir);
    shelf.removed.clear();
    keep(dir, &shelf)
}

/// A name no other recipe has: the name itself, or with a number after it.
fn free_name(dir: &Path, want: &str, but: Option<&str>) -> String {
    let taken: Vec<String> = list(dir).into_iter().filter(|e| Some(e.id.as_str()) != but).map(|e| e.id).collect();
    (1..).map(|n| if n == 1 { want.to_string() } else { format!("{want} {n}") }).find(|name| !taken.contains(&slug(name))).expect("some number is free")
}

/// Give `new` the place and the on or off of `old`. `beside` keeps `old` and puts `new` after it.
fn take_place(dir: &Path, old: &str, new: &str, beside: bool) -> Result<()> {
    let entries = list(dir);
    let on = entries.iter().find(|e| e.id == old).is_none_or(|e| e.on);
    let mut order: Vec<String> = entries.into_iter().map(|e| e.id).filter(|id| id != new).collect();
    match order.iter().position(|id| id == old) {
        Some(i) if beside => order.insert(i + 1, new.to_string()),
        Some(i) => order[i] = new.to_string(),
        None => order.push(new.to_string()),
    }
    let mut shelf = shelf(dir);
    shelf.order = order;
    shelf.on.retain(|x| x != new);
    shelf.off.retain(|x| x != new);
    if !on {
        shelf.off.push(new.to_string());
    }
    keep(dir, &shelf)
}

/// Rename a recipe, keeping its place. Returns its new id.
pub fn rename(dir: &Path, id: &str, name: &str) -> Result<String> {
    let name = name.trim();
    if slug(name).is_empty() {
        return Err(Error::Recipe("a recipe needs a name".into()));
    }
    let mut recipe = load(dir, id)?;
    recipe.name = free_name(dir, name, Some(id));
    let new = save(dir, &recipe)?;
    if new != id {
        take_place(dir, id, &new, true)?;
        delete(dir, id)?;
    }
    Ok(new)
}

/// A copy of a recipe, placed after it. Returns the copy's id.
pub fn duplicate(dir: &Path, id: &str) -> Result<String> {
    let mut recipe = load(dir, id)?;
    recipe.name = free_name(dir, &format!("{} copy", recipe.name), None);
    let new = save(dir, &recipe)?;
    take_place(dir, id, &new, true)?;
    Ok(new)
}

/// Save a recipe under a new name without touching the one it came from, and place it after
/// that one. Returns the new id.
pub fn save_as(dir: &Path, recipe: &Recipe, from: &str) -> Result<String> {
    let mut recipe = recipe.clone();
    recipe.name = free_name(dir, recipe.name.trim(), None);
    let new = save(dir, &recipe)?;
    take_place(dir, from, &new, true)?;
    let mut shelf = shelf(dir);
    shelf.off.retain(|x| *x != new);
    keep(dir, &shelf)?;
    Ok(new)
}

/// Switch a recipe on or off.
pub fn set_on(dir: &Path, id: &str, on: bool) -> Result<()> {
    let mut shelf = shelf(dir);
    shelf.on.retain(|x| x != id);
    shelf.off.retain(|x| x != id);
    if on { &mut shelf.on } else { &mut shelf.off }.push(id.to_string());
    keep(dir, &shelf)
}

/// Keep the recipes in this order.
pub fn set_order(dir: &Path, ids: &[String]) -> Result<()> {
    let mut shelf = shelf(dir);
    shelf.order = ids.to_vec();
    keep(dir, &shelf)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ids(dir: &Path) -> Vec<String> {
        list(dir).into_iter().map(|e| e.id).collect()
    }

    #[test]
    fn starters_are_valid_and_the_everyday_ones_are_on() {
        let d = tempfile::tempdir().unwrap();
        let all = list(d.path());
        assert_eq!(all.len(), STARTERS.len());
        assert_eq!(all[0].id, FIRST);
        assert!(all.iter().all(|e| e.id == slug(&e.name)), "each starter's id is its name");
        let on: Vec<String> = enabled(d.path()).into_iter().map(|e| e.id).collect();
        assert_eq!(on, ["compress-for-web", "just-compress", "convert-to-jpeg", "1080p-16-9", "square-1080", "small-for-email"]);
        assert_eq!(load(d.path(), "Shopify product").unwrap().name, "Shopify product");
    }

    #[test]
    fn a_changed_starter_saves_over_itself_and_can_be_put_back() {
        let d = tempfile::tempdir().unwrap();
        let mut r = load(d.path(), "compress-for-web").unwrap();
        r.size.long_side = Some(1600);
        assert_eq!(save(d.path(), &r).unwrap(), "compress-for-web");
        let e = list(d.path()).into_iter().find(|e| e.id == "compress-for-web").unwrap();
        assert!(e.built_in && e.changed && e.recipe.size.long_side == Some(1600));
        assert_eq!(list(d.path()).len(), STARTERS.len(), "still one of it");
        reset(d.path(), "compress-for-web").unwrap();
        let e = list(d.path()).into_iter().find(|e| e.id == "compress-for-web").unwrap();
        assert!(!e.changed && e.recipe.size.long_side == Some(2560));
    }

    #[test]
    fn any_recipe_can_be_deleted_and_starters_come_back_when_asked() {
        let d = tempfile::tempdir().unwrap();
        delete(d.path(), "just-compress").unwrap();
        assert!(!ids(d.path()).contains(&"just-compress".to_string()));
        let mine = Recipe { name: "Client proofs".into(), ..Recipe::default() };
        let id = save(d.path(), &mine).unwrap();
        assert_eq!(ids(d.path()).last().unwrap(), "client-proofs");
        delete(d.path(), &id).unwrap();
        assert!(!ids(d.path()).contains(&id));
        assert!(delete(d.path(), "never-was").is_err());
        restore(d.path()).unwrap();
        assert!(ids(d.path()).contains(&"just-compress".to_string()));
    }

    #[test]
    fn renaming_keeps_the_place_and_copies_sit_after_the_original() {
        let d = tempfile::tempdir().unwrap();
        let new = rename(d.path(), "just-compress", "Squash").unwrap();
        assert_eq!(new, "squash");
        assert_eq!(ids(d.path())[1], "squash", "where Just compress was");
        assert!(!ids(d.path()).contains(&"just-compress".to_string()));
        let copy = duplicate(d.path(), "squash").unwrap();
        assert_eq!(copy, "squash-copy");
        assert_eq!(ids(d.path())[2], "squash-copy");
        assert_eq!(duplicate(d.path(), "squash").unwrap(), "squash-copy-2", "a second copy gets a number");
        assert!(rename(d.path(), "squash", "  ").is_err());
        // A name another recipe has gets a number rather than replacing it.
        assert_eq!(rename(d.path(), "squash-copy", "Squash").unwrap(), "squash-2");
    }

    #[test]
    fn on_off_and_order_are_kept() {
        let d = tempfile::tempdir().unwrap();
        set_on(d.path(), "compress-for-web", false).unwrap();
        set_on(d.path(), "social-card", true).unwrap();
        let on: Vec<String> = enabled(d.path()).into_iter().map(|e| e.id).collect();
        assert!(!on.contains(&"compress-for-web".to_string()) && on.contains(&"social-card".to_string()));
        set_order(d.path(), &["social-card".to_string(), "just-compress".to_string()]).unwrap();
        assert_eq!(ids(d.path())[..3], ["social-card", "just-compress", "compress-for-web"], "the rest follow in their usual order");
    }

    /// The starter recipes as JSON, for the browser preview's stand-in engine. Written along
    /// with the TypeScript types: `cargo test -p wrangler-engine --features ts export_bindings`.
    #[cfg(feature = "ts")]
    #[test]
    fn export_bindings_dev_recipes() {
        let d = tempfile::tempdir().unwrap();
        let json = serde_json::json!({ "new": Recipe::default(), "entries": list(d.path()) });
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../apps/desktop/dev/recipes.json");
        fs::write(path, serde_json::to_string_pretty(&json).unwrap() + "\n").unwrap();
    }

    #[test]
    fn saving_as_new_leaves_the_original_alone() {
        let d = tempfile::tempdir().unwrap();
        let mut r = load(d.path(), "compress-for-web").unwrap();
        r.size.long_side = Some(1200);
        r.name = "Blog images".into();
        let id = save_as(d.path(), &r, "compress-for-web").unwrap();
        assert_eq!(id, "blog-images");
        assert_eq!(ids(d.path())[..2], ["compress-for-web", "blog-images"]);
        assert_eq!(load(d.path(), "compress-for-web").unwrap().size.long_side, Some(2560));
        r.name = "Compress for web".into();
        assert_eq!(save_as(d.path(), &r, "compress-for-web").unwrap(), "compress-for-web-2", "never over another recipe");
    }
}
