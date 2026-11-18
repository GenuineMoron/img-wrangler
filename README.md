# Wrangler

Wrangler takes a pile of images and makes them ready for where they're going, in one pass:
compress, convert, resize, square them up on a backdrop, trim empty background, rename, and strip
location data. It works on whole folders, shows you exactly what will happen to every file before
anything is written, and keeps your originals so any run can be undone.

It runs entirely on your computer: macOS, Linux and Windows. No accounts, no uploads, no telemetry.

> **Status:** early development, heading for 0.1. The engine, command line tool, desktop app and
> file-manager actions are built. Installers and packaging come next. See [docs/PLAN.md](docs/PLAN.md).

## What it does

- **Size and shape in one place.** Resize the long side, or put every image on the same shape
  (1:1, 4:5, 3:2, 16:9 or an exact size such as 1200×628) by cropping or padding. Padding can trim
  empty background first and centre the product so it fills, say, 85% of the frame, on its own
  backdrop colour, white, transparent or any colour.
- **Compression that looks the same everywhere.** Choose a look (small, balanced, high, pristine)
  instead of an encoder number. Wrangler measures the result and picks the setting per image. Add a
  maximum file size if you have a budget. Files never get bigger than they were.
- **Formats.** Reads JPEG, PNG, WebP, AVIF, HEIC, JPEG XL, GIF, TIFF, BMP, ICO, QOI, TGA, PNM and SVG.
  Writes JPEG (MozJPEG), PNG (oxipng, optional palette reduction), WebP, AVIF, GIF, TIFF, BMP, ICO, QOI.
- **Renaming.** Patterns like `spring-24-{n:000}`, find and replace (plain or regex), slugs and case,
  with clashes numbered automatically.
- **Safe by design.** Plans before runs, atomic writes, an undo store for every replaced original.
- **Recipes.** Save a setup once and run it from the app, the command line, or a right-click in
  Finder, Explorer, Dolphin, Nautilus, Nemo or Thunar.

## The app

One window that works like a piece of hardware. Drop images or folders anywhere on it. The four
channels along the bottom are the everyday controls, and each screen is the control itself:
drag the Size frame's corner, drag the red size-limit line, pick a format from the list. The
Recipe panel lists every step in the order it runs, with the rest of the settings one press away.
The big display shows every file as it will come out, with a live preview of the selected one,
before anything is written.

From a file manager, right-click images or folders and choose a pinned recipe: a small window runs
it, shows what was saved, and keeps Undo one press away. "Open in Wrangler…" loads them into the
main window instead. Set the menus up from Settings, or with `img-wrangler integrate install`.

## Command line

```sh
img-wrangler plan ~/Shoot -r shopify-product          # see what would happen
img-wrangler run ~/Shoot -r shopify-product            # do it
img-wrangler run ~/Shoot --size 2048 --format webp --look balanced --to replace
img-wrangler undo                                      # put the last run back
img-wrangler recipes                                   # built-in and saved recipes
img-wrangler integrate install --pin web-ready         # add right-click actions
```

Recipes are plain TOML`img-wrangler recipes show web-ready` prints one to start from.

## Building

You need Rust (stable). The engine and CLI need no other system libraries.

```sh
cargo test                                  # engine, CLI, integrations
cargo build --release -p img-wrangler
```

The desktop app also needs Node 22 and pnpm, plus WebKitGTK on Linux
(`libwebkit2gtk-4.1-dev` and friends, see the [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/)).

```sh
cd apps/desktop
pnpm install
pnpm tauri dev                              # the app, against the real engine
pnpm dev                                    # the UI alone in a browser, with sample photos (add ?demo)
pnpm check && pnpm test                     # types and tests
pnpm tauri build                            # installers for this OS
```

If you change an engine type the app uses, regenerate the TypeScript types with
`cargo test -p wrangler-engine --features ts export_bindings`.

## How it's built

A Rust engine (`crates/engine`) with three front doors: the CLI (`crates/cli`), a Tauri desktop app
(`apps/desktop`) and file-manager integrations (`crates/integrate`). Every feature is a module.
Start with [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md). Adding a feature is described in
[docs/MODULES.md](docs/MODULES.md).

## Licence

Wrangler is free software under the [GNU AGPL v3 or later](LICENSE). Commercial licences are
available from the author ([NOTICE](NOTICE)). Contributions are welcome under the
[CLA](CLA.md). See [CONTRIBUTING.md](CONTRIBUTING.md). The name and logo are covered by the
[trademark policy](TRADEMARK.md).
