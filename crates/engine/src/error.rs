// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Sean Hawkey

use std::path::PathBuf;

pub type Result<T, E = Error> = std::result::Result<T, E>;

/// Errors carry plain-language messages: they are shown to people, not just logged.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{path}: {source}")]
    Io { path: PathBuf, source: std::io::Error },
    #[error("can't read this image: {0}")]
    Decode(String),
    #[error("can't save as {format}: {reason}")]
    Encode { format: &'static str, reason: String },
    #[error("{0} files aren't supported yet")]
    Unsupported(String),
    #[error("recipe problem: {0}")]
    Recipe(String),
    #[error("{0}")]
    Other(String),
    #[error("stopped")]
    Cancelled,
}

impl Error {
    pub fn io(path: impl Into<PathBuf>, source: std::io::Error) -> Self {
        Error::Io { path: path.into(), source }
    }
}

impl From<image::ImageError> for Error {
    fn from(e: image::ImageError) -> Self {
        Error::Decode(e.to_string())
    }
}
