# Architecture

Wrangler is one Rust image engine with three front doors: a desktop app, a command line tool, and file-manager quick actions. Everything a user can do is described by a **recipe**, and every feature in a recipe is a **module**. This document is the map; [MODULES.md](MODULES.md) is the how-to for adding features.

```
                ┌──────────────────────────── apps/desktop ────────────────────────────┐
 Finder /       │  Svelte 5 UI (src/)                    Tauri host (src-tauri/)        │
 Explorer /     │  design system ─ components ─ modules  commands · events · protocols │
 Dolphin ──────▶│  stores (recipe, batch, selection)  ◀─▶ thumb:// preview:// windows  │
 right-click    └───────────────────────────────▲──────────────────────────────────────┘
                                                │ calls
 terminal ─────▶ crates/cli (img-wrangler) ─────┤
                                                ▼
                ┌──────────────────────────── crates/engine ───────────────────────────┐
                │ recipe ─▶ plan ─▶ run ──▶ journal (undo)                              │
                │            │       │                                                  │
                │      modules (size, rotate, sharpen, …) ─ ops (pixels) ─ codec        │
                │      naming · estimate · thumbs · scan · presets                      │
                └──────────────────────────────────────────────────────────────────────┘
                crates/integrate: installs right-click actions per OS (no image code)
```

## Repository

| Path | What lives there |
|---|---|
| `crates/engine` (`wrangler-engine`) | The image engine. No UI, no OS integration, no global state. Everything is testable without a window. |
| `crates/integrate` (`wrangler-integrate`) | Generates and installs file-manager actions: Finder Quick Actions, Windows Explorer verbs, Dolphin service menus, Nautilus/Nemo/Thunar actions. Pure text generation plus small installers. |
| `crates/cli` (`img-wrangler`) | The command line tool. Thin: parses arguments into a recipe and calls the engine. |
| `apps/desktop` | The desktop app. `src-tauri/` is the Rust host; `src/` is the Svelte front end. |
| `design/` | Design system source (`system/tokens.css`, `components.css`, textures) and the interactive prototype. |
| `docs/` | This file, the plan, the module guide, research notes. |

Dependencies point one way: `cli → engine, integrate`; `desktop → engine, integrate`. The engine depends on nothing of ours.

## Recipes

A recipe is plain TOML. The app, the CLI and quick actions all read the same file, so a recipe tuned in the app runs identically from a script. Every field has a default; a recipe only states what it changes.

```toml
version = 1
name = "Shopify product"

[size]                      # core module: size, shape, fit, trim, padding
long_side = 2048            # omit for original size
shape = "1:1"               # original | 1:1 | 4:5 | 3:2 | 16:9 | 1200x628
fit = "pad"                 # crop (fill the frame) | pad (fit inside on a backdrop) | inside (fit, no padding)
trim = { tolerance = 4 }    # cut empty background first; omit to keep edges
pad = { backdrop = "auto", fill = 85 }

[format]                    # core module
id = "webp"                 # same | jpg | png | webp | avif | gif | tiff | bmp | ico | qoi

[quality]                   # core module
look = "balanced"           # small | balanced | high | pristine | lossless | 1–100
max_kb = 400
never_bigger = true

[name]                      # core module
mode = "pattern"            # keep | pattern | replace
pattern = "spring-24-{n:000}"

[metadata]
keep = "none"               # none | safe (drops GPS) | all

[[addons]]                  # optional modules; each runs at its fixed stage
kind = "sharpen"
amount = 40

[destination]
mode = "replace"            # beside | subfolder | folder | replace
```

## Engine

### Image model
`Img` is an 8-bit RGBA buffer plus what the pipeline must carry: the source format, whether alpha is meaningful, the EXIF block (orientation already applied), and the sampled backdrop colour once known. Colour profiles are converted to sRGB on decode (`moxcms`); output is always sRGB.

### Modules and stages
A **module** is one user-facing feature. In the engine, a module that touches pixels implements `Stage`:

```rust
pub trait Stage: Send + Sync {
    fn id(&self) -> &'static str;
    fn order(&self) -> u16;                       // fixed position in the pipeline
    fn plan(&self, dims: Dims) -> Planned;        // predict output size without pixels
    fn apply(&self, img: Img, cx: &mut Cx) -> Result<Img>;
}
```

Order is fixed, not user-sorted, so recipes can't be built in a broken order:

| Order | Stage | Module |
|---|---|---|
| 10 | orientation | always (EXIF) |
| 20 | rotate | add-on |
| 30 | size (trim → fit → crop/pad) | core |
| 50 | sharpen | add-on |
| 60+ | future pixel add-ons (watermark, border, adjust) | add-on |
| — | encode (format + quality + metadata) | core, not a stage |
| — | name | core, not a stage |

`Pipeline::from_recipe` turns a recipe into a sorted `Vec<Box<dyn Stage>>`. Add-ons live in the `AddOn` enum (serde-tagged by `kind`), one file each in `modules/`.

