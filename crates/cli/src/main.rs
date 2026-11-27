// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Sean Hawkey

//! `img-wrangler`: the command line front door to the Wrangler engine.

use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use anyhow::{bail, Context, Result};
use clap::{Args, Parser, Subcommand};
use indicatif::{ProgressBar, ProgressStyle};
use wrangler_engine::codec::metadata::KeepMeta;
use wrangler_engine::codec::{formats, probe};
use wrangler_engine::journal::{history, undo};
use wrangler_engine::modules::{AddOn, Backdrop, Rotate, Sharpen, Trim, Watermark};
use wrangler_engine::ops::frame::{Fit, Shape};
use wrangler_engine::paths::AppDirs;
use wrangler_engine::plan::{build, Action, PlanCx};
use wrangler_engine::recipe::{DestinationMode, Look, OutputFormat, Rename, RenameMode};
use wrangler_engine::run::{run, Event, RunOptions, Status};
use wrangler_engine::{presets, scan, Recipe};

#[derive(Parser)]
#[command(name = "img-wrangler", version, about = "Compress, resize, convert and rename images in batches.", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Process images and folders with a recipe.
    Run(RunArgs),
    /// Show what `run` would do, without writing anything.
    Plan(RunArgs),
    /// List, show, save or delete recipes.
    Recipes {
        #[command(subcommand)]
        action: Option<RecipeCmd>,
    },
    /// Undo a run (the latest by default).
    Undo {
        /// Run id from `img-wrangler history`.
        id: Option<String>,
    },
    /// Recent runs.
    History,
    /// Format and dimensions of image files.
    Info { files: Vec<PathBuf> },
    /// Formats Wrangler reads and writes.
    Formats,
    /// Run a recipe on files picked in a file manager (what the right-click actions call when
    /// the desktop app isn't installed). Shows a notification when done.
    Quick {
        #[arg(long, default_value = "just-compress")]
        recipe: String,
        #[arg(required = true)]
        paths: Vec<PathBuf>,
    },
    /// Open files and folders in the Wrangler app.
    Open { paths: Vec<PathBuf> },
    /// Add or remove right-click actions in your file manager.
    Integrate {
        #[command(subcommand)]
        action: IntegrateCmd,
    },
}

#[derive(Subcommand)]
enum RecipeCmd {
    List,
    /// Print a recipe as TOML.
    Show { name: String },
    /// Save a recipe file into your recipes.
    Save { file: PathBuf },
    Delete { name: String },
    /// Where your recipes are kept.
    Path,
}

#[derive(Subcommand)]
enum IntegrateCmd {
    /// Which file managers have Wrangler actions.
    Status,
    /// Install actions for the recipes that are switched on (and "Open in Wrangler").
    Install {
        /// Offer these recipes instead of the ones switched on.
        #[arg(long = "pin")]
        pins: Vec<String>,
        /// Program the actions run (defaults to the Wrangler app, then this CLI).
        #[arg(long)]
        program: Option<PathBuf>,
    },
    /// Remove every Wrangler action.
    Remove,
}

