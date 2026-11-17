// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Sean Hawkey

//! Recipes: everything a batch does, as plain TOML shared by the app, the CLI and quick actions.
//! Every section has defaults, so a recipe only states what it changes, and a recipe with
//! nothing in it copies files unchanged.

use serde::{Deserialize, Serialize};

use crate::codec::metadata::KeepMeta;
use crate::codec::Format;
use crate::colour::Rgba;
use crate::error::{Error, Result};
use crate::modules::{AddOn, Size};

/// Bumped when a change would make older apps misread a recipe.
pub const RECIPE_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[serde(default)]
pub struct Recipe {
    pub version: u32,
    pub name: String,
    pub description: String,
    pub select: Select,
    pub size: Size,
    pub format: FormatChoice,
    pub quality: Quality,
    pub rename: Rename,
    pub metadata: Metadata,
    pub addons: Vec<AddOn>,
    pub destination: Destination,
}

impl Default for Recipe {
    fn default() -> Self {
        Self {
            version: RECIPE_VERSION,
            name: "Untitled".into(),
            description: String::new(),
            select: Select::default(),
            size: Size::default(),
            format: FormatChoice::default(),
            quality: Quality::default(),
            rename: Rename::default(),
            metadata: Metadata::default(),
            addons: Vec::new(),
            destination: Destination::default(),
        }
    }
}

impl Recipe {
    pub fn from_toml(text: &str) -> Result<Self> {
        let r: Recipe = toml::from_str(text).map_err(|e| Error::Recipe(e.message().to_string()))?;
        r.validate()?;
        Ok(r)
    }

    pub fn to_toml(&self) -> Result<String> {
        toml::to_string_pretty(self).map_err(|e| Error::Recipe(e.to_string()))
    }

    pub fn validate(&self) -> Result<()> {
        let bad = |m: String| Err(Error::Recipe(m));
        if self.version > RECIPE_VERSION {
            return bad(format!("made by a newer Wrangler (recipe format {}); update to use it", self.version));
        }
        self.size.validate().or_else(bad)?;
        if let Look::Exact(q) = self.quality.look
            && !(1..=100).contains(&q) {
                return bad("an exact quality must be between 1 and 100".into());
            }
        if self.quality.max_kb == Some(0) {
            return bad("the maximum file size must be above 0 KB".into());
        }
        if self.rename.mode == RenameMode::Pattern && self.rename.pattern.trim().is_empty() {
            return bad("the name pattern is empty".into());
        }
        if self.destination.mode == DestinationMode::Folder && self.destination.folder.as_deref().is_none_or(|f| f.trim().is_empty()) {
            return bad("choose a folder to save into".into());
        }
        if let FormatChoice { id: OutputFormat::To(f), .. } = self.format
            && !f.can_encode() {
                return bad(format!("Wrangler can read {} but can't write it yet", f.name()));
            }
        Ok(())
    }

    /// True when files are copied byte for byte: nothing changes pixels, format or metadata.
    pub fn is_passthrough(&self) -> bool {
        crate::modules::stages(&self.size, &self.addons).is_empty() && self.format.id == OutputFormat::Same && self.quality.look == Look::Untouched
    }
}

/// Which files in the selection the recipe touches.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[serde(default)]
pub struct Select {
    /// Glob patterns on file names, e.g. `*.jpg` or `IMG_*`. Empty means every image.
    pub include: Vec<String>,
    pub exclude: Vec<String>,
    /// Skip files smaller than this.
    pub min_kb: Option<u64>,
    /// Look inside folders within folders.
    pub recursive: bool,
}

/// The output format: keep each file's own, or convert everything to one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OutputFormat {
    #[default]
    Same,
    To(Format),
}

impl OutputFormat {
    /// The concrete format to write for a source.
    pub fn resolve(self, source: Format) -> Format {
        match self {
            OutputFormat::Same => source.writable_equivalent(),
            OutputFormat::To(f) => f,
        }
    }
}

impl Serialize for OutputFormat {
    fn serialize<S: serde::Serializer>(&self, s: S) -> std::result::Result<S::Ok, S::Error> {
        match self {
            OutputFormat::Same => s.serialize_str("same"),
            OutputFormat::To(f) => s.serialize_str(f.extension()),
        }
    }
}

impl<'de> Deserialize<'de> for OutputFormat {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> std::result::Result<Self, D::Error> {
        let t = String::deserialize(d)?.trim().to_ascii_lowercase();
        if t == "same" || t == "keep" {
            return Ok(OutputFormat::Same);
        }
        Format::from_extension(&t).map(OutputFormat::To).ok_or_else(|| serde::de::Error::custom(format!("unknown format: {t}")))
    }
}

