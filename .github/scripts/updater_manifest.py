#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Writes latest.json, the file the app reads to find updates, from a release's signatures.

    updater_manifest.py <tag> <notes.md> <folder of .sig files>  >  latest.json

Each build's .sig names the file it signs. The app picks its entry by system and processor,
and by how it was installed when that matters (a .deb updates with a .deb, an AppImage with an
AppImage). Signatures go into the manifest itself, so the release needn't list them."""

import json
import re
import sys
from datetime import datetime, timezone
from pathlib import Path

REPO = "GenuineMoron/img-wrangler"

# (pattern on the signed file's name, the platform keys it serves)
PLATFORMS = [
    (r"universal\.app\.tar\.gz$", ["darwin-aarch64", "darwin-x86_64", "darwin-aarch64-app", "darwin-x86_64-app"]),
    (r"aarch64\.app\.tar\.gz$", ["darwin-aarch64", "darwin-aarch64-app"]),
    (r"x64\.app\.tar\.gz$", ["darwin-x86_64", "darwin-x86_64-app"]),
    (r"_x64-setup\.exe$", ["windows-x86_64", "windows-x86_64-nsis"]),
    (r"_amd64\.AppImage$", ["linux-x86_64", "linux-x86_64-appimage"]),
    (r"_aarch64\.AppImage$", ["linux-aarch64", "linux-aarch64-appimage"]),
    (r"_amd64\.deb$", ["linux-x86_64-deb"]),
    (r"_arm64\.deb$", ["linux-aarch64-deb"]),
    (r"\.x86_64\.rpm$", ["linux-x86_64-rpm"]),
    (r"\.aarch64\.rpm$", ["linux-aarch64-rpm"]),
]


def manifest(tag: str, notes: str, sigs: Path) -> dict:
    platforms = {}
    for sig in sorted(sigs.glob("*.sig")):
        asset = sig.name[: -len(".sig")]
        for pattern, keys in PLATFORMS:
            if re.search(pattern, asset):
                entry = {"signature": sig.read_text().strip(), "url": f"https://github.com/{REPO}/releases/download/{tag}/{asset}"}
                for k in keys:
                    platforms.setdefault(k, entry)
                break
    if not platforms:
        sys.exit(f"no signed builds in {sigs}")
    return {"version": tag.removeprefix("v"), "notes": notes.strip(), "pub_date": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"), "platforms": platforms}


if __name__ == "__main__":
    tag, notes, sigs = sys.argv[1], Path(sys.argv[2]).read_text(), Path(sys.argv[3])
    json.dump(manifest(tag, notes, sigs), sys.stdout, indent=2)
    print()
