// SPDX-License-Identifier: AGPL-3.0-or-later
// Builds the README's graphics from the app itself, so they change when the app does.
//
//   pnpm readme            animated SVG banners into .github/assets
//   pnpm readme --shots    also the demo recording and the social preview card
//
// Colours come from design/system/tokens.css, type from design/system/fonts, the hat from the
// app's HatScreen component, and every photo is made by the real CLI, which also shrinks the
// screenshots. The shots need Chromium (set
// CHROME_PATH, or install one with `npx playwright install chromium`) and ffmpeg.

import { execFileSync } from 'node:child_process';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import subsetFont from 'subset-font';

const APP = path.dirname(path.dirname(fileURLToPath(import.meta.url)));
const ROOT = path.resolve(APP, '../..');
const OUT = path.join(ROOT, '.github/assets');
const TMP = fs.mkdtempSync(path.join(os.tmpdir(), 'wrangler-readme-'));
const SAMPLES = path.join(APP, 'dev/samples');
fs.mkdirSync(OUT, { recursive: true });

// ---------------------------------------------------------------- the CLI does the image work
const exe = path.join(ROOT, 'target/release', process.platform === 'win32' ? 'img-wrangler.exe' : 'img-wrangler');
if (!fs.existsSync(exe)) execFileSync('cargo', ['build', '--release', '-p', 'img-wrangler'], { cwd: ROOT, stdio: 'inherit' });
const wrangle = (files, args, to) => {
  execFileSync(exe, ['run', ...files, ...args, '--to', to], { env: { ...process.env, WRANGLER_HOME: path.join(TMP, 'home') }, stdio: 'ignore' });
  return to;
};
const b64 = (file) => fs.readFileSync(file).toString('base64');
const dataUri = (file) => `data:image/${path.extname(file).slice(1).replace('jpg', 'jpeg')};base64,${b64(file)}`;

// ---------------------------------------------------------------- tokens and type
const tokens = Object.fromEntries([...fs.readFileSync(path.join(ROOT, 'design/system/tokens.css'), 'utf8').matchAll(/(--[\w-]+):\s*([^;]+);/g)].map(m => [m[1], m[2].trim()]));
const C = {
  size: tokens['--accent-size'], sizeText: tokens['--accent-size-text'], format: tokens['--accent-format'], quality: tokens['--accent-quality'],
  name: tokens['--accent-name'], output: tokens['--output'], alarm: tokens['--status-alarm'], ink1: tokens['--ink-1'], ink2: tokens['--ink-2'],
  screen1: tokens['--screen-1'], screen2: tokens['--screen-2'], screen3: tokens['--screen-3'],
};
const FONT = path.join(ROOT, 'design/system/fonts');
const TEXT = Array.from({ length: 95 }, (_, i) => String.fromCharCode(32 + i)).join('') + '·→✓×…■●▲◆’−';
const font = async (file, family, text = TEXT) =>
  `@font-face{font-family:'${family}';src:url(data:font/woff2;base64,${(await subsetFont(fs.readFileSync(path.join(FONT, file)), text, { targetFormat: 'woff2' })).toString('base64')}) format('woff2')}`;
const FONTS = [
  await font('host-grotesk.woff2', 'HG'), await font('departure-mono.woff2', 'DM'), await font('doto.woff2', 'DO', '0123456789%−+'),
].join('') + `.hg{font-family:'HG',system-ui,sans-serif}.dm{font-family:'DM',ui-monospace,monospace}.do{font-family:'DO',monospace;font-weight:900;font-variation-settings:'ROND' 45}`;