#[cfg(feature = "ts")]
impl ts_rs::TS for OutputFormat {
    type WithoutGenerics = Self;
    type OptionInnerType = Self;
    fn name(_: &ts_rs::Config) -> String {
        "string".into()
    }
    fn inline(_: &ts_rs::Config) -> String {
        "string".into()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[serde(default)]
pub struct FormatChoice {
    #[cfg_attr(feature = "ts", ts(type = "string"))]
    pub id: OutputFormat,
    /// Pick per image: WebP for photos, PNG for flat graphics.
    pub auto: bool,
    /// Colour behind transparent areas in formats without transparency.
    #[cfg_attr(feature = "ts", ts(type = "string"))]
    pub matte: Rgba,
}

impl Default for FormatChoice {
    fn default() -> Self {
        Self { id: OutputFormat::Same, auto: false, matte: Rgba::WHITE }
    }
}

/// How hard to compress. Looks aim at a measured visual score, so the same look gives the
/// same result across formats and images.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Look {
    /// Don't re-encode when nothing else changes.
    Untouched,
    /// No quality loss where the format allows it.
    Lossless,
    Pristine,
    High,
    #[default]
    Balanced,
    Small,
    /// A raw encoder quality, 1–100.
    Exact(u8),
}

impl Look {
    /// SSIMULACRA 2 score the search aims for (higher is closer to the original).
    pub fn target_score(self) -> Option<f64> {
        match self {
            Look::Pristine => Some(90.0),
            Look::High => Some(83.0),
            Look::Balanced => Some(75.0),
            Look::Small => Some(65.0),
            _ => None,
        }
    }

    pub fn label(self) -> String {
        match self {
            Look::Untouched => "untouched".into(),
            Look::Lossless => "lossless".into(),
            Look::Pristine => "pristine".into(),
            Look::High => "high".into(),
            Look::Balanced => "balanced".into(),
            Look::Small => "small".into(),
            Look::Exact(q) => format!("quality {q}"),
        }
    }
}

impl Serialize for Look {
    fn serialize<S: serde::Serializer>(&self, s: S) -> std::result::Result<S::Ok, S::Error> {
        match self {
            Look::Exact(q) => s.serialize_u8(*q),
            other => s.serialize_str(&other.label()),
        }
    }
}

impl<'de> Deserialize<'de> for Look {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> std::result::Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Raw {
            N(u8),
            T(String),
        }
        Ok(match Raw::deserialize(d)? {
            Raw::N(q) => Look::Exact(q),
            Raw::T(t) => match t.trim().to_ascii_lowercase().as_str() {
                "untouched" | "original" => Look::Untouched,
                "lossless" => Look::Lossless,
                "pristine" => Look::Pristine,
                "high" => Look::High,
                "balanced" | "auto" => Look::Balanced,
                "small" => Look::Small,
                other => other.parse::<u8>().map(Look::Exact).map_err(|_| serde::de::Error::custom(format!("unknown look: {other}")))?,
            },
        })
    }
}

