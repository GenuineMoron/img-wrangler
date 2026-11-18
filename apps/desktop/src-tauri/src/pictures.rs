// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Sean Hawkey

//! Pictures the windows show: the monitor's previews and the sheet's thumbnails.
//!
//! A preview comes in two steps. The draft (the recipe's framing at screen size) is ready in a
//! fraction of a second; the exact result, which needs the full encode, follows. Both are kept,
//! keyed by file and recipe, so going back to an image is instant, and the app works out the
//! neighbours' previews ahead of time. Work for an image the person has moved away from is
//! cancelled. Thumbnails come from a small pool of threads that serves the newest request first,
//! so the ones just scrolled into view arrive before the ones scrolled past.

use std::collections::{HashMap, VecDeque};
use std::hash::{DefaultHasher, Hash, Hasher};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use parking_lot::{Condvar, Mutex};
use serde::Serialize;
use wrangler_engine::codec::encode::{encoder_for, EncodeOptions, EncodeQuality};
use wrangler_engine::codec::{decode_path, Format};
use wrangler_engine::img::Img;
use wrangler_engine::modules::Cx;
use wrangler_engine::preview::{draft, screen_webp};
use wrangler_engine::process::{process_img, read_source};
use wrangler_engine::{Dims, Recipe};

/// Memory for pictures waiting to be shown.
const MEMORY: usize = 256 * 1024 * 1024;

#[derive(Debug, Clone, Serialize)]
pub struct Preview {
    /// False for the quick draft; the exact result follows.
    pub exact: bool,
    /// Tokens for `preview://localhost/<token>`: the original at screen size, the original at
    /// full size (for actual pixels), and the result (screen size in a draft, the real output
    /// when exact).
    pub before: String,
    pub before_full: String,
    pub after: String,
    pub before_bytes: u64,
    /// Known once exact.
    pub after_bytes: Option<u64>,
    pub source_dims: Dims,
    pub dims: Dims,
    pub format: Format,
    pub quality: Option<u8>,
    pub score: Option<f64>,
    pub kept_original: bool,
    pub over_limit: bool,
    pub downscaled: bool,
}

#[derive(Clone)]
enum Pic {
    Bytes(Arc<Vec<u8>>, &'static str),
    /// Read when asked for: an original at full size.
    File(PathBuf),
}

#[derive(Default)]
struct Store {
    pics: HashMap<String, Pic>,
    order: VecDeque<String>,
    bytes: usize,
    previews: HashMap<String, Preview>,
    /// Screen-sized originals, which don't depend on the recipe.
    befores: HashMap<PathBuf, String>,
    /// Previews being worked out, with the switch that cancels each.
    working: HashMap<String, (PathBuf, u64, Arc<AtomicBool>)>,
}

impl Store {
    fn put(&mut self, pic: Pic) -> String {
        let token = uuid::Uuid::new_v4().simple().to_string();
        if let Pic::Bytes(b, _) = &pic {
            self.bytes += b.len();
        }
        self.pics.insert(token.clone(), pic);
        self.order.push_back(token.clone());
        while self.bytes > MEMORY && self.order.len() > 4 {
            let Some(old) = self.order.pop_front() else { break };
            if let Some(Pic::Bytes(b, _)) = self.pics.remove(&old) {
                self.bytes -= b.len();
            }
            self.previews.retain(|_, p| p.before != old && p.after != old && p.before_full != old);
            self.befores.retain(|_, t| *t != old);
        }
        token
    }
}

#[derive(Default)]
pub struct Pictures {
    store: Mutex<Store>,
    done: Condvar,
}

/// Same recipe, same number: previews are kept per file and recipe.
pub fn recipe_key(recipe: &Recipe) -> u64 {
    let mut h = DefaultHasher::new();
    serde_json::to_string(recipe).unwrap_or_default().hash(&mut h);
    h.finish()
}

impl Pictures {
    /// The preview of `path` under `recipe`: kept from before, or worked out now. A draft asks
    /// for the quick version but gets the exact one when it's already there.
    pub fn preview(&self, path: &Path, recipe: &Recipe, exact: bool) -> Result<Preview, String> {
        let rk = recipe_key(recipe);
        let key = |exact: bool| format!("{}|{rk}|{exact}", path.display());
        let mine = key(exact);
        let cancel = loop {
            let mut s = self.store.lock();
            if let Some(p) = s.previews.get(&key(true)).or_else(|| if exact { None } else { s.previews.get(&mine) }) {
                return Ok(p.clone());
            }
            if !s.working.contains_key(&mine) {
                let flag = Arc::new(AtomicBool::new(false));
                s.working.insert(mine.clone(), (path.to_path_buf(), rk, flag.clone()));
                break flag;
            }
            // Someone is already on it (often the look-ahead): wait for theirs.
            self.done.wait_for(&mut s, Duration::from_millis(40));
        };
        let result = if exact { self.exact(path, recipe, &cancel) } else { self.draft(path, recipe, &cancel) };
        let mut s = self.store.lock();
        s.working.remove(&mine);
        let out = result.inspect(|p| {
            s.previews.insert(mine, p.clone());
        });
        drop(s);
        self.done.notify_all();
        out
    }

