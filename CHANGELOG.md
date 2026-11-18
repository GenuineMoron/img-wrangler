# Changelog

Notable changes, newest first. Versions follow [semantic versioning](https://semver.org).

## 0.1.0-alpha.3, 18 November 2026

Much faster, the hat in the menu bar, and updates from inside the app.

- Previews show in a moment. A draft of the framing comes first, the exact result follows, and the next image's preview is made while you look at this one. On a Mac Studio, arrowing to the next photo now shows its preview in about 50 ms, down from a second.
- Faster on Linux. The case and key textures are baked rather than blended on every repaint, glows and the screens' pixel grid are cheaper to draw, and the parts that change often sit on layers of their own. On a computer drawing without a graphics card, Wrangler notices and turns the soft shadows and glows down by itself (Settings, Effects). On a small Linux computer without one, choosing the next image, dragging the size and switching recipes now take about a fifth of the time they did.
- The hat sits in the menu bar on a Mac and in the system tray on Windows and Linux. Its menu runs a pinned recipe on images you choose, takes back the last run, and checks for updates. On a Mac, drop images straight on it.
- Updates from inside the app. Wrangler tells you when there's a new version, shows what changed, and installs it when you say so. Updates are signed, and the app checks the signature first. On Arch and CachyOS it gives you the command instead, as the package manager looks after updates there.
- Right-click actions in Finder now start Wrangler properly, and the ones already set up are updated when Wrangler starts. A quick action no longer opens the main window too.
- Right-click actions for Luciole, the Noctalia file manager.
- Right-click an image in the app to open it, show it in its folder, compare before and after, run the recipe on just that one, copy its path, or remove it from the batch.
- Layout fixes. On a Mac the channel labels were pushed out of place, "Shrink to fit" spilled out of its key, and some labels were cut off in a small window.
- The settings key is a gear, not a sun.
- `pnpm selftest` times the everyday steps in the real app or a browser and checks every screen for cut-off or overlapping controls. CI runs it in Chromium and WebKit.

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
