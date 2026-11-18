// SPDX-License-Identifier: AGPL-3.0-or-later
// The self-test. It drives the interface the way a person does (adds a folder, arrows through
// the images, scrolls, drags the size, switches recipes, opens every dialog, runs and undoes) and
// measures each step, then checks the layout at several window sizes. It runs the same in the app
// (`img-wrangler-app selftest <images> <report.json>`) and in a browser (`?selftest`), so numbers
// from a Mac, a Linux desktop and Windows can be put side by side. Loaded only when asked for.

import { backend } from '$lib/ipc';
import { MODULES } from '$lib/modules/registry';
import { batch } from '$lib/state/batch.svelte';
import { fx } from '$lib/state/fx.svelte';
import { recipe } from '$lib/state/recipe.svelte';
import { ui, type DialogId } from '$lib/state/ui.svelte';
import type { SelfTest } from '$lib/types';
import { audit, type Issue } from './layout';
import './fx.css';

export interface Frames {
  /** Frames drawn while the step ran. */
  count: number;
  /** Frame times in ms: the median, the 95th percentile and the worst. */
  p50: number;
  p95: number;
  max: number;
  /** Frames that took longer than two refreshes (33 ms), which reads as a stutter. */
  slow: number;
}

export interface Result {
  name: string;
  /** How long until the result was on screen. */
  ms?: number;
  frames?: Frames;
  note?: string;
}

export interface Report {
  version: string;
  fx: string[];
  engine: string;
  platform: string;
  host: Record<string, string>;
  /** Effects in use: light when drawing is slow (see state/fx.svelte.ts). */
  effects: string;
  started: string;
  window: { w: number; h: number; dpr: number };
  /** From the window opening to the unit being ready. */
  boot_ms: number;
  files: number;
  results: Result[];
  layout: (Issue & { state: string; size: string })[];
}

// ---------- timing helpers ----------

// The next frame, or a few seconds later if the window isn't drawing at all (hidden, or the
// display asleep), so the test never hangs. Long enough not to cut a slow frame short.
const frame = () => new Promise<number>(r => { requestAnimationFrame(r); setTimeout(() => r(performance.now()), 3000); });
const sleep = (ms: number) => new Promise(r => setTimeout(r, ms));
const round = (n: number) => Math.round(n * 10) / 10;

/** Records frame times until the returned function is called. */
function recorder(): () => Frames {
  const times: number[] = [];
  let on = true;
  const loop = (t: number) => { times.push(t); if (on) requestAnimationFrame(loop); };
  requestAnimationFrame(loop);
  return () => {
    on = false;
    const d = times.slice(1).map((t, i) => t - times[i]).sort((a, b) => a - b);
    const q = (p: number) => (d.length ? d[Math.min(d.length - 1, Math.floor(p * d.length))] : 0);
    return { count: d.length, p50: round(q(0.5)), p95: round(q(0.95)), max: round(d.at(-1) ?? 0), slow: d.filter(x => x > 33.4).length };
  };
}

/** Milliseconds from `t0` until `ok()` holds when a frame is drawn, or -1 after the timeout. */
async function until(ok: () => boolean, t0 = performance.now(), timeout = 30000): Promise<number> {
  for (;;) {
    const now = await frame();
    if (ok()) return round(now - t0);
    if (now - t0 > timeout) return -1;
  }
}

const median = (xs: number[]) => (xs.length ? [...xs].sort((a, b) => a - b)[Math.floor(xs.length / 2)] : 0);
const p95 = (xs: number[]) => (xs.length ? [...xs].sort((a, b) => a - b)[Math.min(xs.length - 1, Math.floor(xs.length * 0.95))] : 0);

const sheet = () => document.querySelector<HTMLElement>('.sheet');

/** Every thumbnail the sheet is showing has been decoded. */
function thumbsLoaded(): boolean {
  const box = sheet()?.getBoundingClientRect();
  if (!box) return false;
  const imgs = [...document.querySelectorAll<HTMLImageElement>('.sheet .tile img')].filter(i => {
    const r = i.getBoundingClientRect();
    return r.bottom > box.top && r.top < box.bottom;
  });
  return imgs.length > 0 && imgs.every(i => i.complete && i.naturalWidth > 0);
}