    /// Cancel preview work for anything but these files under this recipe.
    pub fn focus(&self, keep: &[PathBuf], recipe: &Recipe) {
        let rk = recipe_key(recipe);
        for (path, r, flag) in self.store.lock().working.values() {
            if *r != rk || !keep.contains(path) {
                flag.store(true, Ordering::Relaxed);
            }
        }
    }

    fn before(&self, path: &Path, img: &Img) -> Result<String, String> {
        if let Some(t) = self.store.lock().befores.get(path) {
            return Ok(t.clone());
        }
        let webp = screen_webp(&img.px).map_err(|e| e.to_string())?;
        let mut s = self.store.lock();
        let t = s.put(Pic::Bytes(Arc::new(webp), "image/webp"));
        s.befores.insert(path.to_path_buf(), t.clone());
        Ok(t)
    }

    fn draft(&self, path: &Path, recipe: &Recipe, cancel: &AtomicBool) -> Result<Preview, String> {
        let d = draft(path, recipe, &mut Cx { cancel: Some(cancel) }).map_err(|e| e.to_string())?;
        let mut s = self.store.lock();
        let before = match s.befores.get(path) {
            Some(t) => t.clone(),
            None => {
                let t = s.put(Pic::Bytes(Arc::new(d.before), "image/webp"));
                s.befores.insert(path.to_path_buf(), t.clone());
                t
            }
        };
        Ok(Preview {
            exact: false,
            before,
            before_full: s.put(Pic::File(path.to_path_buf())),
            after: s.put(Pic::Bytes(Arc::new(d.after), "image/webp")),
            before_bytes: d.before_bytes,
            after_bytes: None,
            source_dims: d.source_dims,
            dims: d.dims,
            format: d.format,
            quality: None,
            score: None,
            kept_original: false,
            over_limit: false,
            downscaled: false,
        })
    }

    fn exact(&self, path: &Path, recipe: &Recipe, cancel: &AtomicBool) -> Result<Preview, String> {
        let (original, img) = read_source(path).map_err(|e| e.to_string())?;
        let source = img.source;
        let source_dims = img.dims();
        let before = self.before(path, &img)?;
        if cancel.load(Ordering::Relaxed) {
            return Err("superseded".into());
        }
        let out = process_img(img, &original, recipe, &mut Cx { cancel: Some(cancel) }).map_err(|e| e.to_string())?;
        let after = if out.kept_original && out.format == source {
            Pic::File(path.to_path_buf())
        } else if out.format.browser_displayable() {
            Pic::Bytes(Arc::new(out.bytes.clone()), out.format.mime())
        } else {
            // TIFF, BMP and the like: show a faithful WebP of the result instead.
            let back = wrangler_engine::codec::decode_bytes(&out.bytes, Some(out.format)).map_err(|e| e.to_string())?;
            let webp = encoder_for(Format::Webp).map_err(|e| e.to_string())?.encode(&back.px, back.has_transparency(), EncodeQuality::Lossy(92), &EncodeOptions { fast: true, ..EncodeOptions::default() }).map_err(|e| e.to_string())?;
            Pic::Bytes(Arc::new(webp), "image/webp")
        };
        let mut s = self.store.lock();
        Ok(Preview {
            exact: true,
            before,
            before_full: s.put(Pic::File(path.to_path_buf())),
            after: s.put(after),
            before_bytes: original.len() as u64,
            after_bytes: Some(out.bytes.len() as u64),
            source_dims,
            dims: out.dims,
            format: out.format,
            quality: out.quality,
            score: out.score,
            kept_original: out.kept_original,
            over_limit: out.over_limit,
            downscaled: out.downscaled,
        })
    }

