# Architecture

Wrangler is one Rust image engine with three front doors: a desktop app, a command line tool, and file-manager quick actions. Everything a user can do is described by a **recipe**, and every feature in a recipe is a **module**. This document is the map, [MODULES.md](MODULES.md) is the how-to for adding features.

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
| `apps/desktop` | The desktop app. `src-tauri/` is the Rust host, `src/` is the Svelte front end. |
| `design/` | The design system (`system/`: tokens, component CSS, fonts, textures), the brand icon, and the first interactive prototype. |
| `docs/` | This file, the plan, the module guide, research notes. |
| `site/` | The download page at genuinemoron.github.io/img-wrangler. It reads files and sizes from the newest release, so a new version needs no edit. `.github/workflows/pages.yml` publishes it with the design system and the install scripts. |
| `install.sh`, `install.ps1`, `packaging/` | The one-line installers and the Arch `PKGBUILD` template the release fills in. |

Dependencies point one way: `cli → engine, integrate` and `desktop → engine, integrate`. The engine depends on nothing of ours.

## Recipes

A recipe is plain TOML. The app, the CLI and quick actions all read the same file, so a recipe tuned in the app runs identically from a script. Every field has a default. A recipe only states what it changes.

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
look = "balanced"           # small | balanced | high | pristine | lossless | 1 to 100
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
`Img` is an 8-bit RGBA buffer plus what the pipeline must carry: the source format, whether alpha is meaningful, the EXIF block (orientation already applied), and the sampled backdrop colour once known. Colour profiles are converted to sRGB on decode (`moxcms`). Output is always sRGB.

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
| 60 | watermark | add-on |
| 70 to 99 | future pixel add-ons (border, adjust) | add-on |
| after | encode (format, quality, metadata) | core, not a stage |
| after | name and save | core, not a stage |

`Pipeline::from_recipe` turns a recipe into a sorted `Vec<Box<dyn Stage>>`. Add-ons live in the `AddOn` enum (serde-tagged by `kind`), one file each in `modules/`.

Pixel work lives in `ops/` as pure functions (`resize`, `trim::detect`, `frame::layout`, `pad::composite`, `rotate`, `sharpen`, `overlay`). Modules compose ops. Ops know nothing about recipes. Geometry (`frame::layout`) is pure maths and unit-tested exhaustively.

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

### Host (`apps/desktop/src-tauri`)
- **Commands** (`commands.rs`) are thin wrappers over the engine. Anything slow runs on a blocking thread: `formats`, `recipes`, `load_recipe`, `save_recipe`, `delete_recipe`, `scan`, `forget`, `plan`, `estimate` (a newer one cancels an older one), `preview` (one image processed exactly as the run would, nothing written), `run` (progress streamed over a Tauri `Channel`), `quick_run`, `cancel`, `undo`, `history`, `take_launch`, `open_main`, `integrations`, `set_integrations`.
- **Protocols** serve images without base64: `thumb://localhost/<path>` (a cached WebP thumbnail, only for files in the batch) and `preview://localhost/<token>` (the last few previews, kept in memory).
- **Launch modes** (`launch.rs`): plain, `open <paths…>` and `quick --recipe <id> <paths…>`, which the right-click actions run. A single-instance guard sends later launches to the running app, and pending launches merge, so the one-process-per-file launches Explorer makes fold into one batch.
- **Windows**: `main`, and `quick`, the small unit a right-click action opens. Quick runs use their own file list (`quick_run`), so they never disturb the main window's batch. When the quick window closes and the main window was never shown, the app exits.

### Front end (`apps/desktop/src`)
```
src/
  main.ts, App.svelte        one bundle, two windows (index.html and index.html#quick)
  views/                     the regions of the unit: Header, Display (Sheet, Monitor, Empty),
                             RecipePanel, ChannelStrip, OutputPanel, dialogs, QuickWindow
  lib/components/            generic parts that know nothing about images: Key, KeyGroup,
                             ScreenChoice, Slider, Field, Screen, Ladder, Led, Tabs, Dialog,
                             ColourKey, AnchorPad, Row, HatScreen, Icon (+ icons.ts)
  lib/modules/               one folder per module, plus registry.ts and types.ts
  lib/state/                 Svelte 5 rune stores: recipe, batch, catalog, ui, and the
                             controller that keeps plan, estimate and preview current
  lib/ipc/                   the Backend interface, the Tauri implementation, and a mock
  lib/types/engine/          TypeScript types generated from the engine (ts-rs)
  lib/geometry.ts            the frame layout mirrored from the engine, for instant diagrams
```