const previewShown = () => !!batch.current && batch.previewPath === batch.currentPath && !batch.previewing;
const draftShown = () => !!batch.current && batch.previewPath === batch.currentPath && !!batch.preview;
const exactShown = () => draftShown() && !!batch.preview?.exact;

// ---------- the steps ----------

async function add(images: string[], out: Result[]) {
  const t0 = performance.now();
  const stop = recorder();
  void batch.add(images, true);
  out.push({ name: 'Add a folder: files listed', ms: await until(() => batch.items.length > 0 && !batch.scanning, t0) });
  out.push({ name: 'Add a folder: first thumbnails shown', ms: await until(thumbsLoaded, t0) });
  out.push({ name: 'Add a folder: first preview shown', ms: await until(previewShown, t0) });
  out.push({ name: 'Add a folder: sizes estimated', ms: await until(() => !!batch.estimate && !batch.estimating, t0, 120000) });
  out.push({ name: 'Add a folder: frames while loading', frames: stop() });
}

async function select(out: Result[]) {
  const list = sheet();
  list?.querySelector<HTMLElement>('[aria-selected="true"]')?.focus();
  const paint: number[] = [], draft: number[] = [], exact: number[] = [];
  const stop = recorder();
  for (let i = 0; i < 12; i++) {
    const t0 = performance.now();
    list?.dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowRight', bubbles: true }));
    await frame();
    paint.push((await frame()) - t0);
    draft.push(await until(draftShown, t0, 15000));
    exact.push(await until(exactShown, t0, 15000));
    // A person looks at each photo for a moment before moving on.
    await sleep(900);
  }
  out.push({ name: 'Arrow to the next image: selection drawn', ms: round(median(paint)), note: `worst ${round(Math.max(...paint))} ms` });
  out.push({ name: 'Arrow to the next image: preview shown', ms: round(median(draft)), note: `p95 ${round(p95(draft))} ms` });
  out.push({ name: 'Arrow to the next image: exact preview', ms: round(median(exact)), note: `p95 ${round(p95(exact))} ms` });
  out.push({ name: 'Arrow through images: frames', frames: stop() });
}

async function scroll(out: Result[]) {
  const list = sheet();
  if (!list) return;
  const max = list.scrollHeight - list.clientHeight;
  const stop = recorder();
  for (let i = 1; i <= 90; i++) {
    list.scrollTop = (max * i) / 90;
    list.dispatchEvent(new Event('scroll'));
    await frame();
  }
  out.push({ name: 'Scroll the sheet top to bottom', frames: stop() });
  const t0 = performance.now();
  out.push({ name: 'Scroll: thumbnails at the bottom shown', ms: await until(thumbsLoaded, t0) });
  list.scrollTop = 0;
  list.dispatchEvent(new Event('scroll'));
}

async function resize(out: Result[]) {
  const size = recipe.value!.size;
  const start = size.long_side;
  const stop = recorder();
  // What dragging the corner of the size frame does, one value per frame.
  for (let i = 0; i <= 60; i++) {
    size.long_side = Math.round((3200 - (2400 * Math.sin((i / 60) * Math.PI))) / 10) * 10;
    await frame();
  }
  out.push({ name: 'Drag the size', frames: stop() });
  const t0 = performance.now();
  size.long_side = start;
  out.push({ name: 'Size settled: preview updated', ms: await until(() => previewShown() && !batch.estimating, t0, 60000) });
}

async function switchRecipes(out: Result[]) {
  const times: number[] = [];
  const stop = recorder();
  for (let i = 0; i < 4; i++) {
    const before = batch.plan;
    const t0 = performance.now();
    void recipe.step(1);
    times.push(await until(() => batch.plan !== before, t0));
  }
  out.push({ name: 'Next recipe: plan redrawn', ms: round(median(times)), note: `worst ${round(Math.max(...times))} ms` });
  out.push({ name: 'Switching recipes: frames', frames: stop() });
  await recipe.load('web-ready');
  await until(() => previewShown() && !batch.estimating, performance.now(), 60000);
}

const DIALOGS: Exclude<DialogId, null>[] = ['settings', 'history', 'recipes', 'palette'];

