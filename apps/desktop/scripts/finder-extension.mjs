// SPDX-License-Identifier: AGPL-3.0-or-later
// Builds Wrangler's Finder extension (src-tauri/finder) into src-tauri/gen/finder, where the
// bundle step picks it up (bundle.macOS.files in tauri.conf.json). Tauri runs this before
// bundling. On Linux and Windows it does nothing. With --test it checks the extension's menu
// and request logic instead (pnpm finder:test).
//
// No Xcode project: swiftc compiles the one source file for Apple silicon and Intel, lipo joins
// them, and codesign signs the result with the sandbox entitlement Finder insists on. The app's
// own signature, made afterwards, seals it in. Needs the Xcode command line tools.

import { execFileSync } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

if (process.platform !== 'darwin') process.exit(0);

const TAURI = path.join(path.dirname(path.dirname(fileURLToPath(import.meta.url))), 'src-tauri');
const SRC = path.join(TAURI, 'finder');
const OUT = path.join(TAURI, 'gen/finder');
const conf = JSON.parse(fs.readFileSync(path.join(TAURI, 'tauri.conf.json'), 'utf8'));
const minMacos = conf.bundle.macOS.minimumSystemVersion;
const run = (cmd, args) => execFileSync(cmd, args, { stdio: ['ignore', 'inherit', 'inherit'] });

// --test: check the menu and request logic without Finder, then stop.
if (process.argv.includes('--test')) {
  fs.mkdirSync(OUT, { recursive: true });
  const tests = path.join(OUT, 'finder-tests');
  run('xcrun', ['swiftc', '-parse-as-library', '-module-name', 'WranglerFinder', '-framework', 'Cocoa', '-framework', 'FinderSync',
    path.join(SRC, 'FinderSync.swift'), path.join(SRC, 'FinderSyncTests.swift'), '-o', tests]);
  run(tests, []);
  fs.rmSync(tests);
  process.exit(0);
}

const appex = path.join(OUT, 'WranglerFinder.appex');
fs.rmSync(OUT, { recursive: true, force: true });
fs.mkdirSync(path.join(appex, 'Contents/MacOS'), { recursive: true });
fs.mkdirSync(path.join(appex, 'Contents/Resources'), { recursive: true });

const slices = ['arm64', 'x86_64'].map(arch => {
  const out = path.join(OUT, `WranglerFinder-${arch}`);
  run('xcrun', [
    'swiftc', '-O', '-parse-as-library', '-application-extension', '-module-name', 'WranglerFinder',
    '-target', `${arch}-apple-macos${minMacos}`, '-framework', 'Cocoa', '-framework', 'FinderSync',
    // An extension has no main of its own: the system's NSExtensionMain starts it.
    '-Xlinker', '-e', '-Xlinker', '_NSExtensionMain',
    path.join(SRC, 'FinderSync.swift'), '-o', out,
  ]);
  return out;
});
run('lipo', ['-create', ...slices, '-output', path.join(appex, 'Contents/MacOS/WranglerFinder')]);
for (const s of slices) fs.rmSync(s);

const plist = fs.readFileSync(path.join(SRC, 'Info.plist.in'), 'utf8')
  .replaceAll('@APP_ID@', conf.identifier).replaceAll('@VERSION@', conf.version).replaceAll('@MIN_MACOS@', minMacos);
fs.writeFileSync(path.join(appex, 'Contents/Info.plist'), plist);
// The hat for the menu, the same one the menu bar shows.
fs.copyFileSync(path.join(TAURI, 'icons/tray/menubar.png'), path.join(appex, 'Contents/Resources/hat.png'));

const identity = process.env.APPLE_SIGNING_IDENTITY || conf.bundle.macOS.signingIdentity || '-';
run('codesign', ['--force', '--sign', identity, '--entitlements', path.join(SRC, 'finder.entitlements'), ...(identity === '-' ? [] : ['--options', 'runtime', '--timestamp']), appex]);
console.log(`Finder extension: ${path.relative(process.cwd(), appex)}`);
