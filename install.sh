#!/bin/sh
# Installs Wrangler on macOS or Linux, from the newest release (alphas included):
#
#   curl -fsSL https://genuinemoron.github.io/img-wrangler/install | sh
#
# It only downloads from this repository's releases. On macOS it copies Wrangler.app into
# Applications. On Linux it uses your package manager (pacman via makepkg, apt, dnf or zypper),
# which asks for your password itself, or else puts the AppImage in ~/.local/bin.
set -eu

repo=GenuineMoron/img-wrangler
say() { printf '%s\n' "$*"; }
fail() { say "wrangler: $*" >&2; exit 1; }
has() { command -v "$1" >/dev/null 2>&1; }

has curl || fail "needs curl"
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT

assets=$(curl -fsSL "https://api.github.com/repos/$repo/releases?per_page=1" | grep -o '"browser_download_url": *"[^"]*"' | cut -d'"' -f4) || true
[ -n "$assets" ] || fail "couldn't find a release on github.com/$repo"

# fetch PATTERN: download the newest release's file matching PATTERN into $tmp, set $file
fetch() {
  url=$(printf '%s\n' "$assets" | grep -E "$1" | head -n 1)
  [ -n "$url" ] || fail "the newest release has no file matching $1"
  file="$tmp/${url##*/}"
  say "Downloading ${url##*/}"
  curl -fL --progress-bar -o "$file" "$url"
}

case "$(uname -s)" in
  Darwin)
    # one universal .dmg runs on every Mac. Releases before 0.1.0-alpha.2 had one per chip.
    if printf '%s\n' "$assets" | grep -q '_universal\.dmg$'; then
      fetch '_universal\.dmg$'
    else
      case "$(uname -m)" in arm64) arch=aarch64 ;; *) arch=x64 ;; esac
      fetch "_${arch}\\.dmg$"
    fi
    mkdir "$tmp/dmg"
    hdiutil attach -nobrowse -quiet -mountpoint "$tmp/dmg" "$file"
    dest=/Applications
    [ -w "$dest" ] || dest="$HOME/Applications"
    mkdir -p "$dest"
    rm -rf "$dest/Wrangler.app"
    cp -R "$tmp/dmg/Wrangler.app" "$dest/"
    hdiutil detach -quiet "$tmp/dmg"
    say "Wrangler is in $dest. Open it from there, or from Spotlight."
    ;;
  Linux)
    case "$(uname -m)" in
      x86_64 | amd64) deb=amd64 rpm=x86_64 img=amd64 ;;
      aarch64 | arm64) deb=arm64 rpm=aarch64 img=aarch64 ;;
      *) fail "there's no Linux build for $(uname -m) yet" ;;
    esac
    if has pacman && has makepkg; then
      fetch "/PKGBUILD$"
      (cd "$tmp" && makepkg -si --noconfirm)
    elif has apt-get; then
      fetch "_${deb}\\.deb$"
      sudo apt-get install -y "$file"
    elif has dnf; then
      fetch "\\.${rpm}\\.rpm$"
      sudo dnf install -y "$file"
    elif has zypper; then
      fetch "\\.${rpm}\\.rpm$"
      sudo zypper --non-interactive install --allow-unsigned-rpm "$file"
    else
      fetch "_${img}\\.AppImage$"
      mkdir -p "$HOME/.local/bin" "$HOME/.local/share/applications"
      install -m 755 "$file" "$HOME/.local/bin/img-wrangler-app"
      cat > "$HOME/.local/share/applications/wrangler.desktop" <<DESKTOP
[Desktop Entry]
Type=Application
Name=Wrangler
Comment=Batch image processing
Exec=$HOME/.local/bin/img-wrangler-app %F
Categories=Graphics;
MimeType=image/jpeg;image/png;image/webp;image/avif;image/heic;image/tiff;image/gif;
DESKTOP
      say "Installed to ~/.local/bin/img-wrangler-app. Some systems need FUSE 2 to run AppImages (libfuse2 or fuse2)."
    fi
    say "Wrangler is installed. Find it in your app launcher."
    ;;
  *) fail "this script is for macOS and Linux. For Windows, see https://genuinemoron.github.io/img-wrangler" ;;
esac
