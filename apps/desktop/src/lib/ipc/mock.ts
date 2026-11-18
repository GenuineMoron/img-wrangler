// SPDX-License-Identifier: AGPL-3.0-or-later
// A stand-in backend for design work in a plain browser (`pnpm dev`, dev builds only). Sample
// photos play a client's product shoot; sizes come from a simple model and previews are drawn
// on a canvas with the same frame layout as the engine. Open with ?demo to start with files.

import type {
  Dims, Estimate, Format, FormatInfo, Integrations, Plan, PlanItem, Preview, Recipe, RecipeEntry, RunEvent, RunInfo, SourceFile, Status, Summary,
} from '$lib/types';
import { layout, specOf } from '$lib/geometry';
import type { Backend } from './backend';
import RECIPES from '../../../dev/recipes.json';

const HOME = '/Users/sean/Clients/Acme/spring-catalogue';
interface Sample { name: string; photo: number; w: number; h: number; bytes: number; detail: number; format: Format }
const SAMPLES: Sample[] = [
  ['DSC_4471.JPG', 1, 6000, 4000, 4.30e6, 0.8], ['DSC_4472.JPG', 2, 6000, 4000, 3.99e6, 1.0], ['DSC_4473.JPG', 3, 4000, 4000, 6.96e6, 0.7],
  ['board_oak_v2.jpg', 4, 4032, 3024, 2.15e6, 1.1], ['cushion-green.tif', 5, 5472, 3648, 38.9e6, 1.5], ['IMG_0921.HEIC', 6, 3024, 4032, 2.10e6, 0.6],
  ['wallet-tan.jpg', 7, 1600, 1600, 0.18e6, 1.0], ['succulent_01.jpg', 8, 4000, 6000, 2.76e6, 1.3], ['DSC_4480.JPG', 1, 6000, 4000, 4.41e6, 0.8],
  ['DSC_4481.JPG', 2, 6000, 4000, 4.05e6, 1.0], ['DSC_4482.JPG', 3, 4000, 4000, 6.89e6, 0.7], ['board_walnut.jpg', 4, 4032, 3024, 2.33e6, 1.2],
  ['cushion-ochre.tif', 5, 5472, 3648, 37.2e6, 1.6], ['IMG_0925.HEIC', 6, 3024, 4032, 2.08e6, 0.6], ['wallet-black.png', 7, 3000, 3000, 9.10e6, 1.0],
  ['succulent_02.jpg', 8, 4000, 6000, 2.81e6, 1.3],
].map(([name, photo, w, h, bytes, detail]) => {
  const ext = String(name).split('.').pop()!.toLowerCase();
  const format: Format = ext === 'tif' ? 'tiff' : ext === 'heic' ? 'heic' : ext === 'png' ? 'png' : 'jpeg';
  return { name, photo, w, h, bytes, detail, format } as Sample;
});

// The engine's format table (crates/engine/src/codec/format.rs).
const FORMATS: FormatInfo[] = ([
  ['jpeg', 'JPG', 'jpg', true, false, false, true, false, 100, 1.0, true, 'works everywhere'],
  ['png', 'PNG', 'png', true, true, false, true, true, 100, 2.6, true, 'graphics and transparency'],
  ['webp', 'WebP', 'webp', true, true, true, true, true, 97, 0.7, true, 'smaller than JPG, nearly universal'],
  ['avif', 'AVIF', 'avif', true, true, true, true, true, 96, 0.52, true, 'smallest, slower to save'],
  ['jxl', 'JPEG XL', 'jxl', false, true, true, true, true, 69, 0.46, false, 'read only for now'],
  ['gif', 'GIF', 'gif', true, true, true, true, false, 100, 1.7, false, '256 colours'],
  ['tiff', 'TIFF', 'tif', true, true, false, false, true, 0, 3.6, false, 'print and archive'],
  ['bmp', 'BMP', 'bmp', true, false, false, false, true, 100, 6.0, false, 'uncompressed'],
  ['ico', 'ICO', 'ico', true, true, false, false, true, 100, 1.0, false, 'favicons, up to 256 px'],
  ['qoi', 'QOI', 'qoi', true, true, false, false, true, 0, 3.0, false, 'fast lossless'],
  ['tga', 'TGA', 'tga', true, true, false, false, true, 0, 4.0, false, 'legacy'],
  ['pnm', 'PNM', 'ppm', true, false, false, false, true, 0, 6.0, false, 'legacy'],
  ['svg', 'SVG', 'svg', false, true, false, false, true, 100, 1.0, false, 'read and rasterised'],
  ['heic', 'HEIC', 'heic', false, true, false, true, false, 0, 0.6, false, 'read only'],
] as const).map(([id, label, ext, encode, alpha, animation, lossy, lossless, browsers, size_vs_jpeg, primary, hint]) =>
  ({ id, label, ext, mime: `image/${ext}`, decode: true, encode, alpha, animation, lossy, lossless, browsers, size_vs_jpeg, primary, hint }) as FormatInfo);

