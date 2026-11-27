// SPDX-License-Identifier: AGPL-3.0-or-later
// Runs Wrangler's self-test (src/lib/selftest) and prints what it measured.
//
//   pnpm selftest app [path/to/img-wrangler-app] [--images DIR] [--photos 120]
//       The real app on this machine: it drives itself through the everyday steps on a folder of
//       photos, checks its layout at three window sizes, writes a report and quits. Without
//       --images it makes a folder of camera-sized test photos with the CLI.
//   pnpm selftest browser [--files 1000] [--engines chromium,webkit]
//       The interface in a browser with the stand-in backend: speed with thousands of files, the
//       layout at three viewport sizes, screenshots of every state for a look, and then the
//       controls used the way a person uses them (scripts/interact.mjs).
//   pnpm selftest compare before.json after.json
//       Two reports side by side.
//
// --strict makes app and browser runs fail when the layout has a fault (CI uses it).
//
// Reports and screenshots go to --out (default: a selftest folder in the system temp folder).
// Browsers need Chromium (CHROME_PATH or `npx playwright install chromium`) and, for WebKit,
// Playwright's WebKit build.

import { execFileSync, spawn } from 'node:child_process';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { interact } from './interact.mjs';

const APP = path.dirname(path.dirname(fileURLToPath(import.meta.url)));
const ROOT = path.resolve(APP, '../..');
const [mode = 'browser', ...rest] = process.argv.slice(2);
const flag = (name, fallback) => {
  const i = rest.indexOf(`--${name}`);
  return i >= 0 ? rest[i + 1] : fallback;
};
const OUT = path.resolve(flag('out', path.join(os.tmpdir(), 'wrangler-selftest')));
fs.mkdirSync(OUT, { recursive: true });
const stamp = new Date().toISOString().replace(/[:.]/g, '-').slice(0, 19);

// ---------------------------------------------------------------- printing

const pad = (s, n) => String(s).padEnd(n);
const lpad = (s, n) => String(s).padStart(n);

function print(report, title) {
  const host = Object.entries(report.host ?? {}).filter(([, v]) => v).map(([k, v]) => `${k} ${v}`).join(' · ');
  console.log(`\n${title}\n${report.engine}${host ? `\n${host}` : ''}${report.effects ? `\neffects ${report.effects}` : ''}\n${report.files} files · window ${report.window.w}×${report.window.h} @${report.window.dpr}x · ready in ${report.boot_ms} ms\n`);
  console.log(`${pad('step', 46)}${lpad('ms', 9)}   frames p50 / p95 / worst   slow`);
  for (const r of report.results) {
    const f = r.frames ? `${lpad(r.frames.p50, 6)} / ${lpad(r.frames.p95, 6)} / ${lpad(r.frames.max, 6)}   ${lpad(`${r.frames.slow}/${r.frames.count}`, 7)}` : '';
    console.log(`${pad(r.name, 46)}${lpad(r.ms ?? '', 9)}   ${f}${r.note ? `   ${r.note}` : ''}`);
  }
  printLayout(report.layout ?? []);
}

/** Layout faults that count: everything but long file names shortened with an ellipsis. */
const faults = issues => issues.filter(i => i.kind !== 'truncated' || i.text.length < 24);

function printLayout(issues) {
  const real = faults(issues);
  if (real.length && rest.includes('--strict')) process.exitCode = 1;
  console.log(`\nlayout: ${real.length} issue${real.length === 1 ? '' : 's'}${issues.length > real.length ? ` (${issues.length - real.length} long names shortened with an ellipsis, not counted)` : ''}`);
  for (const i of real) console.log(`  ${pad(i.kind, 10)} ${pad(i.size, 10)} ${pad(i.state, 18)} ${lpad(`${i.by}px`, 6)}  "${i.text}"  ${i.where}`);
}

function compare(a, b) {
  console.log(`\n${pad('step', 46)}${lpad('before', 10)}${lpad('after', 10)}${lpad('change', 9)}`);
  for (const r of b.results) {
    const o = a.results.find(x => x.name === r.name);
    const val = x => (x?.frames ? x.frames.p95 : x?.ms);
    const [va, vb] = [val(o), val(r)];
    const unit = r.frames ? ' p95' : '';
    const ch = va && vb ? `${Math.round(((vb - va) / va) * 100)}%` : '';
    console.log(`${pad(r.name + unit, 46)}${lpad(va ?? '', 10)}${lpad(vb ?? '', 10)}${lpad(ch, 9)}`);
  }
  console.log(`\nlayout issues: ${a.layout?.length ?? 0} → ${b.layout?.length ?? 0}`);
}