// ---------------------------------------------------------------- shared drawing
const DEFS = `
<linearGradient id="case" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#f4f3f0"/><stop offset=".35" stop-color="#ebeae6"/><stop offset="1" stop-color="#dcdbd7"/></linearGradient>
<filter id="grain" x="0" y="0" width="100%" height="100%"><feTurbulence type="fractalNoise" baseFrequency=".85" numOctaves="2" seed="11"/><feColorMatrix type="matrix" values="0 0 0 0 .5  0 0 0 0 .5  0 0 0 0 .5  0 0 0 -1.4 .95"/><feComposite in2="SourceGraphic" operator="in"/></filter>
<pattern id="px" width="3" height="3" patternUnits="userSpaceOnUse"><rect width="3" height="1" fill="#000" opacity=".22"/><rect width="1" height="3" fill="#000" opacity=".12"/></pattern>
<linearGradient id="glass" x1="0" y1="0" x2="1" y2="1"><stop offset="0" stop-color="#fff" stop-opacity=".08"/><stop offset=".4" stop-color="#fff" stop-opacity="0"/></linearGradient>
<filter id="bloom" x="-30%" y="-30%" width="160%" height="160%"><feGaussianBlur stdDeviation="3" result="b"/><feMerge><feMergeNode in="b"/><feMergeNode in="SourceGraphic"/></feMerge></filter>
<linearGradient id="run" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#ff8650"/><stop offset=".5" stop-color="${C.output}"/><stop offset="1" stop-color="#ec4b12"/></linearGradient>
<linearGradient id="keycap" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#fbfbfa"/><stop offset=".5" stop-color="#f5f5f4"/><stop offset="1" stop-color="#e2e2e1"/></linearGradient>`;

const kase = (w, h, r = 24) => `<rect width="${w}" height="${h}" rx="${r}" fill="url(#case)"/><rect width="${w}" height="${h}" rx="${r}" fill="#fff" filter="url(#grain)" opacity=".55"/><rect x=".5" y=".5" width="${w - 1}" height="${h - 1}" rx="${r}" fill="none" stroke="#fff" stroke-opacity=".8"/>`;
// An OLED screen. The pixel grid and glass sit under the content: nothing is drawn over pictures.
const screen = (x, y, w, h, inner) => `<g transform="translate(${x} ${y})"><rect x="-8" y="-8" width="${w + 16}" height="${h + 16}" rx="13" fill="#121315"/><rect width="${w}" height="${h}" rx="6" fill="#050505"/><rect width="${w}" height="${h}" rx="6" fill="url(#px)"/><rect width="${w}" height="${h}" rx="6" fill="url(#glass)"/>${inner}</g>`;
const svg = (w, h, title, css, body) => `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 ${w} ${h}" width="${w}" height="${h}" role="img" aria-label="${title}"><title>${title}</title><defs><style>${FONTS}${css}@media (prefers-reduced-motion: reduce){*{animation:none!important}}</style>${DEFS}</defs>${body}</svg>`;
/** Keyframes that show an element only between `a` and `b` seconds of a `T`-second loop. */
const window_ = (name, a, b, T) => { const pa = (a / T) * 100, pb = (b / T) * 100; return `@keyframes ${name}{0%,${pa.toFixed(2)}%{opacity:0}${(pa + 0.01).toFixed(2)}%,${pb.toFixed(2)}%{opacity:1}${(pb + 0.01).toFixed(2)}%,100%{opacity:0}}.${name}{opacity:0;animation:${name} ${T}s step-end infinite}`; };
const label = (x, y, text, size = 13, fill = C.ink2) => `<text x="${x}" y="${y}" class="hg" font-size="${size}" font-weight="600" letter-spacing="1.6" fill="${fill}">${text}</text>`;

// Wrangler's mark, read from the app so the two never differ.
const HAT = fs.readFileSync(path.join(APP, 'src/lib/components/HatScreen.svelte'), 'utf8').match(/<path d="([^"]+)"/)[1];
const hat = (x, y, size, colour = C.quality) => `<g transform="translate(${x} ${y}) scale(${size / 256})" fill="${colour}" filter="url(#bloom)"><path d="${HAT}"/></g>`;
const hatScreen = (x, y, w, h) => `<rect x="${x}" y="${y}" width="${w}" height="${h}" rx="8" fill="#050505"/>${hat(x + (w - h * 0.82) / 2, y + h * 0.09, h * 0.82)}`;

// The sample photos, squared up by the CLI for the file-manager window.
const photos = fs.readdirSync(SAMPLES).filter(f => f.endsWith('.jpg')).sort();
const thumbDir = wrangle(photos.map(f => path.join(SAMPLES, f)), ['--shape', '1:1', '--fit', 'crop', '--size', '128', '--format', 'jpg', '--look', 'high'], path.join(TMP, 'thumbs'));
const thumb = i => dataUri(path.join(thumbDir, photos[i % photos.length]));