Pixel work lives in `ops/` as pure functions (`resize`, `trim::detect`, `frame::layout`, `pad::composite`, `rotate`, `sharpen`). Modules compose ops; ops know nothing about recipes. Geometry (`frame::layout`) is pure maths and unit-tested exhaustively.

### Codecs
`codec/` has a format registry: one `FormatInfo` per format (id, label, extensions, MIME, can decode/encode, alpha, animation, lossy/lossless, browser support). The UI asks the engine for this list at start-up, so a new codec appears in the app without UI changes. Each encoder implements:

```rust
pub trait Encoder: Send + Sync {
    fn encode(&self, img: &Img, q: EncodeQuality, meta: &MetaOut) -> Result<Vec<u8>>;
    fn quality_range(&self) -> Option<(u8, u8)>;   // None for lossless-only formats
}
```

Decoders: `image` (JPEG, PNG, GIF, WebP, TIFF, BMP, ICO, QOI, TGA, PNM), `avif-decode` (AVIF, pure Rust via rav1d), `jxl-oxide` (JPEG XL), `resvg` (SVG), HEIC via the OS (macOS `sips`, Linux `heif-dec`) or the optional `heic` feature. Encoders: MozJPEG, oxipng + imagequant, libwebp, ravif, plus `image` for GIF/TIFF/BMP/ICO/QOI.

### Quality
"Looks" (small, balanced, high, pristine) aim at a SSIMULACRA 2 score (65, 75, 83, 90). `quality::search` bisects encoder quality on a few detail-rich crops of the output (fast), then encodes the full image once. A size limit bisects on the full output and only shrinks dimensions if the recipe allows. The never-bigger rule keeps the original when re-encoding in the same format and shape wouldn't save anything.

### Planning, estimating, running
- `scan` expands dropped files and folders (natural sort, filters, hidden files skipped).
- `plan` computes, without decoding pixels, every file's output path, name, predicted dimensions and conflicts. It is instant for thousands of files and is what the app shows before anything runs.
- `estimate` encodes a small sample in the background to predict output bytes per format and look, then extrapolates per file. The app shows these as "≈".
- `run` executes a plan on a rayon pool with a memory budget (a weighted semaphore by pixel count), cancellation, and throttled progress events. Every write is atomic (temp file, then rename).
- `journal` records each run: what was created, and every original moved into the undo store before being replaced. `undo` reverses a run only where files are unchanged since. Runs older than the retention period (30 days by default) are pruned.
- `thumbs` makes small cached previews keyed by path, size and modification time, using JPEG DCT scaling for speed.

## Desktop app

### Host (`src-tauri`)
- **Commands** are thin wrappers over the engine: `scan`, `plan`, `estimate`, `preview`, `run`, `cancel`, `undo`, `history`, `recipes_*`, `formats`, `integrations_*`.
- **Events** stream progress and estimates (`batch://progress`, `batch://estimate`).
- **Protocols** serve images without base64: `thumb://<file-id>` and `preview://<token>`.
- **Windows**: the main window, and the compact quick-action window opened by right-click actions (`wrangler quick --recipe <id> <paths…>`). A single-instance guard folds the many processes Explorer starts (one per file) into one batch.

### Front end (`src`)
- `lib/design/` holds the design tokens and component CSS (copied from `design/system`).
- `lib/components/` holds generic components that know nothing about images: `Key`, `KeyGroup`, `Slider`, `Field`, `Screen`, `Led`, `Ladder`, `Readout`, `Tabs`, `Tile`, `Icon`.
- `lib/modules/` holds one folder per module, each exporting a `UiModule`:
  ```ts
  interface UiModule<S> {
    id: string; label: string; accent: Accent; glyph: string;
    core: boolean; order: number;
    summary(s: S, cx: Cx): string;
    Screen?: Component;   // the instrument shown in the channel strip (core modules)
    Keys?: Component;     // the key row under it
    Details: Component;   // full settings, shown in the recipe panel
  }
  ```
  `lib/modules/registry.ts` lists them. The channel strip renders core modules; the recipe panel renders all modules in run order. Nothing else knows which modules exist.
- `lib/state/` holds Svelte 5 rune stores: the recipe being edited, the batch (files, plan, estimates), the selection, and UI state.
- `lib/ipc/` wraps commands with types generated from the engine (`ts-rs`), plus a **mock backend** so the UI runs in a plain browser for design work and screenshot tests.

## Testing
- Engine: unit tests per op and module (geometry, naming, trim detection, quality search bounds), golden-image tests on small fixtures, round-trip tests per codec, journal/undo tests on temp folders.
- CLI: end-to-end tests that run recipes on fixture folders.
- Integrations: generated files compared against expected text for every OS (runs on any OS).
- Front end: component tests (Vitest) and screenshot checks of the main views in mock mode.

## Safety rules
- Nothing is written until Run. Plans and previews never touch the source folder.
- Every write is temp-then-rename. Replacing an original always moves it to the undo store first.
- Undo only reverses files that are unchanged since the run.
- No network access, no telemetry.
