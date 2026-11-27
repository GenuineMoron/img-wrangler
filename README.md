<img src=".github/assets/hero.svg" width="1200" alt="Wrangler, a batch image unit. A real run plays on its screen: eight photos through the Compress for web recipe, 135 KB to 57 KB, 58 percent smaller." />

<p align="center">
  <b>Batch image processing for people with a few thousand photos and a deadline.</b><br />
  Resize, reshape, compress, convert, rename and watermark. macOS, Windows and Linux. Free and open source.
</p>

<p align="center">
  <a href="#see-every-file-before-anything-is-written">Demo</a> ·
  <a href="#what-it-does">What it does</a> ·
  <a href="#four-ways-in">Four ways in</a> ·
  <a href="#get-it">Get it</a> ·
  <a href="#licence">Licence</a>
</p>

<p align="center">
  <a href="https://github.com/GenuineMoron/img-wrangler/actions/workflows/ci.yml"><img src="https://github.com/GenuineMoron/img-wrangler/actions/workflows/ci.yml/badge.svg" alt="CI" /></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/licence-AGPL--3.0-ff5a1f?style=flat-square" alt="Licence: AGPL 3.0 or later" /></a>
  <img src="https://img.shields.io/badge/macOS%20·%20Windows%20·%20Linux-1f2023?style=flat-square" alt="Runs on macOS, Windows and Linux" />
  <img src="https://img.shields.io/badge/engine-Rust-3d5cff?style=flat-square" alt="Engine written in Rust" />
  <a href="https://github.com/GenuineMoron/img-wrangler/releases"><img src="https://img.shields.io/github/v/release/GenuineMoron/img-wrangler?include_prereleases&style=flat-square&color=18c06a&label=release" alt="Latest release" /></a>
</p>

Images never arrive the way you need them. A phone full of HEICs, a folder of 40 MB TIFFs from a photographer, a shoot named DSC_something, a pile of screenshots for a blog post. Too heavy, the wrong shape, named by a camera.

Wrangler sorts the lot in one pass. It shows every file as it will come out, new name, new size, new weight, before it writes a thing. Then you press Run. And if you change your mind afterwards, Undo puts the originals back.

Websites, online shops, email, social posts, slide decks, archives: whatever needs images in a particular size, shape, format or weight, a recipe does it the same way every time.

It all happens on your computer. No accounts, no uploads, no telemetry.

## See every file before anything is written

<img src=".github/assets/demo.webp" width="1080" alt="The Wrangler window: photos are added, the Small for email recipe is chosen from the list, a number is added to the names, the size frame is dragged smaller and Run processes every photo, each tile ticking off as it finishes." />

<sub>Recorded from the browser preview of the app, which plays sample photos with simulated sizes. The figures in the banner above and the right-click card are from a real run of the Compress for web recipe on eight sample photos.</sub>

## What it does

