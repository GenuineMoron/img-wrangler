# Plan

Milestones in build order. Each one ends working and tested, so the project is usable at every step.

## M1 · Engine foundation
- [x] Workspace: `crates/engine`, `crates/integrate`, `crates/cli`, `apps/desktop`
- [x] Image model, decode (all formats incl. AVIF, JPEG XL, SVG, HEIC via OS), orientation, sRGB conversion
- [x] Format registry and encoders: JPEG (MozJPEG), PNG (oxipng, optional palette), WebP, AVIF, GIF, TIFF, BMP, ICO, QOI
- [x] Ops: resize, trim detection, frame layout, pad, crop, rotate, sharpen
- [x] Modules: size (core), rotate, sharpen (add-ons). Stage pipeline
- [x] Quality: look targeting (SSIMULACRA 2), size limit, never bigger
- [x] Naming: patterns, replace, case, collisions
- [x] Recipe TOML with defaults, validation, built-in presets

## M2 · Batch engine
- [x] Scan, plan (instant, no pixels), estimate (sampled), run (parallel, memory budget, cancel, progress)
- [x] Atomic writes, destinations (beside, subfolder, folder, replace), keep dates
- [x] Journal and undo with retention
- [x] Thumbnail cache

## M3 · Command line
- [x] `img-wrangler run | plan | recipes | undo | history | info | integrate | quick | open`
- [x] Flags for common settings layered on a recipe. JSON output for scripts

## M4 · Desktop app
- [x] Tauri host: commands, events, `thumb://` and `preview://`, single instance, window state
- [x] Svelte front end: design system, generic components, module registry, stores, typed IPC, mock backend
- [x] Main window: header, display (sheet, monitor, tabs, hints), channel strip, recipe panel, output panel
- [x] Keyboard model and command palette
- [ ] Accessibility pass with a screen reader on each OS
- [x] Quick-action window

## M5 · Lives where the files are
- [x] Finder Quick Actions, Windows Explorer verbs, Dolphin / Nautilus / Nemo / Thunar actions
- [x] Install, update and remove from Settings and from the CLI
- [ ] Open With and drag onto the Dock / taskbar icon (declared. MacOS Open With and Dock drops handled, to try on a Mac)
- [ ] Try each integration on a real machine: macOS Finder, Windows 11 Explorer, KDE Dolphin, GNOME Files, Cinnamon Nemo, Xfce Thunar

## M5½ · More of the everyday
- [x] Watermark add-on: a logo placed by anchor, sized to each image's shorter side, with opacity and margin
- [x] Rename from a list: `old, new` pairs pasted from a spreadsheet or loaded from CSV, or names in batch order

## M6 · Ship
- [x] Licence (AGPL-3.0-or-later), NOTICE, CLA, CONTRIBUTING, TRADEMARK, THIRD_PARTY
- [x] CI workflows: test on macOS, Linux, Windows. Release builds (one universal DMG, AppImage/deb/rpm, NSIS). CLI archives
- [x] First green CI run
- [x] First alpha release (0.1.0-alpha.1)
- [x] A download page that recognises the computer, and one-line installs for every system
- [ ] Signed, notarised macOS build and a signed Windows installer
- [ ] Homebrew tap, AUR package
- [x] README with screenshots

## Later
Watch folders, clipboard optimise, responsive image sets and `<picture>` snippets, ThumbHash placeholders, borders, colour adjust, local background removal (commercially licensed model only), JPEG XL encode, animated GIF/WebP processing.
