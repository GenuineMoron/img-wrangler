# Plan

Milestones in build order. Each one ends working and tested, so the project is usable at every step.

## M1 · Engine foundation
- [ ] Workspace: `crates/engine`, `crates/integrate`, `crates/cli`, `apps/desktop`
- [ ] Image model, decode (all formats incl. AVIF, JPEG XL, SVG, HEIC via OS), orientation, sRGB conversion
- [ ] Format registry and encoders: JPEG (MozJPEG), PNG (oxipng, optional palette), WebP, AVIF, GIF, TIFF, BMP, ICO, QOI
- [ ] Ops: resize, trim detection, frame layout, pad, crop, rotate, sharpen
- [ ] Modules: size (core), rotate, sharpen (add-ons); stage pipeline
- [ ] Quality: look targeting (SSIMULACRA 2), size limit, never bigger
- [ ] Naming: patterns, replace, case, collisions
- [ ] Recipe TOML with defaults, validation, built-in presets

## M2 · Batch engine
- [ ] Scan, plan (instant, no pixels), estimate (sampled), run (parallel, memory budget, cancel, progress)
- [ ] Atomic writes, destinations (beside, subfolder, folder, replace), keep dates
- [ ] Journal and undo with retention
- [ ] Thumbnail cache

## M3 · Command line
- [ ] `img-wrangler run | plan | recipes | undo | history | info | integrate`
- [ ] Flags for common settings layered on a recipe; JSON output for scripts

## M4 · Desktop app
- [ ] Tauri host: commands, events, `thumb://` and `preview://`, single instance, window state
- [ ] Svelte front end: design system, generic components, module registry, stores, typed IPC, mock backend
- [ ] Main window: header, display (sheet, monitor, tabs, hints), channel strip, recipe panel, output panel
- [ ] Keyboard model, command palette, accessibility pass
- [ ] Quick-action window

## M5 · Lives where the files are
- [ ] Finder Quick Actions, Windows Explorer verbs, Dolphin / Nautilus / Nemo / Thunar actions
- [ ] Install, update and remove from Settings and from the CLI
- [ ] Open With and drag onto the Dock / taskbar icon

## M6 · Ship
- [ ] Licence (AGPL-3.0-or-later), NOTICE, CLA, CONTRIBUTING, TRADEMARK, THIRD_PARTY
- [ ] CI: test on macOS, Linux, Windows; release builds (DMG, AppImage/deb/rpm, MSI/NSIS); CLI archives
- [ ] Homebrew tap, AUR package
- [ ] README with screenshots

## Later
Watch folders, clipboard optimise, responsive image sets and `<picture>` snippets, ThumbHash placeholders, CSV rename mapping, watermark, borders, colour adjust, local background removal (commercially licensed model only), JPEG XL encode, animated GIF/WebP processing.