    /// The bytes and type behind a token. Originals are read now, and converted to WebP when
    /// the webview can't show their format.
    pub fn serve(&self, token: &str) -> Option<(Arc<Vec<u8>>, &'static str)> {
        let pic = self.store.lock().pics.get(token).cloned()?;
        match pic {
            Pic::Bytes(b, mime) => Some((b, mime)),
            Pic::File(path) => {
                let bytes = std::fs::read(&path).ok()?;
                let format = Format::sniff(&bytes[..bytes.len().min(64)]).or_else(|| Format::from_path(&path))?;
                if format.browser_displayable() {
                    return Some((Arc::new(bytes), format.mime()));
                }
                let img = decode_path(&path).ok()?;
                let webp = encoder_for(Format::Webp).ok()?.encode(&img.px, img.has_transparency(), EncodeQuality::Lossy(95), &EncodeOptions { fast: true, ..EncodeOptions::default() }).ok()?;
                Some((Arc::new(webp), "image/webp"))
            }
        }
    }
}

// ---------- thumbnails ----------

type Job = Box<dyn FnOnce() + Send>;

/// A few threads that take the newest job first.
pub struct Pool {
    jobs: Arc<(Mutex<Vec<Job>>, Condvar)>,
}

impl Pool {
    pub fn new(threads: usize) -> Self {
        let jobs: Arc<(Mutex<Vec<Job>>, Condvar)> = Arc::default();
        for _ in 0..threads {
            let jobs = jobs.clone();
            std::thread::spawn(move || loop {
                let job = {
                    let mut q = jobs.0.lock();
                    while q.is_empty() {
                        jobs.1.wait(&mut q);
                    }
                    q.pop()
                };
                if let Some(job) = job {
                    job();
                }
            });
        }
        Self { jobs }
    }

    pub fn spawn(&self, job: impl FnOnce() + Send + 'static) {
        self.jobs.0.lock().push(Box::new(job));
        self.jobs.1.notify_one();
    }
}

/// Threads for thumbnails: enough to keep up with a fast scroll, leaving cores for the rest.
pub fn thumb_threads() -> usize {
    std::thread::available_parallelism().map_or(2, |n| n.get()).div_ceil(2).clamp(2, 6)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn previews_are_kept_per_recipe() {
        let a = Recipe::default();
        let mut b = Recipe::default();
        assert_eq!(recipe_key(&a), recipe_key(&b));
        b.size.long_side = Some(1200);
        assert_ne!(recipe_key(&a), recipe_key(&b));
    }

    #[test]
    fn the_newest_thumbnail_request_goes_first() {
        // One thread, held busy while three jobs queue: they run newest first.
        let pool = Pool::new(1);
        let (gate, order) = (Arc::new(Mutex::new(())), Arc::new(Mutex::new(Vec::new())));
        let hold = gate.lock();
        let g = gate.clone();
        pool.spawn(move || drop(g.lock()));
        std::thread::sleep(Duration::from_millis(50));
        for n in 1..=3 {
            let order = order.clone();
            pool.spawn(move || order.lock().push(n));
        }
        drop(hold);
        for _ in 0..100 {
            if order.lock().len() == 3 {
                break;
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        assert_eq!(*order.lock(), vec![3, 2, 1]);
    }
}