const recipes = structuredClone(RECIPES) as unknown as Record<string, Recipe>;
const delay = (ms: number) => new Promise(r => setTimeout(r, ms));
const clone = <T>(v: T): T => structuredClone(v);

let batch: SourceFile[] = [];
let cancelled = false;
const history: RunInfo[] = [];
let pins = ['web-ready', 'just-compress', 'shopify-product'];
const installed = new Set<string>();
const launched = new Set<string>();

const sampleFor = (path: string) => SAMPLES.find(s => path.endsWith('/' + s.name))!;
const photoUrl = (n: number) => `/dev/samples/p${n}.jpg`;

function outputFormat(r: Recipe, src: Format): Format {
  if (r.format.auto) return ['png', 'gif', 'svg', 'bmp', 'ico', 'tga', 'qoi'].includes(src) ? 'png' : 'webp';
  if (r.format.id === 'same') return ({ heic: 'jpeg', jxl: 'png', svg: 'png', tga: 'png', pnm: 'png' } as Record<string, Format>)[src] ?? src;
  return FORMATS.find(f => f.ext === r.format.id || f.id === r.format.id)?.id ?? 'jpeg';
}

const LOOK_KB: Record<string, number> = { small: 243, balanced: 411, high: 650, pristine: 871, lossless: 1900, untouched: 0 };
const ext = (f: Format) => FORMATS.find(x => x.id === f)!.ext;

function slug(s: string) {
  return s.normalize('NFKD').replace(/[̀-ͯ]/g, '').toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/^-|-$/g, '');
}

/** A simple stand-in for the engine's list parsing: `old, new` pairs, or names in order. */
function listName(text: string, stem: string, index: number): string | null {
  const rows = text.split('\n').map(l => l.trim()).filter(l => l && !l.startsWith('#')).map(l => l.split(/\t|,/).map(c => c.trim().replace(/^"|"$/g, '')));
  const strip = (n: string) => n.replace(/\.[a-z0-9]{1,5}$/i, '');
  const pairs = rows.filter(r => r.length >= 2 && r[1]);
  if (pairs.length * 2 > rows.length) return strip(pairs.find(r => strip(r[0]).toLowerCase() === stem.toLowerCase())?.[1] ?? '') || null;
  return rows[index]?.[0] ? strip(rows[index][0]) : null;
}

