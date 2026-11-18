# Changelog

Notable changes, newest first. Versions follow [semantic versioning](https://semver.org).

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