#[derive(Args, Clone)]
struct RunArgs {
    /// Image files and folders.
    #[arg(required = true)]
    paths: Vec<PathBuf>,
    /// Recipe name, id, or path to a .toml file.
    #[arg(short, long)]
    recipe: Option<String>,
    /// Long side in pixels.
    #[arg(long)]
    size: Option<u32>,
    /// original, 1:1, 4:5, 3:2, 16:9 or WxH.
    #[arg(long)]
    shape: Option<String>,
    /// crop, pad or inside.
    #[arg(long)]
    fit: Option<String>,
    /// Trim empty background first, with an optional tolerance in percent.
    #[arg(long, num_args = 0..=1, default_missing_value = "4")]
    trim: Option<u8>,
    /// Backdrop for padding: auto, white, clear or #hex.
    #[arg(long)]
    backdrop: Option<String>,
    /// Percent of the frame the image fills when padding.
    #[arg(long)]
    fill: Option<f32>,
    /// Allow making images bigger.
    #[arg(long)]
    upscale: bool,
    /// same, jpg, png, webp, avif, gif, tiff, bmp, ico or qoi.
    #[arg(short, long)]
    format: Option<String>,
    /// small, balanced, high, pristine, lossless or a number 1-100.
    #[arg(short, long)]
    look: Option<String>,
    /// Maximum file size in KB.
    #[arg(long)]
    max_kb: Option<u32>,
    /// Rename with a pattern, e.g. "spring-{n:000}".
    #[arg(long)]
    rename: Option<String>,
    /// Metadata to keep: none, safe (no GPS) or all.
    #[arg(long)]
    keep_meta: Option<String>,
    /// Sharpen after resizing, amount in percent.
    #[arg(long, num_args = 0..=1, default_missing_value = "40")]
    sharpen: Option<u16>,
    /// Rotate clockwise by 90, 180 or 270 degrees.
    #[arg(long)]
    rotate: Option<u32>,
    /// Put this logo in the bottom-right corner of every image.
    #[arg(long, value_name = "LOGO")]
    watermark: Option<PathBuf>,
    /// New names from a file: `old, new` pairs (CSV or tab-separated) or one name per line.
    #[arg(long, value_name = "FILE")]
    names: Option<PathBuf>,
    /// replace, beside, subfolder, or a folder path.
    #[arg(long)]
    to: Option<String>,
    /// Look inside folders within folders.
    #[arg(short = 'R', long)]
    recursive: bool,
    /// Worker threads (default: all cores).
    #[arg(short, long, default_value_t = 0)]
    jobs: usize,
    /// Print events as JSON lines instead of a progress bar.
    #[arg(long)]
    json: bool,
}

fn recipe_from(args: &RunArgs, dirs: &AppDirs) -> Result<Recipe> {
    let mut r = match &args.recipe {
        Some(key) => presets::load(&dirs.recipes(), key)?,
        None => Recipe { name: "Command line".into(), ..Recipe::default() },
    };
    if let Some(s) = args.size {
        r.size.long_side = Some(s);
    }
    if let Some(s) = &args.shape {
        r.size.shape = Shape::parse(s).with_context(|| format!("not a shape: {s}"))?;
    }
    if let Some(f) = &args.fit {
        r.size.fit = match f.as_str() {
            "crop" => Fit::Crop,
            "pad" => Fit::Pad,
            "inside" => Fit::Inside,
            other => bail!("fit must be crop, pad or inside, not {other}"),
        };
    }
    if let Some(t) = args.trim {
        r.size.trim = Some(Trim { tolerance: t, ..Trim::default() });
    }
    if let Some(b) = &args.backdrop {
        r.size.pad.backdrop = toml::Value::String(b.clone()).try_into::<Backdrop>().with_context(|| format!("not a backdrop: {b}"))?;
    }
    if let Some(f) = args.fill {
        r.size.pad.fill = f;
    }
    if args.upscale {
        r.size.upscale = true;
    }
    if let Some(f) = &args.format {
        r.format.id = toml::Value::String(f.clone()).try_into::<OutputFormat>().with_context(|| format!("unknown format: {f}"))?;
    }
    if let Some(l) = &args.look {
        r.quality.look = l.parse::<u8>().map(Look::Exact).or_else(|_| toml::Value::String(l.clone()).try_into::<Look>()).with_context(|| format!("unknown look: {l}"))?;
    }
    if let Some(kb) = args.max_kb {
        r.quality.max_kb = Some(kb);
    }
    if let Some(p) = &args.rename {
        r.rename = Rename { mode: RenameMode::Pattern, pattern: p.clone(), ..r.rename };
    }
    if let Some(k) = &args.keep_meta {
        r.metadata.keep = toml::Value::String(k.clone()).try_into::<KeepMeta>().with_context(|| format!("keep-meta must be none, safe or all, not {k}"))?;
    }
    if let Some(a) = args.sharpen {
        r.addons.push(AddOn::Sharpen(Sharpen { amount: a, ..Sharpen::default() }));
    }
    if let Some(t) = args.rotate {
        r.addons.push(AddOn::Rotate(Rotate { turn: t, ..Rotate::default() }));
    }
    if let Some(logo) = &args.watermark {
        r.addons.push(AddOn::Watermark(Watermark { image: Some(logo.clone()), ..Watermark::default() }));
    }
    if let Some(file) = &args.names {
        let list = std::fs::read_to_string(file).with_context(|| file.display().to_string())?;
        r.rename = Rename { mode: RenameMode::List, list, ..r.rename };
    }
    if let Some(to) = &args.to {
        match to.as_str() {
            "replace" => r.destination.mode = DestinationMode::Replace,
            "beside" => r.destination.mode = DestinationMode::Beside,
            "subfolder" => r.destination.mode = DestinationMode::Subfolder,
            path => {
                r.destination.mode = DestinationMode::Folder;
                r.destination.folder = Some(path.to_string());
            }
        }
    }
    if args.recursive {
        r.select.recursive = true;
    }
    r.validate()?;
    Ok(r)
}

