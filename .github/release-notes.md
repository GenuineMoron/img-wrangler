
## Installing

These builds aren't signed yet, so your computer will be suspicious of them. Fair enough.

- **macOS:** take the `.dmg` for your Mac (`aarch64` for Apple silicon, `x64` for Intel) and drag Wrangler to Applications. macOS calls unsigned downloads "damaged", so run `xattr -cr /Applications/Wrangler.app` once in Terminal before opening it.
- **Windows:** run the `-setup.exe`. If SmartScreen stops it, choose More info, then Run anyway.
- **Linux:** the `.AppImage` runs anywhere once it's executable (`chmod +x`), or install the `.deb` or `.rpm`.
- **Command line only:** the `img-wrangler-…` archives hold just the CLI.

Undo keeps replaced originals for 30 days, but this is an alpha. Try it on copies before you point it at anything precious, and [say what breaks](https://github.com/GenuineMoron/img-wrangler/issues).