// A real run of the Compress for web recipe over the sample photos. Every figure on the hero and the
// right-click card comes from these files.
const webDir = wrangle(photos.map(f => path.join(SAMPLES, f)), ['-r', 'compress-for-web'], path.join(TMP, 'compress-for-web'));
const files = photos.map(f => ({ in: fs.statSync(path.join(SAMPLES, f)).size, out: fs.statSync(path.join(webDir, f.replace(/\.jpg$/, '.webp'))).size }));
const totalIn = files.reduce((a, f) => a + f.in, 0);
const size = n => (n >= 1e6 ? `${(n / 1e6).toFixed(1)} MB` : `${Math.round(n / 1e3)} KB`);
/** After `k` files: the batch's size so far (finished files at their new size, the rest as they were). */
const after = k => files.reduce((a, f, i) => a + (i < k ? f.out : f.in), 0);
const pct = n => `−${Math.round((1 - n / totalIn) * 100)}%`;
const N = files.length, totalOut = after(N);

// ---------------------------------------------------------------- hero: a real run on the unit's screen
function hero() {
  const W = 1200, H = 420, T = 10, start = 1.4, each = 0.6;
  const css = [], b = [kase(W, H)];
  b.push(hatScreen(56, 50, 54, 36));
  b.push(label(126, 74, 'BATCH IMAGE UNIT'));
  b.push(`<text x="50" y="178" class="hg" font-size="96" font-weight="700" letter-spacing="-4.5" fill="#18191b">wrangler</text>`);
  b.push(`<text x="56" y="226" class="hg" font-size="21" font-weight="500" fill="#3c3e43">Resize, reshape, compress, convert, rename and</text>`);
  b.push(`<text x="56" y="254" class="hg" font-size="21" font-weight="500" fill="#3c3e43">watermark images. See every file first.</text>`);
  let x = 56;
  for (const [g, name, c] of [['■', 'size', C.size], ['●', 'format', C.format], ['▲', 'quality', '#e0a800'], ['◆', 'name', C.name]]) {
    b.push(`<text x="${x}" y="298" class="dm" font-size="14" fill="${c}">${g}</text><text x="${x + 18}" y="298" class="dm" font-size="14" fill="${C.ink2}">${name}</text>`);
    x += 32 + name.length * 9;
  }
  b.push(`<g transform="translate(56 328)"><rect y="3" width="120" height="50" rx="11" fill="#a9aaad"/><rect width="120" height="50" rx="11" fill="url(#keycap)"/><text x="60" y="31" class="hg" font-size="12" font-weight="600" letter-spacing="1" fill="${C.ink2}" text-anchor="middle">UNDO</text></g>`);
  css.push(`@keyframes press{0%,4%{transform:translateY(0)}5%,8%{transform:translateY(3px)}9%,100%{transform:translateY(0)}}.run{animation:press ${T}s linear infinite}`);
  b.push(`<g transform="translate(190 328)"><rect y="3" width="250" height="50" rx="11" fill="#b33a0b"/><g class="run"><rect width="250" height="50" rx="11" fill="url(#run)"/><rect width="250" height="50" rx="11" fill="#fff" filter="url(#grain)" opacity=".18"/><text x="125" y="32" class="hg" font-size="17" font-weight="800" letter-spacing="1.2" fill="#fff" text-anchor="middle">RUN</text></g></g>`);

  const sw = 538, sh = 316, inner = [];
  inner.push(`<text x="16" y="26" class="dm" font-size="12" fill="${C.screen2}">~/Pictures/<tspan fill="${C.screen1}">to-sort</tspan> · ${N} · ${size(totalIn)}</text><text x="${sw - 16}" y="26" class="dm" font-size="12" fill="${C.screen1}" text-anchor="end">WEB READY</text><rect y="38" width="${sw}" height="1" fill="#242424"/>`);
  files.forEach((_, i) => {
    const cx = 16 + (i % 3) * 60, cy = 52 + Math.floor(i / 3) * 60;
    inner.push(`<image x="${cx}" y="${cy}" width="52" height="52" href="${thumb(i)}"/>`);
    css.push(window_(`tick${i}`, start + i * each, T - 0.4, T));
    inner.push(`<g class="tick${i} final"><rect x="${cx + 2}" y="${cy + 2}" width="22" height="13" rx="2" fill="#050505" fill-opacity=".86"/><text x="${cx + 13}" y="${cy + 12}" class="dm" font-size="10" fill="${C.screen1}" text-anchor="middle">✓</text></g>`);
  });
  // the recipe's settings: long side 2560, each photo keeps its shape, WebP at the balanced look
  const dx = 222, dy = 52;
  inner.push(`<rect x="${dx + 0.5}" y="${dy + 0.5}" width="111" height="74" fill="none" stroke="${C.screen3}" stroke-dasharray="2 2"/><rect x="${dx + 0.75}" y="${dy + 0.75}" width="110.5" height="73.5" fill="${C.size}" fill-opacity=".14" stroke="${C.sizeText}" stroke-width="1.5" filter="url(#bloom)"/>`);
  inner.push(`<text x="${dx + 132}" y="${dy + 48}" class="hg" font-size="30" font-weight="300" fill="${C.sizeText}" filter="url(#bloom)">2560</text><text x="${dx + 133}" y="${dy + 68}" class="dm" font-size="12" fill="${C.sizeText}">long side · own shape</text><text x="${dx + 133}" y="${dy + 86}" class="dm" font-size="12" fill="${C.format}">jpg → webp · balanced</text>`);
  // the meter: OUT falls as each file finishes, to the run's real total
  const my = 210, lx = 258, lit = k => Math.round((24 * after(k)) / totalIn);
  inner.push(`<text x="222" y="${my + 9}" class="dm" font-size="12" fill="${C.screen2}">IN</text><text x="222" y="${my + 31}" class="dm" font-size="12" fill="${C.screen2}">OUT</text>`);
  for (let i = 0; i < 24; i++) {
    inner.push(`<rect x="${lx + i * 9}" y="${my}" width="7" height="10" rx="1" fill="${C.screen1}"/>`);
    const k = files.findIndex((_, j) => lit(j + 1) <= i);
    if (k >= 0) {
      const pa = ((start + k * each) / T) * 100;
      css.push(`@keyframes seg${i}{0%,${pa.toFixed(2)}%{fill:${C.output}}${(pa + 0.01).toFixed(2)}%,96%{fill:#1b1b1b}96.01%,100%{fill:${C.output}}}.seg${i}{animation:seg${i} ${T}s step-end infinite}`);
    }
    inner.push(`<rect class="seg${i}" x="${lx + i * 9}" y="${my + 22}" width="7" height="10" rx="1" fill="${C.output}"/>`);
  }
  inner.push(`<text x="${lx + 228}" y="${my + 9}" class="dm" font-size="12" fill="${C.screen1}">${size(totalIn)}</text>`);
  for (let k = 0; k <= N; k++) {
    const t0 = k === 0 ? 0 : start + (k - 1) * each, t1 = k === N ? T : start + k * each;
    css.push(window_(`rd${k}`, t0, t1, T));
    inner.push(`<g class="rd${k}${k === N ? ' final' : ''}"><text x="${lx + 228}" y="${my + 31}" class="dm" font-size="12" fill="${C.screen1}">${size(after(k))}</text><text x="${sw - 16}" y="${sh - 14}" class="do" font-size="58" fill="${C.output}" text-anchor="end" filter="url(#bloom)">${pct(after(k))}</text><text x="16" y="${sh - 18}" class="dm" font-size="12" fill="${C.screen1}">${k}/${N}</text></g>`);
  }
  inner.push(`<text x="58" y="${sh - 18}" class="dm" font-size="12" fill="${C.screen2}">done · into Wrangled/</text>`);
  b.push(screen(606, 52, sw, sh, inner.join('')));
  css.push(`@media (prefers-reduced-motion: reduce){.final{opacity:1!important}${Array.from({ length: 24 }, (_, i) => (i >= lit(N) ? `.seg${i}{fill:#1b1b1b}` : '')).join('')}}`);
  return svg(W, H, `Wrangler, a batch image unit. A real run plays on its screen: ${N} photos through the Compress for web recipe, ${size(totalIn)} to ${size(totalOut)}, ${Math.round((1 - totalOut / totalIn) * 100)} percent smaller.`, css.join(''), b.join(''));
}

// ---------------------------------------------------------------- right-click, then the quick window
function rightClick() {
  const W = 1200, H = 380, T = 10;
  const css = [], b = [kase(W, H)];
  b.push(label(56, 52, 'FROM THE FILE MANAGER  ·  FINDER, EXPLORER, DOLPHIN, FILES, NEMO, THUNAR'));
  const wx = 56, wy = 76, ww = 600, wh = 272;
  b.push(`<g transform="translate(${wx} ${wy})"><rect width="${ww}" height="${wh}" rx="10" fill="#fbfbfa" stroke="#000" stroke-opacity=".12"/><rect width="${ww}" height="34" rx="10" fill="#f0f0ee"/><rect y="24" width="${ww}" height="10" fill="#f0f0ee"/><rect y="34" width="${ww}" height="1" fill="#000" opacity=".08"/><circle cx="18" cy="17" r="5.5" fill="#d9d9d6"/><circle cx="36" cy="17" r="5.5" fill="#d9d9d6"/><circle cx="54" cy="17" r="5.5" fill="#d9d9d6"/><text x="${ww / 2}" y="22" class="hg" font-size="13" font-weight="600" fill="${C.ink2}" text-anchor="middle">to-sort</text>`);
  const names = [...photos.map((_, i) => ['IMG_2041', 'IMG_2042', 'DSC_4473', 'IMG_2047', 'cushion', 'IMG_2051', 'wallet', 'succulent'][i] ?? `IMG_${2060 + i}`), 'notes.txt'];
  names.forEach((n, i) => {
    const cx = 26 + (i % 5) * 112, cy = 52 + Math.floor(i / 5) * 104, txt = n.endsWith('.txt'), sel = !txt;
    if (sel) b.push(`<rect x="${cx - 6}" y="${cy - 6}" width="88" height="94" rx="7" fill="#2f6fea" fill-opacity=".14"/>`);
    b.push(txt ? `<path d="M${cx + 18} ${cy}h30l14 14v52h-44z" fill="#fff" stroke="#bbb"/><path d="M${cx + 26} ${cy + 30}h28M${cx + 26} ${cy + 40}h28M${cx + 26} ${cy + 50}h20" stroke="#ccc"/>` : `<image x="${cx + 6}" y="${cy}" width="64" height="64" href="${thumb(i)}"/>`);
    b.push(`<text x="${cx + 38}" y="${cy + 80}" class="hg" font-size="11" font-weight="500" fill="${sel ? '#2f6fea' : C.ink2}" text-anchor="middle">${n}${txt ? '' : '.jpg'}</text>`);
  });
  b.push('</g>');
  const mx = wx + 300, my = wy + 62;
  css.push(window_('menu', 0.6, 3, T), window_('sub', 1.3, 3, T), window_('hiw', 1.1, 3, T), window_('his', 2, 3, T));
  const menu = (x, y, items, cls, hi, hiCls) => {
    const h = items.reduce((a, it) => a + (it === '-' ? 8 : 26), 0) + 10;
    const o = [`<g class="${cls}"><rect x="${x}" y="${y}" width="210" height="${h}" rx="8" fill="#f6f6f6" stroke="#000" stroke-opacity=".18"/>`];
    let yy = y + 5;
    items.forEach((it, k) => {
      if (it === '-') { o.push(`<rect x="${x + 8}" y="${yy + 3.5}" width="194" height="1" fill="#000" opacity=".12"/>`); yy += 8; return; }
      const arrow = it.endsWith(' ›'), text = arrow ? it.slice(0, -2) : it;
      o.push(`<text x="${x + 16}" y="${yy + 16.5}" class="hg" font-size="13" font-weight="500" fill="#1d1d1f">${text}</text>`);
      if (k === hi) o.push(`<g class="${hiCls}"><rect x="${x + 5}" y="${yy}" width="200" height="24" rx="5" fill="#2f6fea"/><text x="${x + 16}" y="${yy + 16.5}" class="hg" font-size="13" font-weight="500" fill="#fff">${text}</text></g>`);
      if (arrow) o.push(`<text x="${x + 196}" y="${yy + 16.5}" class="hg" font-size="13" fill="#555" text-anchor="end">›</text>`);
      yy += 26;
    });
    return o.join('') + '</g>';
  };
  b.push(menu(mx, my, ['Open', 'Get Info', 'Rename…', '-', 'Wrangler ›', 'Copy'], 'menu', 4, 'hiw'));
  b.push(menu(mx + 204, my + 76, ['Compress for web', 'Just compress', 'Convert to JPEG', 'Open in Wrangler…'], 'sub', 0, 'his'));
  // the pointer: right-click on the selection, along to Wrangler, across to Compress for web, click
  css.push(`@keyframes ptr{0%{transform:translate(300px,120px)}5%{transform:translate(340px,150px)}10%,14%{transform:translate(${mx + 70}px,${my + 104}px)}20%{transform:translate(${mx + 240}px,${my + 104}px)}24%,100%{transform:translate(${mx + 240}px,${my + 91}px)}}.ptr{animation:ptr ${T}s ease-in-out infinite}`);
  b.push('<g class="ptr"><path d="M0 0L0 17L4.5 13L7.5 20L10 19L7 12L13 12Z" fill="#111" stroke="#fff" stroke-width="1.2"/></g>');
  // the quick window, showing the same real run
  const qx = 760, qy = 66, qw = 340, qh = 292;
  css.push(window_('qa', 3, T, T));
  const q = [`<g class="qa final"><g transform="translate(${qx} ${qy})">${kase(qw, qh, 18)}${label(18, 32, 'WRANGLER · QUICK', 11, '#18191b')}${hatScreen(qw - 58, 16, 40, 26)}</g>`];
  const inner = ['<text x="14" y="26" class="dm" font-size="12" fill="' + C.screen2 + '">WEB READY</text>'];
  for (let i = 0; i < 24; i++) {
    const pa = ((3 + (i + 1) * 0.09) / T) * 100;
    css.push(`@keyframes qs${i}{0%,${pa.toFixed(2)}%{fill:#1b1b1b}${(pa + 0.01).toFixed(2)}%,100%{fill:${C.output}}}.qs${i}{animation:qs${i} ${T}s step-end infinite}`);
    inner.push(`<rect class="qs${i}" x="${14 + i * 11.5}" y="44" width="9.5" height="11" rx="1" fill="${C.output}"/>`);
  }
  const steps = [0, Math.round(N / 3), Math.round((2 * N) / 3), N];
  steps.forEach((k, j) => {
    const t0 = 3 + j * 0.7, t1 = j === steps.length - 1 ? T : 3 + (j + 1) * 0.7;
    css.push(window_(`qr${j}`, t0, t1, T));
    inner.push(`<g class="qr${j}${j === steps.length - 1 ? ' final' : ''}"><text x="290" y="26" class="dm" font-size="12" fill="${C.screen1}" text-anchor="end">${k}/${N}</text><text x="14" y="126" class="do" font-size="62" fill="${C.output}" filter="url(#bloom)">${pct(after(k))}</text><text x="14" y="154" class="dm" font-size="12" fill="${C.screen2}">${size(totalIn)} → ${size(after(k))}</text></g>`);
  });
  inner.push(`<text x="14" y="176" class="dm" font-size="12" fill="${C.screen2}">into Wrangled/</text>`);
  q.push(screen(qx + 18, qy + 52, 304, 192, inner.join('')));
  [['UNDO', false], ['SHOW', false], ['OPEN', true]].forEach(([l, dark], k) => {
    q.push(`<g transform="translate(${qx + 18 + k * 104} ${qy + 256})"><rect y="2.5" width="96" height="26" rx="7" fill="${dark ? '#141518' : '#a9aaad'}"/><rect width="96" height="26" rx="7" fill="${dark ? '#2f3034' : 'url(#keycap)'}"/><text x="48" y="17" class="hg" font-size="10.5" font-weight="600" letter-spacing=".8" fill="${dark ? '#ececea' : C.ink1}" text-anchor="middle">${l}</text></g>`);
  });
  q.push('</g>');
  b.push(q.join(''));
  css.push('@media (prefers-reduced-motion: reduce){.final{opacity:1!important}.ptr,.menu,.sub{display:none}}');
  return svg(W, H, `${N} photos selected in a file manager. Right-click, Wrangler, Compress for web. A small Wrangler window runs the recipe: ${size(totalIn)} to ${size(totalOut)}, ${Math.round((1 - totalOut / totalIn) * 100)} percent smaller, with Undo, Show and Open.`, css.join(''), b.join(''));
}

// ---------------------------------------------------------------- social preview (upload it in the repo's Settings)
function social() {
  const W = 1280, H = 640;
  const b = [kase(W, H, 0)];
  b.push(hatScreen(96, 96, 120, 80));
  b.push(`<text x="88" y="330" class="hg" font-size="150" font-weight="700" letter-spacing="-7" fill="#18191b">wrangler</text>`);
  b.push(`<text x="96" y="446" class="hg" font-size="32" font-weight="500" fill="#3c3e43">Resize, reshape, compress, convert, rename and</text>`);
  b.push(`<text x="96" y="488" class="hg" font-size="32" font-weight="500" fill="#3c3e43">watermark images. See every file first.</text>`);
  b.push(`<text x="96" y="566" class="dm" font-size="22" fill="${C.ink2}">macOS · Windows · Linux · free and open source</text>`);
  return svg(W, H, 'Wrangler: resize, reshape, compress, convert, rename and watermark images.', '', b.join(''));
}

for (const [name, make] of [['hero.svg', hero], ['right-click.svg', rightClick], ['social.svg', social]]) {
  fs.writeFileSync(path.join(OUT, name), make());
  console.log(`${name}  ${Math.round(fs.statSync(path.join(OUT, name)).size / 1024)} KB`);
}

// ---------------------------------------------------------------- screenshots and the demo, from the app
if (process.argv.includes('--shots')) {
  const { createServer } = await import('vite');
  const { chromium } = await import('playwright-core');
  const server = await createServer({ root: APP, configFile: path.join(APP, 'vite.config.ts'), server: { port: 1430, strictPort: true, host: '127.0.0.1' }, logLevel: 'error' });
  await server.listen();
  const base = 'http://127.0.0.1:1430';
  const browser = await chromium.launch({ executablePath: process.env.CHROME_PATH || undefined, args: ['--no-sandbox'] });
  try {
    // a still of the app at work
    const card = await browser.newPage({ viewport: { width: 1280, height: 640 } });
    await card.goto(`file://${path.join(OUT, 'social.svg')}`);
    await card.waitForTimeout(500);
    await card.screenshot({ path: path.join(TMP, 'social.png') });
    await card.close();
    fs.rmSync(path.join(OUT, 'social.svg'));
    fs.copyFileSync(path.join(wrangle([path.join(TMP, 'social.png')], ['--format', 'jpg', '--look', 'high'], path.join(TMP, 'social')), 'social.jpg'), path.join(OUT, 'social.jpg'));

    // the demo: empty unit, add a shoot, pick a recipe, size it, run it
    const ctx = await browser.newContext({ viewport: { width: 1440, height: 900 }, recordVideo: { dir: TMP, size: { width: 1440, height: 900 } } });
    const p = await ctx.newPage();
    const t0 = Date.now();
    await p.goto(`${base}/`);
    // the recording starts on a blank page, so the demo is cut to begin once the app has painted
    await p.waitForSelector('.empty');
    await p.evaluate(() => document.fonts.ready);
    const painted = (Date.now() - t0) / 1000 + 0.3;
    await p.waitForTimeout(2200);
    await p.click('.empty button:has-text("add images")');
    await p.waitForSelector('.tile');
    await p.waitForTimeout(1400);
    // pick a recipe from the list
    await p.click('.recipe__name');
    await p.waitForTimeout(900);
    await p.click('dialog[open] button.pick:has-text("Small for email")');
    await p.waitForTimeout(1300);
    // build the names from pieces: the old name, then a number
    await p.click('#chan-name .seg button:has-text("Build")');
    await p.waitForTimeout(900);
    await p.click('#chan-name .tray button:has-text("001")');
    await p.waitForTimeout(1300);
    const h = await p.locator('#chan-size .handle').boundingBox();
    await p.mouse.move(h.x + 6, h.y + 6);
    await p.mouse.down();
    await p.mouse.move(h.x - 10, h.y + 12, { steps: 12 });
    await p.mouse.up();
    await p.waitForTimeout(1400);
    await p.click('button.run');
    await p.waitForSelector('button.run:not(.run--stop)', { timeout: 30000 });
    await p.waitForTimeout(2200);
    const video = await p.video().path();
    await ctx.close();
    execFileSync('ffmpeg', ['-y', '-loglevel', 'error', '-ss', painted.toFixed(2), '-i', video, '-vf', 'fps=14,scale=1080:-1:flags=lanczos', '-c:v', 'libwebp_anim', '-quality', '72', '-loop', '0', path.join(OUT, 'demo.webp')]);
    for (const f of ['social.jpg', 'demo.webp']) console.log(`${f}  ${Math.round(fs.statSync(path.join(OUT, f)).size / 1024)} KB`);
  } finally {
    await browser.close();
    await server.close();
  }
}
fs.rmSync(TMP, { recursive: true, force: true });