/// The size change as people read it: −58% when smaller, +12% when bigger.
fn change(before: u64, after: u64) -> String {
    if before == 0 {
        return "no change".into();
    }
    let pct = (after as f64 / before as f64 - 1.0) * 100.0;
    if pct < 0.0 { format!("−{:.0}%", -pct) } else { format!("+{pct:.0}%") }
}

fn human(bytes: u64) -> String {
    let b = bytes as f64;
    if b >= 1e9 { format!("{:.2} GB", b / 1e9) } else if b >= 1e6 { format!("{:.1} MB", b / 1e6) } else { format!("{:.0} KB", b / 1e3) }
}

fn cmd_run(args: &RunArgs, dirs: &AppDirs, dry: bool) -> Result<ExitCode> {
    let recipe = recipe_from(args, dirs)?;
    let sources = scan::collect(&args.paths, &recipe.select)?;
    if sources.is_empty() {
        bail!("no images found in {}", args.paths.iter().map(|p| p.display().to_string()).collect::<Vec<_>>().join(", "));
    }
    let plan = build(&sources, &recipe, &PlanCx::default());
    if dry {
        if args.json {
            println!("{}", serde_json::to_string_pretty(&plan)?);
            return Ok(ExitCode::SUCCESS);
        }
        for item in &plan.items {
            let dims = item.planned.map(|p| format!("{}{}", if p.exact { "" } else { "≈" }, p.dims)).unwrap_or_else(|| "?".into());
            match &item.action {
                Action::Skip { reason } => println!("skip  {}  ({reason})", item.source.display()),
                _ => println!("{}  →  {}  {dims}{}", item.source.display(), item.output.display(), if item.renamed_for_clash { "  (renamed: name taken)" } else { "" }),
            }
        }
        println!("\n{} images, {} in, {} name clashes. Nothing was written.", plan.items.len(), human(plan.bytes_in), plan.clashes);
        return Ok(ExitCode::SUCCESS);
    }

    let replacing = plan.items.iter().filter(|i| i.in_place || recipe.destination.mode == DestinationMode::Replace).count();
    if replacing > 0 && !args.json {
        eprintln!("Replacing {replacing} originals. Undo with: img-wrangler undo");
    }
    let opts = RunOptions { threads: args.jobs, memory_budget: 2 << 30, runs_dir: dirs.runs() };
    let cancel = Arc::new(AtomicBool::new(false));
    let bar = (!args.json).then(|| {
        let b = ProgressBar::new(plan.items.len() as u64);
        b.set_style(ProgressStyle::with_template("{bar:32.yellow/black} {pos}/{len}  {msg}").expect("valid template"));
        b
    });
    let json = args.json;
    let summary = run(&plan, &recipe, &opts, &cancel, &|e| {
        if json {
            println!("{}", serde_json::to_string(&e).expect("serialisable"));
        } else if let (Some(b), Event::Item(r)) = (&bar, &e) {
            b.inc(1);
            if matches!(r.status, Status::Failed) {
                b.println(format!("failed: {}: {}", plan.items[r.id as usize].source.display(), r.message.clone().unwrap_or_default()));
            }
        }
    })?;
    if let Some(b) = bar {
        b.finish_and_clear();
        println!(
            "{} done, {} kept as they were, {} over the size limit, {} skipped, {} failed. {} → {} ({}) in {:.1}s. Run {}",
            summary.done, summary.kept, summary.over_limit, summary.skipped, summary.failed, human(summary.bytes_in), human(summary.bytes_out), change(summary.bytes_in, summary.bytes_out), summary.seconds, summary.run_id
        );
    }
    Ok(if summary.failed > 0 { ExitCode::from(2) } else { ExitCode::SUCCESS })
}