// ---------------------------------------------------------------- the real app

function testPhotos(count) {
  const dir = path.join(os.homedir(), '.cache', 'wrangler-selftest', `photos-${count}`);
  if (fs.existsSync(dir) && fs.readdirSync(dir).length >= count) return dir;
  const cli = path.join(ROOT, 'target/release', process.platform === 'win32' ? 'img-wrangler.exe' : 'img-wrangler');
  if (!fs.existsSync(cli)) execFileSync('cargo', ['build', '--release', '-p', 'img-wrangler'], { cwd: ROOT, stdio: 'inherit' });
  const big = fs.mkdtempSync(path.join(os.tmpdir(), 'wrangler-photos-'));
  // The samples, made camera-sized: 4000 px JPEGs, about what a phone or a small camera makes.
  execFileSync(cli, ['run', path.join(APP, 'dev/samples'), '--size', '4000', '--upscale', '--shape', '4:3', '--fit', 'crop', '--format', 'jpg', '--look', 'pristine', '--to', big], { stdio: 'ignore' });
  const made = fs.readdirSync(big).filter(f => f.endsWith('.jpg'));
  fs.mkdirSync(dir, { recursive: true });
  for (let i = 0; i < count; i++) fs.copyFileSync(path.join(big, made[i % made.length]), path.join(dir, `DSC_${String(1000 + i)}.jpg`));
  fs.rmSync(big, { recursive: true, force: true });
  return dir;
}

function defaultApp() {
  const names = { darwin: 'bundle/macos/Wrangler.app/Contents/MacOS/img-wrangler-app', win32: 'img-wrangler-app.exe' };
  return path.join(ROOT, 'target/release', names[process.platform] ?? 'img-wrangler-app');
}

async function runApp() {
  const bin = rest[0] && !rest[0].startsWith('--') ? path.resolve(rest[0]) : defaultApp();
  if (!fs.existsSync(bin)) throw new Error(`no app at ${bin}; build it with pnpm tauri build, or pass its path`);
  const source = flag('images') ? path.resolve(flag('images')) : testPhotos(Number(flag('photos', 120)));
  // A fresh folder each time (hard links, so it's instant), because the run writes beside them.
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'wrangler-batch-'));
  for (const f of fs.readdirSync(source)) {
    const from = path.join(source, f);
    if (fs.statSync(from).isFile()) {
      try { fs.linkSync(from, path.join(dir, f)); } catch { fs.copyFileSync(from, path.join(dir, f)); }
    }
  }
  const report = path.join(OUT, `app-${process.platform}-${stamp}${flag('fx') ? `-fx-${flag('fx').replace(/,/g, '+')}` : ''}${flag('effects') ? `-${flag('effects')}` : ''}.json`);
  const extra = rest.filter(a => ['--no-measure', '--no-run', '--no-layout'].includes(a));
  if (flag('fx')) extra.push('--fx', flag('fx'));
  if (flag('effects')) extra.push('--effects', flag('effects'));
  if (flag('update')) extra.push('--update', flag('update'));
  console.log(`running ${bin}\non ${fs.readdirSync(dir).length} photos in ${dir}`);
  // On a Mac, start it through Launch Services so it opens in the desktop session even when
  // this script runs over SSH.
  const bundle = process.platform === 'darwin' && bin.match(/^(.*\.app)\/Contents\/MacOS\//)?.[1];
  const [cmd, args] = bundle ? ['open', ['-n', '-W', bundle, '--args', 'selftest', dir, report, ...extra]] : [bin, ['selftest', dir, report, ...extra]];
  // A Mac stops drawing a window nobody can see, so wake the display (-u) and keep it awake.
  const awake = process.platform === 'darwin' ? spawn('caffeinate', ['-d', '-i', '-u'], { stdio: 'ignore' }) : null;
  await new Promise((resolve, reject) => {
    const child = spawn(cmd, args, { stdio: 'inherit' });
    const timer = setTimeout(() => { child.kill(); reject(new Error('the self-test took over 15 minutes')); }, 15 * 60 * 1000);
    child.on('exit', () => { clearTimeout(timer); awake?.kill(); resolve(); });
    child.on('error', reject);
  });
  fs.rmSync(dir, { recursive: true, force: true });
  if (!fs.existsSync(report)) throw new Error('the app quit without writing a report');
  print(JSON.parse(fs.readFileSync(report, 'utf8')), `app on ${process.platform}`);
  console.log(`\nreport: ${report}`);
}

