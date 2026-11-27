// SPDX-License-Identifier: AGPL-3.0-or-later
//! Times each stage of processing one image, to see where the time goes.
//!
//!   cargo run --release -p wrangler-engine --example stages -- photo.jpg [recipe-id]

use std::time::Instant;

use wrangler_engine::codec::{decode_bytes, Format};
use wrangler_engine::modules::{self, Cx};
use wrangler_engine::paths::AppDirs;
use wrangler_engine::{presets, process, quality};

fn main() {
    let mut args = std::env::args().skip(1);
    let path = std::path::PathBuf::from(args.next().expect("usage: stages <image> [recipe]"));
    let id = args.next().unwrap_or_else(|| "compress-for-web".into());
    let recipe = presets::load(&AppDirs::detect().recipes(), &id).expect("recipe");
    let lap = |what: &str, t: Instant| println!("{what:<28}{:>8.1} ms", t.elapsed().as_secs_f64() * 1000.0);

    let t = Instant::now();
    let original = std::fs::read(&path).unwrap();
    lap("read", t);
    let t = Instant::now();
    let img = decode_bytes(&original, Format::from_path(&path)).unwrap();
    lap(&format!("decode {}x{}", img.px.width(), img.px.height()), t);
    let t = Instant::now();
    let stages = modules::stages(&recipe.size, &recipe.addons);
    let out = modules::apply(&stages, img, &mut Cx::default()).unwrap();
    lap(&format!("stages to {}x{}", out.px.width(), out.px.height()), t);
    let format = process::output_format(&recipe, Format::from_path(&path).unwrap_or(Format::Jpeg));
    let t = Instant::now();
    let e = quality::encode(&out.px, out.has_transparency(), format, &recipe.quality, recipe.format.matte).unwrap();
    lap(&format!("search + encode {format:?} q{:?}", e.quality), t);
    let t = Instant::now();
    let whole = process::process_path(&path, &recipe, &mut Cx::default()).unwrap();
    lap(&format!("whole, {} KB", whole.bytes.len() / 1024), t);
}