/// A desktop notification, best effort: the result of a quick action should be seen even
/// though no terminal is open.
fn notify(title: &str, body: &str) {
    use std::process::Command as Proc;
    let _ = if cfg!(target_os = "macos") {
        let quote = |s: &str| s.replace('\\', "\\\\").replace('"', "\\\"");
        Proc::new("osascript").args(["-e", &format!("display notification \"{}\" with title \"{}\"", quote(body), quote(title))]).status()
    } else if cfg!(windows) {
        let ps = format!("[void][System.Reflection.Assembly]::LoadWithPartialName('System.Windows.Forms'); $n = New-Object System.Windows.Forms.NotifyIcon; $n.Icon = [System.Drawing.SystemIcons]::Information; $n.Visible = $true; $n.ShowBalloonTip(6000, '{}', '{}', 'None'); Start-Sleep 6", title.replace('\'', "''"), body.replace('\'', "''"));
        Proc::new("powershell").args(["-NoProfile", "-WindowStyle", "Hidden", "-Command", &ps]).spawn().map(|_| std::process::ExitStatus::default())
    } else {
        Proc::new("notify-send").args(["--app-name=Wrangler", title, body]).status()
    };
}

fn cmd_quick(recipe: &str, paths: &[PathBuf], dirs: &AppDirs) -> Result<ExitCode> {
    let r = presets::load(&dirs.recipes(), recipe)?;
    let sources = scan::collect(paths, &r.select)?;
    if sources.is_empty() {
        notify(&r.name, "No images in the selection.");
        bail!("no images found");
    }
    let plan = build(&sources, &r, &PlanCx::default());
    let opts = RunOptions { threads: 0, memory_budget: 2 << 30, runs_dir: dirs.runs() };
    let s = run(&plan, &r, &opts, &AtomicBool::new(false), &|_| {})?;
    let mut msg = format!("{} images, {} → {} ({}).", s.done + s.kept, human(s.bytes_in), human(s.bytes_out), change(s.bytes_in, s.bytes_out));
    if s.failed > 0 {
        msg.push_str(&format!(" {} failed.", s.failed));
    }
    msg.push_str(" Undo: img-wrangler undo");
    println!("{msg}");
    notify(&r.name, &msg);
    Ok(if s.failed > 0 { ExitCode::from(2) } else { ExitCode::SUCCESS })
}

fn cmd_open(paths: &[PathBuf]) -> Result<ExitCode> {
    let app = wrangler_integrate::default_program().context("the Wrangler app isn't installed. Use `img-wrangler run` or `img-wrangler quick` instead")?;
    if cfg!(target_os = "macos") {
        // Through Launch Services, so a running app receives the files instead of a second copy starting.
        let bundle = app.ancestors().nth(3).map(PathBuf::from).unwrap_or(app.clone());
        std::process::Command::new("open").arg("-a").arg(bundle).arg("--args").arg("open").args(paths).spawn()?;
    } else {
        std::process::Command::new(app).arg("open").args(paths).spawn()?;
    }
    Ok(ExitCode::SUCCESS)
}

fn main() -> ExitCode {
    match real_main() {
        Ok(code) => code,
        Err(e) => {
            eprintln!("img-wrangler: {e:#}");
            ExitCode::FAILURE
        }
    }
}

