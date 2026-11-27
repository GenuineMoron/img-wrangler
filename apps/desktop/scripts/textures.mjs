// SPDX-License-Identifier: AGPL-3.0-or-later
// Bakes the material textures, so the interface draws them as plain images instead of blending
// layers on every repaint (which some webviews, WebKitGTK without a GPU above all, do slowly).
//
//   pnpm textures
//
// The case: the scratch and dust textures (design/system/tex) soft-light blended over the case
// colour, rendered by the browser exactly as CSS would, into one tile (tex/case.webp).
// The fine grain, on the case and on the keys: an overlay that needs no blending
// (tex/grain-*.png), worked out with the soft-light formula for the case and for a light, a mid
// and a dark keycap. Kept apart from the case tile because compression would smooth it away.
// The screens' pixel grid: one tile of --px-pitch with the grid lines at --px-grid strength
// (tex/pixel-grid.png), instead of two repeating gradients redrawn on every repaint.
// Run it again after changing a surface colour, a texture or the pixel grid.

import { execFileSync } from 'node:child_process';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const APP = path.dirname(path.dirname(fileURLToPath(import.meta.url)));
const ROOT = path.resolve(APP, '../..');
const SYSTEM = path.join(ROOT, 'design/system');
const TEX = path.join(SYSTEM, 'tex');
const TMP = fs.mkdtempSync(path.join(os.tmpdir(), 'wrangler-tex-'));

// The tile repeats every 1920 px, so every layer's size has to divide it.
const TILE = 1920;
const tokens = fs.readFileSync(path.join(SYSTEM, 'tokens.css'), 'utf8');
const token = name => tokens.match(new RegExp(`--${name}:\\s*([^;]+);`))?.[1].trim();
const sizes = { fine: parseInt(token('tex-fine-size')), dust: parseInt(token('tex-dust-size')), scratch: parseInt(token('tex-scratch-size')) };
for (const [k, v] of Object.entries(sizes)) if (TILE % v) throw new Error(`--tex-${k}-size (${v}px) must divide ${TILE}px for the baked tile to repeat cleanly`);

const cli = path.join(ROOT, 'target/release', process.platform === 'win32' ? 'img-wrangler.exe' : 'img-wrangler');
if (!fs.existsSync(cli)) execFileSync('cargo', ['build', '--release', '-p', 'img-wrangler'], { cwd: ROOT, stdio: 'inherit' });

const { chromium } = await import('playwright-core');
const browser = await chromium.launch({ executablePath: process.env.CHROME_PATH || undefined, args: ['--no-sandbox'] });
try {
  const page = await browser.newPage({ viewport: { width: TILE, height: TILE }, deviceScaleFactor: 1 });
  // As data URLs: a blank page may not read local files.
  const url = f => `data:${f.endsWith('.png') ? 'image/png' : 'image/jpeg'};base64,${fs.readFileSync(path.join(TEX, f)).toString('base64')}`;
  await page.setContent(`<!doctype html><style>html,body{margin:0}</style>
    <div id="case" style="width:${TILE}px;height:${TILE}px;background-color:${token('surface-case')};
      background-image:url(${url('dust-l.jpg')}),url(${url('scratch-l.jpg')});
      background-size:${sizes.dust}px,${sizes.scratch}px;background-blend-mode:soft-light,soft-light"></div>
    <canvas id="c"></canvas>`);
  await page.waitForLoadState('networkidle');
  await page.locator('#case').screenshot({ path: path.join(TMP, 'case.png') });
  // How light the case is, for its grain. Painted into a pixel and read back, as the colour may be
  // written in any CSS colour space.
  const caseLight = await page.evaluate(() => {
    const c = document.createElement('canvas');
    c.width = c.height = 1;
    const g = c.getContext('2d');
    g.fillStyle = getComputedStyle(document.getElementById('case')).backgroundColor;
    g.fillRect(0, 0, 1, 1);
    const [r, gr, b] = g.getImageData(0, 0, 1, 1).data;
    return (0.2126 * r + 0.7152 * gr + 0.0722 * b) / 255;
  });

  // Grain overlays: for a keycap of brightness b, the soft-light result of the grain, written as
  // white or black at the alpha that gives the same brightness when laid over b normally. The
  // coloured keys (mid) get a gentler grain: on a strong colour it read as noise.
  const grains = await page.evaluate(async ({ src, bases, strength }) => {
    const img = new Image();
    img.src = src;
    await img.decode();
    const c = document.getElementById('c');
    c.width = img.width;
    c.height = img.height;
    const g = c.getContext('2d');
    const out = {};
    for (const [name, b] of Object.entries(bases)) {
      g.clearRect(0, 0, c.width, c.height);
      g.drawImage(img, 0, 0);
      const data = g.getImageData(0, 0, c.width, c.height);
      const d = data.data;
      const D = x => (x <= 0.25 ? ((16 * x - 12) * x + 4) * x : Math.sqrt(x));
      for (let i = 0; i < d.length; i += 4) {
        const s = d[i] / 255;
        const r = s <= 0.5 ? b - (1 - 2 * s) * b * (1 - b) : b + (2 * s - 1) * (D(b) - b);
        const lighter = r >= b;
        const a = lighter ? (r - b) / (1 - b) : (b - r) / b;
        d[i] = d[i + 1] = d[i + 2] = lighter ? 255 : 0;
        d[i + 3] = Math.round(Math.min(1, a) * (strength[name] ?? 1) * 255);
      }
      g.putImageData(data, 0, 0);
      out[name] = c.toDataURL('image/png');
    }
    return out;
  }, { src: url('fine.png'), bases: { case: caseLight, light: 0.96, mid: 0.5, dark: 0.19 }, strength: { mid: 0.45 } });
  for (const [name, dataUrl] of Object.entries(grains)) {
    fs.writeFileSync(path.join(TEX, `grain-${name}.png`), Buffer.from(dataUrl.split(',')[1], 'base64'));
  }

  // The pixel grid: a dark row along the top and a lighter column down the left of each cell.
  const grid = await page.evaluate(({ pitch, strength }) => {
    const c = document.getElementById('c');
    c.width = c.height = pitch;
    const g = c.getContext('2d');
    g.clearRect(0, 0, pitch, pitch);
    g.fillStyle = `rgba(0,0,0,${strength})`;
    g.fillRect(0, 0, pitch, 1);
    g.fillStyle = `rgba(0,0,0,${strength * 0.6})`;
    g.fillRect(0, 0, 1, pitch);
    return c.toDataURL('image/png');
  }, { pitch: parseInt(token('px-pitch')), strength: Number(token('px-grid')) });
  fs.writeFileSync(path.join(TEX, 'pixel-grid.png'), Buffer.from(grid.split(',')[1], 'base64'));
} finally {
  await browser.close();
}

execFileSync(cli, ['run', path.join(TMP, 'case.png'), '--format', 'webp', '--look', '82', '--to', TMP], { stdio: 'ignore' });
fs.copyFileSync(path.join(TMP, 'case.webp'), path.join(TEX, 'case.webp'));
fs.rmSync(TMP, { recursive: true, force: true });
for (const f of ['case.webp', 'grain-case.png', 'grain-light.png', 'grain-mid.png', 'grain-dark.png', 'pixel-grid.png']) console.log(`${f}  ${Math.round(fs.statSync(path.join(TEX, f)).size / 1024)} KB`);