// ---------------------------------------------------------------- the browser

async function runBrowser() {
  const { createServer } = await import('vite');
  const pw = await import('playwright-core');
  const files = Number(flag('files', 1000));
  const engines = flag('engines', 'chromium').split(',');
  const server = await createServer({ root: APP, configFile: path.join(APP, 'vite.config.ts'), server: { port: 1431, strictPort: true, host: '127.0.0.1' }, logLevel: 'error' });
  await server.listen();
  const base = 'http://127.0.0.1:1431';
  try {
    for (const name of engines) {
      const browser = await pw[name].launch(name === 'chromium' ? { executablePath: process.env.CHROME_PATH || undefined, args: ['--no-sandbox'] } : {});
      try {
        // Speed, with a big batch.
        const page = await browser.newPage({ viewport: { width: 1440, height: 900 } });
        page.on('pageerror', e => console.log(`  page error: ${e.message}`));
        await page.goto(`${base}/?selftest=${files}`);
        const report = await page.waitForFunction(() => window.__selftest, null, { timeout: 15 * 60 * 1000, polling: 500 }).then(h => h.jsonValue());
        await page.close();

        // Layout at each size, and a screenshot of every state at the middle size.
        const shots = path.join(OUT, `${name}-${stamp}`);
        fs.mkdirSync(shots, { recursive: true });
        for (const [w, h] of [[1200, 760], [1440, 900], [1920, 1080]]) {
          const p = await browser.newPage({ viewport: { width: w, height: h } });
          await p.goto(`${base}/?selftest=layout`);
          await p.waitForFunction(() => window.__selftest, null, { timeout: 60000 });
          await p.waitForTimeout(600);
          await p.screenshot({ path: path.join(shots, `main-${w}x${h}.png`) });
          report.layout.push(...(await p.evaluate(() => window.__wrangler.layoutHere())));
          if (w === 1440) {
            // Each module's further settings, opened from its own control.
            const states = await p.evaluate(() => [...document.querySelectorAll('[id^="more-"]')].map(e => e.id.slice(5)));
            for (const s of states) {
              await p.click(`#more-${s}`);
              await p.waitForTimeout(300);
              await p.screenshot({ path: path.join(shots, `more-${s}.png`) });
              await p.keyboard.press('Escape');
            }
            await p.evaluate(() => (window.__wrangler.ui.more = null));
            for (const d of ['settings', 'history', 'recipes']) {
              await p.evaluate(id => (window.__wrangler.ui.dialog = id), d);
              await p.waitForTimeout(300);
              await p.screenshot({ path: path.join(shots, `dialog-${d}.png`) });
              await p.evaluate(() => (window.__wrangler.ui.dialog = null));
            }
          }
          await p.close();
        }
        // Then use it: press, drag and type, and check what happened.
        const used = await interact(browser, base);
        report.interactions = used;
        const file = path.join(OUT, `${name}-${stamp}.json`);
        fs.writeFileSync(file, JSON.stringify(report, null, 2));
        print(report, `${name} (browser preview)`);
        console.log(`\nusing it: ${used.count - used.failed.length} of ${used.count} checks passed`);
        for (const f of used.failed) console.log(`  FAILED  ${f}`);
        if (used.failed.length) process.exitCode = 1;
        console.log(`\nreport: ${file}\nscreenshots: ${shots}`);
      } finally {
        await browser.close();
      }
    }
  } finally {
    await server.close();
  }
}

// ----------------------------------------------------------------

if (mode === 'compare') {
  const [a, b] = rest.filter(x => !x.startsWith('--')).map(f => JSON.parse(fs.readFileSync(f, 'utf8')));
  compare(a, b);
} else if (mode === 'app') {
  await runApp();
} else {
  await runBrowser();
}