fn real_main() -> Result<ExitCode> {
    let cli = Cli::parse();
    let dirs = AppDirs::detect();
    match cli.command {
        Command::Run(a) => cmd_run(&a, &dirs, false),
        Command::Plan(a) => cmd_run(&a, &dirs, true),
        Command::Recipes { action } => {
            match action.unwrap_or(RecipeCmd::List) {
                RecipeCmd::List => {
                    for e in presets::list(&dirs.recipes()) {
                        println!("{:<20} {}{}{}  {}", e.id, e.name, if e.built_in { "" } else { " (yours)" }, if e.on { "" } else { " (off)" }, e.description);
                    }
                }
                RecipeCmd::Show { name } => print!("{}", presets::load(&dirs.recipes(), &name)?.to_toml()?),
                RecipeCmd::Save { file } => {
                    let r = Recipe::from_toml(&std::fs::read_to_string(&file).with_context(|| file.display().to_string())?)?;
                    println!("saved as {}", presets::save(&dirs.recipes(), &r)?);
                }
                RecipeCmd::Delete { name } => presets::delete(&dirs.recipes(), &name)?,
                RecipeCmd::Path => println!("{}", dirs.recipes().display()),
            }
            Ok(ExitCode::SUCCESS)
        }
        Command::Undo { id } => {
            let id = match id {
                Some(id) => id,
                None => history(&dirs.runs()).into_iter().find(|r| !r.undone).map(|r| r.id).context("nothing to undo")?,
            };
            let r = undo(&dirs.runs(), &id)?;
            println!("Restored {} originals, removed {} new files.", r.restored, r.removed);
            for (p, why) in r.skipped {
                println!("left alone: {}: {why}", p.display());
            }
            Ok(ExitCode::SUCCESS)
        }
        Command::History => {
            for r in history(&dirs.runs()) {
                println!("{}  {:<20} {:>5} files  {} → {}{}", r.id, r.recipe, r.files, human(r.bytes_in), human(r.bytes_out), if r.undone { "  (undone)" } else { "" });
            }
            Ok(ExitCode::SUCCESS)
        }
        Command::Info { files } => {
            for f in files {
                match probe(&f) {
                    Ok(p) => println!("{}  {}  {}  {}", f.display(), p.format.name(), p.dims.map(|d| d.to_string()).unwrap_or_else(|| "?".into()), human(p.bytes)),
                    Err(e) => println!("{}  {e}", f.display()),
                }
            }
            Ok(ExitCode::SUCCESS)
        }
        Command::Formats => {
            for f in formats() {
                println!("{:<8} read{}  {}", f.label, if f.encode { " write" } else { "      " }, f.hint);
            }
            Ok(ExitCode::SUCCESS)
        }
        Command::Quick { recipe, paths } => cmd_quick(&recipe, &paths, &dirs),
        Command::Open { paths } => cmd_open(&paths),
        Command::Integrate { action } => integrate(action, &dirs),
    }
}

fn integrate(action: IntegrateCmd, dirs: &AppDirs) -> Result<ExitCode> {
    use wrangler_integrate as wi;
    match action {
        IntegrateCmd::Status => {
            for t in wi::targets() {
                let place = wi::where_it_shows(t).map(|w| format!(" ({w})")).unwrap_or_default();
                println!("{:<17} {}{place}", t.label(), if wi::installed(t) { "installed" } else { "not installed" });
            }
        }
        IntegrateCmd::Install { pins, program } => {
            let program = program.or_else(wi::default_program).unwrap_or(std::env::current_exe()?);
            let actions = if pins.is_empty() {
                presets::enabled(&dirs.recipes()).into_iter().map(|e| wi::QuickAction { id: e.id, label: e.name }).collect()
            } else {
                pins.iter()
                    .map(|p| presets::load(&dirs.recipes(), p).map(|r| wi::QuickAction { id: wrangler_engine::naming::slug(p), label: r.name }))
                    .collect::<Result<Vec<_>, _>>()?
            };
            let setup = wi::Setup { program, actions, extensions: wrangler_engine::codec::Format::extensions().into_iter().map(String::from).collect(), asked: true };
            for t in wi::targets() {
                match wi::install(t, &setup) {
                    Ok(files) => println!("{}: installed ({} files)", t.label(), files.len()),
                    Err(e) => println!("{}: {e}", t.label()),
                }
            }
        }
        IntegrateCmd::Remove => {
            for t in wi::targets() {
                match wi::remove(t) {
                    Ok(()) => println!("{}: removed", t.label()),
                    Err(e) => println!("{}: {e}", t.label()),
                }
            }
        }
    }
    Ok(ExitCode::SUCCESS)
}