#[cfg(feature = "ts")]
impl ts_rs::TS for Look {
    type WithoutGenerics = Self;
    type OptionInnerType = Self;
    fn name(_: &ts_rs::Config) -> String {
        "string | number".into()
    }
    fn inline(_: &ts_rs::Config) -> String {
        "string | number".into()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[serde(default)]
pub struct Quality {
    #[cfg_attr(feature = "ts", ts(type = "string | number"))]
    pub look: Look,
    /// Hard cap on file size; quality drops until each file fits.
    pub max_kb: Option<u32>,
    /// When the cap can't be met at the lowest acceptable quality, shrink the image as well.
    pub allow_downscale: bool,
    /// Keep the original when a same-format, same-size result wouldn't be smaller.
    pub never_bigger: bool,
    pub progressive: bool,
}

impl Default for Quality {
    fn default() -> Self {
        Self { look: Look::Balanced, max_kb: None, allow_downscale: false, never_bigger: true, progressive: true }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[serde(rename_all = "snake_case")]
pub enum RenameMode {
    #[default]
    Keep,
    Pattern,
    Replace,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[serde(rename_all = "snake_case")]
pub enum Case {
    #[default]
    Keep,
    Lower,
    Upper,
    Title,
    /// lowercase-words-joined-by-hyphens, safe for URLs.
    Slug,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[serde(default)]
pub struct Replacement {
    pub find: String,
    pub with: String,
    pub regex: bool,
    pub ignore_case: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[serde(default)]
pub struct Rename {
    pub mode: RenameMode,
    /// Tokens: {name} {n} {n:000} {w} {h} {date} {taken} {folder} {ext}.
    /// Filters: {name|slug} {name|lower} {name|upper} {name|title}.
    pub pattern: String,
    pub case: Case,
    /// Applied to the original name in `replace` mode, and before the pattern in `pattern` mode.
    pub replace: Vec<Replacement>,
    pub start: u32,
    pub step: u32,
}

impl Default for Rename {
    fn default() -> Self {
        Self { mode: RenameMode::Keep, pattern: "{name}".into(), case: Case::Keep, replace: Vec::new(), start: 1, step: 1 }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[serde(default)]
pub struct Metadata {
    pub keep: KeepMeta,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[serde(rename_all = "snake_case")]
pub enum DestinationMode {
    /// New files next to the originals, with a suffix.
    Beside,
    /// A folder created next to each original.
    #[default]
    Subfolder,
    /// One folder, mirroring the dropped folder structure.
    Folder,
    /// Write over the originals; each original goes to the undo store first.
    Replace,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[serde(rename_all = "snake_case")]
pub enum OnConflict {
    /// Add -2, -3 and so on.
    #[default]
    Number,
    Skip,
    Overwrite,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[serde(default)]
pub struct Destination {
    pub mode: DestinationMode,
    /// Used by `beside`.
    pub suffix: String,
    /// Folder name for `subfolder`; path for `folder`.
    pub folder: Option<String>,
    pub on_conflict: OnConflict,
    /// Give new files the original's modified time.
    pub keep_dates: bool,
}

impl Default for Destination {
    fn default() -> Self {
        Self { mode: DestinationMode::Subfolder, suffix: "-web".into(), folder: Some("Wrangled".into()), on_conflict: OnConflict::Number, keep_dates: false }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::{Backdrop, Sharpen, Trim};
    use crate::ops::frame::{Fit, Shape};

    #[test]
    fn empty_recipe_changes_nothing() {
        let r = Recipe::from_toml("").unwrap();
        assert_eq!(r, Recipe::default());
        assert!(!crate::modules::stages(&r.size, &r.addons).iter().any(|s| s.active()));
    }

    #[test]
    fn reads_the_documented_example() {
        let text = r##"
            name = "Shopify product"
            [size]
            long_side = 2048
            shape = "1:1"
            fit = "pad"
            trim = { tolerance = 4 }
            pad = { backdrop = "auto", fill = 85 }
            [format]
            id = "webp"
            [quality]
            look = "balanced"
            max_kb = 400
            [rename]
            mode = "pattern"
            pattern = "spring-24-{n:000}"
            [metadata]
            keep = "safe"
            [[addons]]
            kind = "sharpen"
            amount = 40
            [destination]
            mode = "replace"
        "##;
        let r = Recipe::from_toml(text).unwrap();
        assert_eq!(r.size.shape, Shape::Ratio(1, 1));
        assert_eq!(r.size.fit, Fit::Pad);
        assert_eq!(r.size.trim, Some(Trim { tolerance: 4, margin: 0 }));
        assert_eq!(r.size.pad.backdrop, Backdrop::Auto);
        assert_eq!(r.format.id, OutputFormat::To(Format::Webp));
        assert_eq!(r.quality.max_kb, Some(400));
        assert_eq!(r.metadata.keep, KeepMeta::Safe);
        assert_eq!(r.addons, vec![AddOn::Sharpen(Sharpen { amount: 40, ..Sharpen::default() })]);
        assert_eq!(r.destination.mode, DestinationMode::Replace);
    }

    #[test]
    fn round_trips_through_toml() {
        let mut r = Recipe { name: "Shop".into(), ..Recipe::default() };
        r.size = Size { long_side: Some(2000), shape: Shape::Exact(1200, 628), fit: Fit::Crop, ..Size::default() };
        r.format.id = OutputFormat::To(Format::Avif);
        r.quality.look = Look::Exact(70);
        r.addons.push(AddOn::Sharpen(Sharpen::default()));
        assert_eq!(Recipe::from_toml(&r.to_toml().unwrap()).unwrap(), r);
    }

    #[test]
    fn explains_bad_recipes() {
        for (text, needle) in [
            ("version = 99", "newer"),
            ("[size]\nshape = \"square\"", "shape"),
            ("[size]\npad = { fill = 150 }", "fill"),
            ("[format]\nid = \"jxl\"", "can't write"),
            ("[destination]\nmode = \"folder\"\nfolder = \"\"", "folder"),
        ] {
            let err = Recipe::from_toml(text).unwrap_err().to_string();
            assert!(err.contains(needle), "{text}: {err}");
        }
    }
}
