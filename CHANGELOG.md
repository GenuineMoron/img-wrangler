# Changelog

Notable changes, newest first. Versions follow [semantic versioning](https://semver.org).

## 0.1.0-alpha.2, 18 November 2026

Easier to install. Nothing in the app itself has changed.

- One Mac download that runs on Apple silicon and Intel alike, instead of one per chip.
- A [download page](https://genuinemoron.github.io/img-wrangler/) that works out which computer you're on and offers the right file, with every other system one tap away.
- A shorter install command for macOS and Linux: `curl -fsSL https://genuinemoron.github.io/img-wrangler/install | sh`
- Release notes open with a table of which file you need, and the duplicate Mac archives are gone from the list.

## 0.1.0-alpha.1, 18 November 2026

The first alpha: a complete rewrite in Rust, for trying out and finding what breaks. Builds aren't signed yet, and the app hasn't been tried on Windows or macOS yet.

- An image engine that plans every file before writing, runs batches in parallel within a memory budget, writes atomically and keeps 30 days of undo.
- Size, shape, crop, pad and trim as one control, with the backdrop taken from the photo.
- Compression by measured look (SSIMULACRA 2), size limits, and keeping originals that wouldn't get smaller.
- Renaming by pattern, find and replace, or a list pasted from a spreadsheet.
- Watermark, rotate and sharpen steps. Metadata kept, stripped or kept without location.
- Reads 14 formats including HEIC, AVIF and JPEG XL, and writes 11.
- The command line tool, `img-wrangler`.
- The desktop app for macOS, Windows and Linux.
- Right-click actions for Finder, Explorer, Dolphin, Files, Nemo and Thunar, with a quick window and undo.
- Downloads for macOS (`.dmg`), Windows (setup `.exe`) and Linux (`.deb`, `.rpm`, AppImage, and a `PKGBUILD` for Arch and CachyOS), plus one-line install scripts. The Linux builds run on any system from Ubuntu 22.04 and Debian 12 on.
