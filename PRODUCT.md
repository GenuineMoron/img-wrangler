# Product

<!-- impeccable:product-schema 1 -->

## Platform

web

(Desktop app for macOS, Linux and Windows. The interface is a web frontend inside a Tauri 2 shell, so it is designed and built as `web`. It must still feel at home on each OS: native menus, file dialogs, drag and drop from the file manager, keyboard shortcuts that follow each platform.)

## Stack

Delegated, with the user's lean toward Rust. Chosen after research (Nov 2026): a Rust core library (decode, transform, encode, rename, plan, journal) shared by a CLI and a Tauri 2 desktop app with a Svelte 5 + TypeScript frontend. Native Rust UI kits were ruled out for now: Iced and egui cannot receive file drops on Wayland, GPUI's usable crates are unofficial weekly snapshots, and Slint's external drop support is unverified. Linux WebKitGTK performance is the known risk, so the grid stays light and thumbnails are made in Rust.

## Users

People who prepare many images for the web and online shops, usually at work and under time pressure:

- Marketers and agency staff handling client product shots, blog images and campaign assets.
- E-commerce managers preparing catalogue images for Shopify, WooCommerce, Amazon, eBay and Australian marketplaces.
- Web developers who need images compressed, resized and named correctly before they ship.

The first user is the author, who uses the old tool heavily at work on macOS and CachyOS Linux. The project is also meant for anyone, on any of the three desktop OSes.

## Product Purpose

Wrangler takes a pile of images and makes them ready for where they are going, in one pass: compress, convert, resize, trim, pad onto a canvas with the right backdrop, rename, and strip or keep metadata. It works on whole folders, in place or into a new folder, without losing originals.

Success: a user drops a folder of a few thousand product photos, sees exactly what will happen to every file (new size, new name, new dimensions) before anything is written, runs it, and gets smaller, consistent, correctly named images with an undo if they change their mind.

## Positioning

Every other tool does one slice: ImageOptim and Caesium compress, PowerToys resizes, renamers rename, Photoroom pads and cuts out in the cloud. Wrangler is a single local recipe that chains all of it, shows the result for the whole batch before running, and processes in place safely with an undo journal. It is fast because the engine is native Rust, and private because nothing leaves the machine.

## Operating Context

- Inputs: folders of JPEG, PNG, WebP, HEIC, AVIF, TIFF, GIF and other formats straight from photographers, phones, suppliers and design tools, often with inconsistent names, sizes and backgrounds. Batches range from one image to several thousand; single images can be 50 MP.
- Typical jobs: "make these web-ready", "square white 2048 px canvas with even margins for the shop", "rename these to SKU-01, SKU-02", "convert everything to WebP under 200 KB", "process this folder in place".
- Work happens on laptops and desktops, often with Finder or a Linux file manager open alongside. Repeated jobs are saved as recipes and reused weekly.
- Developers also want the same recipes from a command line and in scripts.

## Capabilities and Constraints

Confirmed for v1:

- Formats in: JPEG, PNG, WebP, AVIF, GIF, BMP, TIFF, ICO, QOI, plus HEIC and JPEG XL where the platform library allows.
- Formats out: JPEG (MozJPEG), PNG (lossless optimise, optional palette quantise), WebP, AVIF, plus "keep original format".
- Steps: trim to content, resize (fit, fill/cover, pad/contain with backdrop, exact, long edge, short edge, width, height, never enlarge), canvas with even margins and target fill percentage, backdrop colour (auto from image edges, white, transparent, custom), format and quality (perceptual presets, exact quality, or a maximum file size), rename (tokens, find and replace, regex, case, sequence numbers), metadata (strip GPS and camera data by default, keep colour profile and orientation applied).
- Output: replace originals (with a recoverable backup), save beside originals with a suffix, or save to a folder that mirrors subfolders. Never overwrite a larger file with a smaller-looking one silently: the "never bigger" guard is on by default.
- A preview of the whole batch before anything is written: per-file new name, dimensions and estimated size, collisions flagged.
- Recipes saved as plain files usable by the GUI and the CLI (`img-wrangler`).
- Everything runs locally. No accounts, no telemetry.
- Wrangler lives where the files are. Selecting images or folders in Finder, Windows Explorer, Dolphin, Nautilus, Nemo or Thunar offers right-click actions: quick actions that run a pinned recipe immediately (with a small progress panel and undo), and "Open in Wrangler…" for the full window with those files loaded. Also "Open With", drop on the Dock or taskbar icon, and the `img-wrangler` command. Integrations are installed and removed from inside the app.

Later (designed for, not in v1): watch folders, clipboard optimise, responsive srcset sets and `<picture>` snippets, ThumbHash placeholders, local background removal and upscaling with commercially licensed models only, CSV/SKU rename mapping.

## Brand Commitments

- On-screen name: **Wrangler**. Repository, package and CLI name: `img-wrangler`.
- Licence: AGPL-3.0-or-later, copyright Sean Hawkey, with a contributor licence agreement so the author can also sell commercial or app-store builds. A trademark policy protects the name.
- Repository: GenuineMoron/img-wrangler, private until the first usable release.

## Evidence on Hand

- The previous Python/PyQt tool in `../ImageWrangler/` (feature reference only; its look is discarded).
- Research reports gathered Nov 2026 (frameworks, codecs, compression, competitors, e-commerce specs, licensing, distribution, performance), summarised in `docs/research.md`.
- No testimonials, user counts, benchmarks or press exist yet. Do not invent any.

## Product Principles

1. **See it before it happens.** Every batch is a visible plan first; writing files is the last, deliberate step.
2. **Safe by default.** Originals are recoverable, files never get bigger by accident, and collisions are caught before they happen.
3. **One recipe, everywhere.** The same recipe runs from the window, the command line and (later) a watched folder.
4. **Fast enough to feel instant.** Thumbnails stream in as soon as a folder lands; previews update while a slider moves.
5. **Plain words over codec jargon.** "Small, Balanced, High, Lossless" and "under 200 KB" first; encoder numbers are there for experts who want them.

## Accessibility & Inclusion

Full keyboard operation (command palette, shortcuts for every action), visible focus, WCAG 2.2 AA contrast in both light and dark appearance, respects reduced-motion and the OS light/dark setting, screen-reader labels on every control and progress state.
