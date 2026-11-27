# Changelog

Notable changes, newest first. Versions follow [semantic versioning](https://semver.org).

## 0.1.0-alpha.5, 27 November 2026

Controls that look like what they are, names built from pieces, and recipes you can look after.

- **Keys, switches and selectors.** A key does something when you press it. A switch is on or off. A selector picks one of several, with one raised cap in a track. Every toggle and every choice in the app is now the right one of the three, so nothing that stays on looks like a button any more. Each screen has two rows of them beneath it, and the step keys carry a lamp that's lit while the step is on.
- **Size makes sense.** Type any size, or switch on "Enlarge smaller images" and drag the frame past the original. With it off, the screen says when an image is smaller and stays as it is. The big number says what it is (pixels on the long side). A "W:H" place on the shape selector takes any ratio in two fields, or pixels for an exact size, with a key to turn it on its side.
- **Names built from pieces.** The Name screen shows the new name as blocks: the old name, a number, a date, the size, the folder, your own text. Press a piece to add it, drag blocks to reorder them, drag one away to remove it, press one for its options (a number's digits, the date the photo was taken). Letter case is a selector under the screen.
- **Recipes you can look after.** The recipe list says what each recipe does, read from its settings. Rename, copy, delete, drag into order, and switch any of them on or off. Changes you haven't saved wait with their recipe while you look at another. Save keeps the changes, and Save as new names a new recipe right on the recipe screen.
- **A plainer starter set.** Compress for web, Just compress, Convert to JPEG, 1080p 16:9, Square 1080 and Small for email are on. Social card, Shopify product and Amazon main image come switched off. Web ready is now Compress for web, and Email safe is Small for email (`-r compress-for-web` on the command line).
- **One list for the menus.** The recipes that are on are what the right-click menu and the menu bar offer, in your order. The separate list of pinned recipes in Settings is gone.
- Images dropped on the hat in the menu bar open in Wrangler. Settings can have a drop run a recipe instead. The choice has left the hat's menu.
- Fixed in Finder: "Wrangler" showed twice for files on an external or network drive, the hat was squashed, and there was an empty row above "Open in Wrangler…".
- Settings uses switches, and says where to choose the menu's recipes.
- The self-test now uses the controls the way a person does (pressing, dragging and typing) and checks what happened, in Chromium and WebKit on every push.

## 0.1.0-alpha.4, 26 November 2026

Every setting in one place, Wrangler in Finder's own menu, and set up from the first launch.

- **One home for every setting.** The panel on the right is gone. Each screen's further settings now open right above it, the other steps (files, rotate, sharpen, watermark, metadata) are a row of keys, and Save's settings open from the Output panel. Nothing is set in two places any more: crop, pad and inside are only on the Size screen, the size limit is typed on the Quality screen, and the folder is only chosen with the Folder… key.
- **More room for the images.** The display runs the full width of the window, and the preview is about twice the size.
- **Wrangler in Finder's right-click menu.** A "Wrangler" entry with the hat, from a small Finder extension inside the app, in place of Quick Actions that macOS tucked away (and that macOS 26.1 sometimes didn't list at all). Quick Actions remain as the fallback when macOS won't load the extension.
- **Ready from the first launch.** Wrangler puts itself in the right-click menu of every file manager it finds. Settings takes it out again, and that sticks.
- Settings take effect as you press them. Pinning a recipe updates the menus at once, with no separate Update step.
- Add-ons switch on with one press of their key, and off from their settings. There's no Add step menu to go through.
- Simpler header. The hat is the mark, not a second Add button, Add offers images or a folder, and the command palette is gone: it repeated every visible control. "Save" is now "Save recipe", so it can't be mistaken for saving images.
- The before and after view is called Compare everywhere.
- Clear the batch from the top of the display.
- The menu bar menu reads better: "Run Web ready…", and "Images dropped on the hat run" for choosing the drop recipe.
- Right-click menus for two more Linux file managers: Caja (MATE), and PCManFM and PCManFM-Qt (LXDE and LXQt).
- Right-click menus list your recipes first and "Open in Wrangler…" last. Thunar, Nemo and PCManFM keep that order, and Dolphin shows Wrangler's icon beside the entry.
- Fixed: at the smallest window the preview could be blank. The Name screen overflowed in Replace mode. Crop, pad and inside spilled out of the Size screen in a small window.
- Fixed: starting in the tray, or running a recipe from a right-click menu, could bring up the main window as well. Seen on Linux.
- Fixed: in Files (Nautilus), choosing a recipe did nothing while Wrangler was already waiting in the tray. The same went for a file named from a terminal in another folder.
- Fixed: right-click menus set up from the AppImage only worked while Wrangler was running. They now start the AppImage itself.
- Fixed: the output meter could read −100% for a very large saving. It now stops at −99%, as the tiles do.
- Needs macOS 13 or later. The interface uses colour and layout features that the web view in macOS 12 and earlier can't draw, so earlier versions never looked right there.
- The self-test now opens every module's settings in each of its modes, which is how the three layout faults above were found.

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