- **Design system.** `design/system` (tokens, component CSS, fonts, textures) is imported directly through the `@design` alias, so there is one copy of every token.
- **Types.** `cargo test -p wrangler-engine --features ts export_bindings` regenerates `lib/types/engine`. The recipe the UI edits is the engine's own type, so the two can't drift.
- **State flow.** Controls write straight into `recipe.value` (a deep `$state`). The controller watches it: the plan refreshes within 40 ms, the sampled estimate after 450 ms, and the preview of the selected image after 220 ms. Stale replies are dropped by sequence number. `recipe.edited` compares against the last loaded or saved copy.
- **Mock backend.** In `pnpm dev` outside Tauri, `lib/ipc/mock.ts` plays a product shoot from `dev/samples`, draws previews on a canvas with the same frame layout as the engine, and simulates runs. `?demo` starts with files`#quick` opens the quick window. Production builds leave it out.
- **Sheet.** The contact sheet draws only the rows in view, so thousands of files scroll as lightly as a dozen (WebKitGTK on Linux is the slowest webview we ship on).

### The unit's layout
| Region | Holds |
|---|---|
| Header | grille (Add, drop target), recipe selector, Save, Add/History/Settings/Commands keys |
| Display | where the batch is, tabs (All, Ready, Done, Kept, Clash, Over, Skipped, Failed), the virtualised sheet, the monitor (result, original, split, 1:1 zoom) and the status line |
| Recipe panel | every step in run order, open one (accordion) for its Details, add-on keys switch on and off, Add step |
| Channel strip | the core modules that have channels: Size, Format, Quality, Name. Each screen is itself the control |
| Output panel | the meter, Save to, Undo and Run |

## Why this stack

Everything heavy (decoding, resizing, trimming, quality search, encoding, writing) is Rust, in the
engine, and would be the same whatever drew the window. The interface is a thin layer of about
4,000 lines in the system webview. We looked at drawing it natively in Rust instead (checked again
in November 2026):

- Iced and egui still can't take files dragged in from the file manager on Linux Wayland. The fix
  is in winit 0.31, which is still a beta.
- Slint added drag and drop in 1.17 and 1.18, but drops from Finder, Explorer or Dolphin aren't
  confirmed, and it has no blend modes or shader effects for the case material and the glowing
  screens.
- GPUI has had no official standalone release since October 2025.

A native interface would save memory and some start-up time, not processing time. The costs we
watch instead are WebKitGTK's performance on Linux (the sheet is virtualised and thumbnails come
from Rust) and the webview's memory. Tauri 3's Chromium runtime, in alpha, is the way out if
WebKitGTK ever becomes the bottleneck. Revisit when winit 0.31 ships.

## Testing
- Engine: unit tests per op and module (geometry, naming, trim detection, quality search bounds), golden-image tests on small fixtures, round-trip tests per codec, journal/undo tests on temp folders.
- CLI: end-to-end tests that run recipes on fixture folders.
- Integrations: generated files compared against expected text for every OS (runs on any OS).
- README graphics: `pnpm readme --shots` in `apps/desktop` rebuilds every banner, screenshot and the demo from the app, with photos made by the CLI.
- Front end: `svelte-check` for types, Vitest for logic (the geometry mirror runs the engine's own frame tests), and screenshot checks of the main views in mock mode.

## Safety rules
- Nothing is written until Run. Plans and previews never touch the source folder.
- Every write is temp-then-rename. Replacing an original always moves it to the undo store first.
- Undo only reverses files that are unchanged since the run.
- No network access, no telemetry.