| | |
|---|---|
| Resize and reshape | A long side like 2048 (or the short side, width or height), or full size, and bigger than the original if you switch enlarging on. Keep each photo's shape, or make them all 1:1, 4:5, 3:2, 16:9, any ratio you type or exact pixels, by cropping, padding onto a backdrop or fitting inside. Empty background can be trimmed first. |
| Compress | Choose a look (small, balanced, high or pristine) instead of an encoder number. Each aims at a measured visual score ([SSIMULACRA 2](https://github.com/cloudinary/ssimulacra2)), and Wrangler finds the setting that reaches it for every image. Add a size limit if a platform has one. |
| Convert | Reads 14 formats, HEIC, AVIF and JPEG XL included. Writes JPEG (MozJPEG), PNG (oxipng), WebP, AVIF, GIF, TIFF, BMP, ICO, QOI, TGA and PNM. Or keeps each file's own format, or picks WebP or PNG per image. |
| Rename | Names built from pieces you drag into order: the old name, a number, a date, the size, the folder, your own text. Or find and replace, or old and new names pasted from a spreadsheet. Clashes get numbered rather than overwritten. |
| Watermark | A logo in the same place on every image, sized to each one so portraits and landscapes match. |
| Rotate and sharpen | Quarter turns and flips, and sharpening to bring back crispness after shrinking. |
| Metadata | Strip it all, keep the camera and copyright details without the location, or keep everything. |
| Save | Beside the originals, into a subfolder, into another folder, or over the originals. Every write goes to a temporary file first, and replaced originals sit in an undo store for 30 days. |

## Four ways in

### The window

Drop files or folders anywhere on it. The four screens along the bottom are the everyday controls, and each screen is the control itself: drag the corner of the size frame, drag the red line to set a size limit, pick the format from the list, build a name from blocks. The row of keys above them holds the other steps: which files to take, rotate, sharpen, watermark, metadata. Every setting has one home, and the rest of a control's settings open right above it.

Three kinds of control, and each looks like what it is. A key does something when you press it. A switch is on or off. A selector picks one of several.

Arrow through the images and each preview shows at once: a draft of the framing first, then the exact result with its real file size a moment later. The next image's preview is already being made while you look at this one. Right-click an image to open it, show it in its folder, compare before and after, or run the recipe on just that one.

### Right-click

<img src=".github/assets/right-click.svg" width="1200" alt="Eight photos selected in a file manager. Right-click, Wrangler, Web ready. A small Wrangler window runs the recipe: 135 KB to 57 KB, with Undo, Show and Open." />

Wrangler puts itself in the right-click menu of Finder, Explorer, Dolphin, Files (Nautilus), Nemo, Thunar, Caja, PCManFM and Luciole the first time it runs. Pick a recipe there and a small window runs it straight away, shows what was saved, and keeps Undo one press away. "Open in Wrangler…" loads the selection into the main window instead. The menu offers the recipes you have switched on, and Settings takes the menus out again. From a terminal it's `img-wrangler integrate install`.

In Finder it's a "Wrangler" entry in the menu itself, from a small Finder extension inside the app. If macOS won't load the extension, the recipes go under Finder's Quick Actions instead.

### The menu bar

The hat sits in the menu bar on a Mac, and in the system tray on Windows and Linux. Its menu runs a recipe on images you choose and takes back the last run, without opening the window. On a Mac, drop images straight on the hat and they open in Wrangler. Settings can have a drop run a recipe there and then instead, with the small window opening under the hat. Closing the main window leaves Wrangler there, and it can start at login.

### The command line

```sh
img-wrangler plan ~/Pictures/to-sort -r compress-for-web   # what would happen, nothing written
img-wrangler run ~/Pictures/to-sort -r compress-for-web    # do it
img-wrangler run ~/Pictures/to-sort --size 2048 --format webp --look balanced --to replace
img-wrangler run ~/Pictures/to-sort --names new-names.csv --watermark logo.png
img-wrangler undo                                          # put the last run back
```

`--json` turns the progress into JSON lines for scripts.

## Recipes

A recipe is every setting in one plain TOML file. The app, the command line and the right-click menu all read the same files, so a recipe you tune in the window runs exactly the same from a script. `img-wrangler recipes show compress-for-web` prints one to start your own from.

In the window, the recipe list says what each one does, read from its settings. Change the controls and press Save, or Save as new and type a name. Recipes can be renamed, copied, deleted, dragged into order and switched off. The ones that are on are what the window, the right-click menu and the menu bar offer.

These come with it. The first six start switched on, and the last three are there to switch on if you sell on those sites:

| Recipe | What it makes |
|---|---|
| Compress for web | 2560 px on the long side, WebP. Saved into a Wrangled folder. |
| Just compress | Same format and size, smaller files. Replaces the originals, with undo for 30 days. |
| Convert to JPEG | Same size, saved as JPEG. For HEIC, PNG, WebP and anything else that won't open. |
| 1080p, 16:9 | Exactly 1920 × 1080, cropped from the centre to fill, JPEG. |
| Square 1080 | Exactly 1080 × 1080, cropped from the centre to fill, JPEG. |
| Small for email | 1200 px on the long side, JPEG, under 200 KB each. |
| Social card | 1200 × 628 link previews, cropped from the centre, JPEG. |
| Shopify product | Trimmed, centred on a 2048 px square with the photo's own backdrop, WebP. |
| Amazon main image | Pure white 2000 px square, product filling 85%, JPEG. |

## Get it

**[Download Wrangler](https://genuinemoron.github.io/img-wrangler/)**. The download page works out which computer you're on and offers the right file, with every other system one tap away.

Or install it with one line, which works out the same thing:

```sh
# macOS and Linux
curl -fsSL https://genuinemoron.github.io/img-wrangler/install | sh
```

```powershell
# Windows (PowerShell)
irm https://raw.githubusercontent.com/GenuineMoron/img-wrangler/main/install.ps1 | iex
```

On a Mac it installs the one app that runs on Apple silicon and Intel alike. On Linux it uses your own package manager: a native package on Arch and CachyOS (through `makepkg`), the `.deb` on Debian and Ubuntu, the `.rpm` on Fedora and openSUSE, and the AppImage anywhere else. The scripts are short, so read them first if you like: [install.sh](install.sh), [install.ps1](install.ps1).

Every file is also on [Releases](https://github.com/GenuineMoron/img-wrangler/releases), with a table of which one you need. The builds aren't signed by Apple or Microsoft yet, so a downloaded app needs one extra click the first time. It's an alpha: try it on copies first, and [say what breaks](https://github.com/GenuineMoron/img-wrangler/issues).

From 0.1.0-alpha.3 on, Wrangler keeps itself up to date. It tells you when there's a new version and what changed, and installs it when you say so. Each update is signed, and the app checks the signature before installing anything. On Arch and CachyOS the package manager looks after updates instead, so the app gives you the command.

### Or build it

```sh
git clone https://github.com/GenuineMoron/img-wrangler && cd img-wrangler
cargo build --release -p img-wrangler        # the command line tool, needs only Rust

cd apps/desktop && pnpm install
pnpm tauri build                             # the app, needs Node 22 and pnpm too
```

On Linux the app also needs WebKitGTK and a few friends, listed in the [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/). `pnpm dev` runs the interface in a browser with sample photos, and `?demo` starts it with a shoot loaded.

## How it's made

The image work is a Rust engine (`crates/engine`): decoding, resizing, trimming, encoding, naming, the plan, the run and the undo journal. The command line tool and the desktop app are two thin front doors onto it. The app is [Tauri](https://tauri.app) with a [Svelte](https://svelte.dev) interface, drawn from one design system in `design/system`.

It's built to feel instant. Previews come as a quick draft and then the exact result, worked out ahead for the next image. Big batches skip deep reactivity, and progress is drawn once a frame. The case textures are baked into plain images rather than blended on every repaint, and on a computer drawing without a graphics card Wrangler measures that and turns the soft shadows and glows down by itself. `pnpm selftest` drives the real app through the everyday steps and times them, and checks every screen for cut-off or overlapping controls at three window sizes. CI runs the same checks in Chromium and WebKit on every push.

Every feature is a module, with an engine half and an interface half, each listed in one registry. Adding a step like the watermark took one Rust file, one interface folder and a line or two in each registry. [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) is the map and [docs/MODULES.md](docs/MODULES.md) walks through adding a feature. [DESIGN.md](DESIGN.md) covers how it looks and why.

## Help build it

Issues and pull requests are welcome. [CONTRIBUTING.md](CONTRIBUTING.md) explains the house style and how to run the tests. Contributions are accepted under the [contributor licence agreement](CLA.md), which keeps the project free for everyone and leaves room for a commercial licence later.

## Licence

Wrangler is free software under the [GNU AGPL 3.0 or later](LICENSE). Commercial licences are available from the author, see [NOTICE](NOTICE). The name and the hat are covered by the [trademark policy](TRADEMARK.md). Third-party work it builds on is listed in [THIRD_PARTY.md](THIRD_PARTY.md).

DSC_4471.JPG was never a name. It doesn't have to be one any more.