function renderName(r: Recipe, s: Sample, index: number, dims: Dims, fmt: Format): string {
  const stem = s.name.replace(/\.[^.]+$/, '');
  let base = stem;
  for (const x of r.rename.replace) if (x.find) base = base.split(x.find).join(x.with);
  const listed = r.rename.mode === 'list' ? listName(r.rename.list, stem, index) : null;
  let name = r.rename.mode === 'keep' ? stem : r.rename.mode === 'replace' ? base : r.rename.mode === 'list' && listed == null ? stem
    : (listed ?? r.rename.pattern).replace(/\{(\w+)(?::(0+))?(?:\|(\w+))?\}/g, (m, tok: string, pad?: string, filter?: string) => {
      const n = r.rename.start + index * Math.max(1, r.rename.step);
      const v = { name: base, n: pad ? String(n).padStart(pad.length, '0') : String(n), w: String(dims.w), h: String(dims.h), date: '2026-11-17', taken: '2026-10-02', folder: 'spring-catalogue', ext: ext(fmt) }[tok];
      if (v == null) return m;
      return filter === 'slug' ? slug(v) : filter === 'lower' ? v.toLowerCase() : filter === 'upper' ? v.toUpperCase() : v;
    });
  name = { keep: name, lower: name.toLowerCase(), upper: name.toUpperCase(), slug: slug(name), title: name.replace(/\b\w/g, c => c.toUpperCase()) }[r.rename.case];
  return name.replace(/[/\\:*?"<>|]/g, '-') || 'image';
}

function plan(r: Recipe): Plan {
  const taken = new Set<string>();
  const items: PlanItem[] = batch.map((src, id) => {
    const s = sampleFor(src.path);
    const fmt = outputFormat(r, s.format);
    const dims = layout({ w: s.w, h: s.h }, specOf(r.size)).canvas;
    const d = r.destination;
    const dir = d.mode === 'subfolder' ? `${HOME}/${d.folder || 'Wrangled'}` : d.mode === 'folder' ? (d.folder || '~/Desktop/Wrangled') : HOME;
    let name = renderName(r, s, id, dims, fmt);
    if (d.mode === 'beside' && `${name}.${ext(fmt)}` === s.name) name += d.suffix;
    let output = `${dir}/${name}.${ext(fmt)}`;
    let clash = false;
    for (let n = 2; taken.has(output) || (d.mode !== 'replace' && SAMPLES.some(x => `${HOME}/${x.name}` === output && output !== src.path)); n++) {
      output = `${dir}/${name}-${n}.${ext(fmt)}`;
      clash = true;
    }
    taken.add(output);
    return {
      id, source: src.path, probe: { format: s.format, dims: s.format === 'heic' ? null : { w: s.w, h: s.h }, bytes: s.bytes }, output, format: fmt,
      planned: { dims, exact: r.size.trim == null }, action: { kind: 'process' }, renamed_for_clash: clash, in_place: d.mode === 'replace' && output === src.path,
    };
  });
  const unlisted = r.rename.mode === 'list' ? batch.filter((b, i) => listName(r.rename.list, sampleFor(b.path).name.replace(/\.[^.]+$/, ''), i) == null).length : 0;
  return { items, clashes: items.filter(i => i.renamed_for_clash).length, bytes_in: items.reduce((a, i) => a + (i.probe?.bytes ?? 0), 0), unlisted };
}

function guess(r: Recipe, item: PlanItem): number {
  const s = sampleFor(item.source);
  const d = item.planned!.dims;
  const factor = FORMATS.find(f => f.id === item.format)!.size_vs_jpeg;
  const look = typeof r.quality.look === 'number' ? 300 + r.quality.look * 6 : LOOK_KB[r.quality.look] ?? 411;
  const area = r.size.pad.fill < 100 && r.size.fit === 'pad' ? (r.size.pad.fill / 100) ** 2 : 1;
  let out = look * 1000 * factor * s.detail * ((d.w * d.h) / (2048 * 2048)) * (0.25 + 0.75 * area);
  if (r.quality.max_kb && out > r.quality.max_kb * 1024) out = r.quality.max_kb * 1024 * 0.95;
  const same = item.format === s.format && d.w === s.w && d.h === s.h;
  if (r.quality.never_bigger && same && out >= s.bytes) out = s.bytes;
  return Math.round(out);
}

function estimate(r: Recipe): Estimate {
  const p = plan(r);
  const per_item = p.items.map(i => guess(r, i));
  return { sampled: Math.min(8, per_item.length), per_item, bytes_out: per_item.reduce((a, b) => a + b, 0) };
}

const loaded = new Map<number, Promise<HTMLImageElement>>();
function photo(n: number) {
  if (!loaded.has(n)) loaded.set(n, new Promise((ok, fail) => { const i = new Image(); i.onload = () => ok(i); i.onerror = fail; i.src = photoUrl(n); }));
  return loaded.get(n)!;
}

/** Draw a sample at its pretend aspect, then lay it out like the engine would. */
async function render(s: Sample, r: Recipe | null): Promise<{ url: string; dims: Dims }> {
  const img = await photo(s.photo);
  const scale = 900 / Math.max(s.w, s.h);
  const sw = Math.round(s.w * scale), sh = Math.round(s.h * scale);
  const src = document.createElement('canvas');
  Object.assign(src, { width: sw, height: sh });
  const sc = src.getContext('2d')!;
  const k = Math.max(sw / img.width, sh / img.height);
  sc.drawImage(img, (sw - img.width * k) / 2, (sh - img.height * k) / 2, img.width * k, img.height * k);
  const full = { w: s.w, h: s.h };
  if (!r) return { url: src.toDataURL('image/jpeg', 0.9), dims: full };
  const l = layout({ w: sw, h: sh }, { ...specOf(r.size), long_side: r.size.long_side ? Math.round(r.size.long_side * scale) : null, upscale: true, shape: specOf(r.size).shape.kind === 'exact' ? { kind: 'exact', w: Math.round((specOf(r.size).shape as { w: number }).w * scale), h: Math.round((specOf(r.size).shape as { h: number }).h * scale) } : specOf(r.size).shape });
  const out = document.createElement('canvas');
  Object.assign(out, { width: l.canvas.w, height: l.canvas.h });
  const oc = out.getContext('2d')!;
  const corner = sc.getImageData(2, 2, 1, 1).data;
  const bd = r.size.pad.backdrop;
  oc.fillStyle = bd === 'auto' ? `rgb(${corner[0]},${corner[1]},${corner[2]})` : bd === 'transparent' ? 'rgba(0,0,0,0)' : bd;
  if (bd !== 'transparent') oc.fillRect(0, 0, out.width, out.height);
  const c = l.crop ?? { x: 0, y: 0, w: sw, h: sh };
  oc.drawImage(src, c.x, c.y, c.w, c.h, l.offset[0], l.offset[1], l.scaled.w, l.scaled.h);
  const wm = r.addons.find(a => a.kind === 'watermark');
  if (wm && wm.kind === 'watermark' && wm.enabled && wm.image) {
    // A stand-in mark: the engine draws the real logo file.
    const short = Math.min(out.width, out.height), long = short * wm.size / 100, h = long / 3.2, m = short * wm.margin / 100;
    const [fx, fy] = { center: [0.5, 0.5], top: [0.5, 0], bottom: [0.5, 1], left: [0, 0.5], right: [1, 0.5], top_left: [0, 0], top_right: [1, 0], bottom_left: [0, 1], bottom_right: [1, 1] }[wm.anchor];
    const x = m + (out.width - long - 2 * m) * fx, y = m + (out.height - h - 2 * m) * fy;
    oc.globalAlpha = wm.opacity / 100;
    oc.fillStyle = '#18191b';
    oc.beginPath(); oc.roundRect(x, y, long, h, h * 0.22); oc.fill();
    oc.fillStyle = '#ffffff'; oc.font = `700 ${h * 0.5}px "Host Grotesk", sans-serif`; oc.textAlign = 'center'; oc.textBaseline = 'middle';
    oc.fillText('acme', x + long / 2, y + h / 2 + 1);
    oc.globalAlpha = 1;
  }
  const dims = layout(full, specOf(r.size)).canvas;
  return { url: out.toDataURL('image/png'), dims };
}

export const mock: Backend = {
  kind: 'mock',
  platform: /Mac/.test(navigator.userAgent) ? 'mac' : 'linux',

  formats: async () => clone(FORMATS),
  recipes: async () => Object.entries(recipes).filter(([id]) => id !== 'new').map(([id, r]): RecipeEntry => ({ id, name: r.name, description: r.description, built_in: id in RECIPES })),
  loadRecipe: async id => { if (!recipes[id]) throw new Error(`no recipe called "${id}"`); return clone(recipes[id]); },
  saveRecipe: async r => { const id = slug(r.name); recipes[id] = clone(r); return id; },
  deleteRecipe: async id => { delete recipes[id]; },

  scan: async (_paths, _select, append) => {
    await delay(120);
    const all = SAMPLES.map(s => ({ path: `${HOME}/${s.name}`, root: HOME, rel: s.name }));
    batch = append ? [...batch, ...all.filter(a => !batch.some(b => b.path === a.path))] : all;
    return clone(batch);
  },
  forget: async paths => { batch = paths.length ? batch.filter(b => !paths.includes(b.path)) : []; return clone(batch); },
  plan: async r => plan(r),
  estimate: async r => { await delay(350); return estimate(r); },
  async preview(path, r) {
    const s = sampleFor(path);
    const item = plan(r).items.find(i => i.source === path)!;
    const [before, after] = await Promise.all([render(s, null), render(s, r)]);
    const bytes = guess(r, item);
    return {
      before: before.url, after: after.url, before_bytes: s.bytes, after_bytes: bytes, source_dims: { w: s.w, h: s.h }, dims: after.dims,
      format: item.format, quality: typeof r.quality.look === 'number' ? r.quality.look : 72, score: 75.4,
      kept_original: bytes === s.bytes, over_limit: false, downscaled: false,
    };
  },

  async run(r, onEvent: (e: RunEvent) => void) {
    cancelled = false;
    const p = plan(r);
    const run_id = `mock-${Date.now()}`;
    const t0 = performance.now();
    onEvent({ type: 'started', run_id, total: p.items.length });
    const sum: Summary = { run_id, total: p.items.length, done: 0, kept: 0, over_limit: 0, skipped: 0, failed: 0, bytes_in: 0, bytes_out: 0, seconds: 0, cancelled: false };
    for (const item of p.items) {
      if (cancelled) { sum.cancelled = true; break; }
      await delay(90 + Math.random() * 140);
      const out = guess(r, item);
      const status: Status = out === item.probe!.bytes ? 'kept' : 'done';
      sum[status === 'kept' ? 'kept' : 'done']++;
      sum.bytes_in += item.probe!.bytes;
      sum.bytes_out += out;
      onEvent({ type: 'item', id: item.id, status, bytes_in: item.probe!.bytes, bytes_out: out, output: item.output, message: null });
    }
    sum.seconds = (performance.now() - t0) / 1000;
    onEvent({ type: 'finished', ...sum });
    history.unshift({ id: run_id, recipe: r.name, started: new Date().toISOString(), finished: new Date().toISOString(), files: sum.done + sum.kept, bytes_in: sum.bytes_in, bytes_out: sum.bytes_out, undone: false });
    return sum;
  },
  async quickRun(recipeId, paths, onEvent) {
    const recipe = clone(recipes[recipeId] ?? recipes['web-ready']);
    if (!batch.length) await this.scan(paths, recipe.select, false);
    const summary = await this.run(recipe, onEvent);
    return { recipe, summary, first_output: `${HOME}/Wrangled/${SAMPLES[0].name}` };
  },
  openMain: async paths => console.info('open in main window', paths),
  cancel: async () => { cancelled = true; },
  undo: async id => {
    const h = history.find(x => x.id === id);
    if (h) h.undone = true;
    return { restored: h?.files ?? 0, removed: 0, skipped: [] };
  },
  history: async () => clone(history),

  takeLaunch: async kind => {
    // Like the host: each launch is handed out once.
    if (launched.has(kind)) return null;
    launched.add(kind);
    const q = new URLSearchParams(location.search);
    if (kind === 'quick') return { mode: 'quick', recipe: q.get('recipe') ?? 'shopify-product', paths: [HOME] };
    return q.has('demo') ? { mode: 'open', paths: [HOME] } : null;
  },
  onLaunch: async () => () => {},
  async onDrop(drop, hover) {
    const over = (e: DragEvent) => { e.preventDefault(); hover(true); };
    const leave = (e: DragEvent) => { if (!e.relatedTarget) hover(false); };
    const dropped = (e: DragEvent) => { e.preventDefault(); hover(false); drop([HOME]); };
    addEventListener('dragover', over); addEventListener('dragleave', leave); addEventListener('drop', dropped);
    return () => { removeEventListener('dragover', over); removeEventListener('dragleave', leave); removeEventListener('drop', dropped); };
  },
  pick: async () => [HOME],
  pickFolder: async () => '/Users/sean/Desktop/Wrangled',
  pickImage: async () => '/Users/sean/Brand/acme-logo.png',
  reveal: async path => console.info('reveal', path),
  thumbUrl: path => photoUrl(sampleFor(path)?.photo ?? 1),

  integrations: async (): Promise<Integrations> => ({ targets: ['Finder', 'Dolphin', 'Nautilus'].map(l => ({ target: l.toLowerCase(), label: l, installed: installed.has(l) })), pins: [...pins] }),
  async setIntegrations(install, p) {
    pins = p;
    for (const l of ['Finder', 'Dolphin', 'Nautilus']) install ? installed.add(l) : installed.delete(l);
    return this.integrations();
  },

  ready: async () => {},
  close: async () => console.info('close window'),
};