async function dialogs(out: Result[]) {
  for (const d of DIALOGS) {
    const t0 = performance.now();
    ui.dialog = d;
    const ms = await until(() => [...document.querySelectorAll('dialog')].some(x => x.open), t0);
    out.push({ name: `Open ${d}`, ms });
    ui.dialog = null;
    await frame();
  }
}

async function runAndUndo(out: Result[]) {
  const t0 = performance.now();
  const stop = recorder();
  void batch.run();
  await until(() => batch.running, t0);
  const ms = await until(() => !batch.running, t0, 600000);
  out.push({ name: `Run ${batch.items.length} images`, ms, note: batch.summary ? `${batch.summary.done} done, ${batch.summary.kept} kept, ${batch.summary.failed} failed` : undefined });
  out.push({ name: 'Running: frames', frames: stop() });
  const t1 = performance.now();
  await batch.undo();
  out.push({ name: 'Undo the run', ms: round(performance.now() - t1) });
}

// ---------- layout ----------

async function settle() {
  await frame();
  await frame();
  await sleep(120);
}

async function layoutAt(w: number, h: number, issues: Report['layout']) {
  const size = `${w}×${h}`;
  if (await backend.setWindowSize(w, h)) await settle();
  else if (innerWidth !== w) return; // the browser harness sets its own viewport
  const take = (state: string) => issues.push(...audit().map(i => ({ ...i, state, size })));
  take('main');
  for (const m of MODULES) {
    ui.step = m.id;
    await settle();
    take(`step: ${m.id}`);
  }
  ui.step = null;
  for (const d of DIALOGS) {
    ui.dialog = d;
    await settle();
    take(`dialog: ${d}`);
    ui.dialog = null;
  }
  await settle();
}

export const SIZES: [number, number][] = [[1200, 760], [1440, 900], [1920, 1080]];

/** Layout issues at the current size only, for the browser harness. */
export async function layoutHere(): Promise<Report['layout']> {
  const issues: Report['layout'] = [];
  await layoutAt(innerWidth, innerHeight, issues);
  return dedupe(issues);
}

function dedupe(issues: Report['layout']) {
  const seen = new Set<string>();
  return issues.filter(i => (seen.has(i.kind + i.where + i.size) ? false : (seen.add(i.kind + i.where + i.size), true)));
}

// ---------- the run ----------

export async function run(cfg: SelfTest, bootMs: number): Promise<Report> {
  const report: Report = {
    version: __APP_VERSION__,
    fx: cfg.fx ?? [],
    engine: navigator.userAgent,
    platform: backend.platform,
    host: Object.fromEntries(cfg.host ?? []),
    effects: '',
    started: new Date().toISOString(),
    window: { w: innerWidth, h: innerHeight, dpr: devicePixelRatio },
    boot_ms: round(bootMs),
    files: 0,
    results: [],
    layout: [],
  };
  const out = report.results;
  for (const f of cfg.fx ?? []) document.documentElement.classList.add(f === 'layers' ? 'fx-layers' : `fx-off-${f}`);
  try {
    await recipe.load('web-ready');
    await settle();
    // The automatic effects check, as on a first launch, unless the test asks for one (for this
    // run only: the person's own setting stays as it is).
    if (cfg.effects) fx.mode = cfg.effects;
    await until(() => fx.slow !== null, performance.now(), 30000);
    report.effects = `${fx.light ? 'light' : 'full'} (${fx.mode}, drawing ${fx.slow ? 'slow' : 'fast'}: ${JSON.parse(localStorage.getItem('wrangler.fx-probe') ?? '{}').ms ?? '?'} ms a repaint)`;
    await add(cfg.images, out);
    report.files = batch.items.length;
    if (cfg.measure) {
      await select(out);
      await scroll(out);
      await resize(out);
      await switchRecipes(out);
      await dialogs(out);
    }
    if (cfg.run) await runAndUndo(out);
    if (cfg.layout) for (const [w, h] of SIZES) await layoutAt(w, h, report.layout);
    report.layout = dedupe(report.layout);
  } catch (e) {
    out.push({ name: 'self-test stopped', note: String(e) });
  }
  await backend.selftestDone(JSON.stringify(report, null, 2));
  return report;
}
