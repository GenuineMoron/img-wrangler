
## Installing

The quickest way is one line in a terminal. It fetches this release and installs it properly for your system.

- **macOS and Linux:** `curl -fsSL https://raw.githubusercontent.com/GenuineMoron/img-wrangler/main/install.sh | sh`
- **Windows** (PowerShell): `irm https://raw.githubusercontent.com/GenuineMoron/img-wrangler/main/install.ps1 | iex`

Or download a file below:

- **macOS:** the `.dmg` for your Mac (`aarch64` for Apple silicon, `x64` for Intel). The app isn't notarised by Apple yet, so the first time you open it macOS refuses. Go to System Settings, Privacy & Security, scroll down and choose Open Anyway. You only do that once.
- **Windows:** the `-setup.exe`. If SmartScreen stops it, choose More info, then Run anyway.
- **Arch, CachyOS, EndeavourOS, Manjaro:** download `PKGBUILD` into an empty folder and run `makepkg -si`. It builds a native package that uses your system's WebKitGTK.
- **Debian, Ubuntu, Mint, Pop!_OS:** the `.deb`. **Fedora, openSUSE:** the `.rpm`.
- **Anything else:** the `.AppImage`, made executable with `chmod +x`. Some systems need FUSE 2 for AppImages (`libfuse2` or `fuse2`).
- **Command line only:** the `img-wrangler-…` archives hold just the CLI.

Undo keeps replaced originals for 30 days, but this is an alpha. Try it on copies before you point it at anything precious, and [say what breaks](https://github.com/GenuineMoron/img-wrangler/issues).
