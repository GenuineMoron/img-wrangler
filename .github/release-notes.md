
## Installing

**The easy way:** [the download page](https://genuinemoron.github.io/img-wrangler/) works out which computer you're on and offers the right file. Or use one line in a terminal, which does the same and installs it properly:

- **macOS and Linux:** `curl -fsSL https://genuinemoron.github.io/img-wrangler/install | sh`
- **Windows** (PowerShell): `irm https://raw.githubusercontent.com/GenuineMoron/img-wrangler/main/install.ps1 | iex`

### Which file do I need?

| Your computer | The file |
|---|---|
| Any Mac, Apple silicon or Intel, on macOS 13 or later | `Wrangler_…_universal.dmg` |
| Windows 10 or 11 | `Wrangler_…_x64-setup.exe` |
| Arch, CachyOS, EndeavourOS, Manjaro | `PKGBUILD`, then `makepkg -si` in the same folder |
| Debian, Ubuntu, Mint, Pop!_OS | `…_amd64.deb`, or `…_arm64.deb` on ARM |
| Fedora, openSUSE | `….x86_64.rpm`, or `….aarch64.rpm` on ARM |
| Any other Linux | `…_amd64.AppImage`, or `…_aarch64.AppImage` on ARM |
| Just the command line tool | `img-wrangler-…` for your system |

`latest.json` and `Wrangler_universal.app.tar.gz` are for the app's own updates: you don't need to download them. From 0.1.0-alpha.3 on, Wrangler tells you about new versions and installs them when you say so.

### The first time you open it

The builds aren't signed by Apple or Microsoft yet, so the first launch takes one extra step:

- **macOS** refuses to open a downloaded copy. Go to System Settings, then Privacy & Security, scroll down and choose Open Anyway. You only do that once. The one-line install skips this step. It's also the surest way to get the "Wrangler" entry in Finder's right-click menu: some Macs won't load a Finder extension from a copy they had to be told to open, and Wrangler then puts its recipes under Finder's Quick Actions instead.
- **Windows** SmartScreen may stop the installer. Choose More info, then Run anyway.
- **AppImage:** make it executable with `chmod +x`. Some systems need FUSE 2 for AppImages (`libfuse2` or `fuse2`).

Undo keeps replaced originals for 30 days, but this is an alpha. Try it on copies before you point it at anything precious, and [say what breaks](https://github.com/GenuineMoron/img-wrangler/issues).
