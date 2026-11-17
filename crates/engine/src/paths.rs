// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Sean Hawkey

//! Where Wrangler keeps things. `WRANGLER_HOME` puts everything under one folder (portable
//! installs and tests).

use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct AppDirs {
    /// Recipes and settings.
    pub config: PathBuf,
    /// Undo store and run history.
    pub data: PathBuf,
    /// Thumbnails; safe to delete.
    pub cache: PathBuf,
}

impl AppDirs {
    pub fn detect() -> Self {
        if let Some(home) = std::env::var_os("WRANGLER_HOME") {
            let home = PathBuf::from(home);
            return Self { config: home.join("config"), data: home.join("data"), cache: home.join("cache") };
        }
        let base = |d: Option<PathBuf>| d.unwrap_or_else(std::env::temp_dir).join("img-wrangler");
        Self { config: base(dirs::config_dir()), data: base(dirs::data_dir()), cache: base(dirs::cache_dir()) }
    }

    pub fn recipes(&self) -> PathBuf {
        self.config.join("recipes")
    }
    pub fn runs(&self) -> PathBuf {
        self.data.join("runs")
    }
    pub fn thumbs(&self) -> PathBuf {
        self.cache.join("thumbs")
    }
}
