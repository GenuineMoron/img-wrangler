// SPDX-License-Identifier: AGPL-3.0-or-later
// The images being worked on: what's in the batch, what the recipe will do to each (the plan),
// how big the results will be (the estimate), a preview of the selected one, and the run.

import { backend } from '$lib/ipc';
import type { Estimate, ItemReport, Plan, PlanItem, Preview, Summary } from '$lib/types';
import { recipe } from './recipe.svelte';
import { ui } from './ui.svelte';
import { bytes, change, plural } from '$lib/text';

export type ItemStatus = 'ready' | 'clash' | 'kept' | 'over' | 'skipped' | 'failed' | 'done';
export type Tab = 'all' | ItemStatus;

const TAB_ORDER: { id: Tab; label: string; alarm?: boolean }[] = [
  { id: 'all', label: 'All' }, { id: 'ready', label: 'Ready' }, { id: 'done', label: 'Done' }, { id: 'kept', label: 'Kept' },
  { id: 'clash', label: 'Clash', alarm: true }, { id: 'over', label: 'Over', alarm: true }, { id: 'skipped', label: 'Skipped' }, { id: 'failed', label: 'Failed', alarm: true },
];

class BatchState {
  /** What was dropped or picked: files and folders, before the Files step filters them. */
  roots = $state<string[]>([]);
  // Big values are replaced whole, never edited in place, so they skip deep reactivity: a batch
  // of thousands stays as quick as a dozen.
  sources = $state.raw<{ path: string }[]>([]);
  /** Bumped whenever the batch changes, so effects can watch one number. */
  version = $state(0);
  scanning = $state(false);

  plan = $state.raw<Plan | null>(null);
  estimate = $state.raw<Estimate | null>(null);
  estimating = $state(false);

  selectedPath = $state<string | null>(null);
  tab = $state<Tab>('all');

  preview = $state.raw<Preview | null>(null);
  previewPath = $state<string | null>(null);
  previewing = $state(false);
  previewError = $state<string | null>(null);

  running = $state(false);
  progress = $state.raw({ total: 0, done: 0, bytesIn: 0, bytesOut: 0 });
  reports = $state.raw<Record<number, ItemReport>>({});
  summary = $state.raw<Summary | null>(null);
  /** The last run that can be undone. */
  undoable = $state<string | null>(null);

  #planSeq = 0;
  #estSeq = 0;
  #prevSeq = 0;
  #timers: Record<string, ReturnType<typeof setTimeout>> = {};

  items = $derived(this.plan?.items ?? []);
  empty = $derived(this.sources.length === 0);
  selected = $derived(Math.max(0, this.items.findIndex(i => i.source === this.selectedPath)));
  current = $derived<PlanItem | null>(this.items[this.selected] ?? null);
  /** A string, so effects keyed on it don't re-run when only the plan object is new. */
  currentPath = $derived(this.current?.source ?? null);
  bytesIn = $derived(this.items.reduce((a, i) => a + (i.probe?.bytes ?? 0), 0));

  statusOf(item: PlanItem, neverBigger = !!recipe.value?.quality.never_bigger): ItemStatus {
    const rep = this.reports[item.id];
    if (rep) return rep.status === 'over_limit' ? 'over' : rep.status;
    if (item.action.kind === 'skip') return 'skipped';
    if (item.renamed_for_clash) return 'clash';
    const est = this.estimate?.per_item[item.id];
    if (neverBigger && est != null && item.probe && est >= item.probe.bytes) return 'kept';
    return 'ready';
  }

  statuses = $derived.by(() => {
    const neverBigger = !!recipe.value?.quality.never_bigger;
    return this.items.map(i => this.statusOf(i, neverBigger));
  });

  tabs = $derived.by(() => {
    const n: Record<string, number> = { all: this.items.length };
    for (const s of this.statuses) n[s] = (n[s] ?? 0) + 1;
    return TAB_ORDER.filter(t => t.id === 'all' || t.id === 'ready' || n[t.id]).map(t => ({ ...t, label: t.label.toUpperCase(), n: n[t.id] ?? 0 }));
  });

  /** Indices of the items the current tab shows. */
  shown = $derived(this.tab === 'all' ? this.items.map((_, i) => i) : this.items.map((_, i) => i).filter(i => this.statuses[i] === this.tab));

  /** Estimated or measured output bytes for an item. */
  outBytes(item: PlanItem): number | null {
    const rep = this.reports[item.id];
    if (rep) return rep.bytes_out;
    if (this.previewPath === item.source && this.preview?.after_bytes != null) return this.preview.after_bytes;
    return this.estimate?.per_item[item.id] ?? null;
  }

  totals = $derived.by(() => {
    if (this.running || this.summary) {
      const p = this.progress;
      return { bytesIn: this.bytesIn, bytesOut: p.done ? (p.bytesOut / Math.max(1, p.bytesIn)) * this.bytesIn : null, measured: true };
    }
    return { bytesIn: this.bytesIn, bytesOut: this.estimate ? this.estimate.bytes_out : null, measured: false };
  });

  select(path: string | null) {
    this.selectedPath = path;
  }

  move(by: number) {
    const list = this.shown;
    if (!list.length) return;
    const at = Math.max(0, list.indexOf(this.selected));
    const next = list[Math.max(0, Math.min(list.length - 1, at + by))];
    this.select(this.items[next]?.source ?? null);
  }

  async add(paths: string[], replace = false) {
    if (!paths.length || !recipe.value) return;
    this.scanning = true;
    try {
      const before = this.sources.length;
      this.roots = replace ? [...paths] : [...new Set([...this.roots, ...paths])];
      this.sources = await backend.scan(paths, recipe.snapshot().select, !replace);
      this.#batchChanged();
      const added = this.sources.length - (replace ? 0 : before);
      if (!added) ui.say('No images found there that this recipe takes.', true);
      if (!this.selectedPath && this.sources.length) this.select(this.sources[0].path);
    } catch (e) {
      ui.say(String(e), true);
    } finally {
      this.scanning = false;
    }
  }

  /** The Files step changed: read the dropped files and folders again with its filters. */
  async rescan() {
    if (!this.roots.length || !recipe.value || this.running) return;
    this.scanning = true;
    try {
      this.sources = await backend.scan(this.roots, recipe.snapshot().select, false);
      if (this.selectedPath && !this.sources.some(s => s.path === this.selectedPath)) this.select(this.sources[0]?.path ?? null);
      this.#batchChanged();
    } catch (e) {
      ui.say(String(e), true);
    } finally {
      this.scanning = false;
    }
  }

  async remove(paths: string[]) {
    this.sources = await backend.forget(paths);
    if (this.selectedPath && paths.includes(this.selectedPath)) this.select(this.sources[0]?.path ?? null);
    this.#batchChanged();
  }

  async clear() {
    this.roots = [];
    this.sources = await backend.forget([]);
    this.select(null);
    this.plan = null;
    this.estimate = null;
    this.preview = null;
    this.#batchChanged();
  }

  #batchChanged() {
    this.version++;
    this.reports = {};
    this.summary = null;
    this.tab = 'all';
  }

  /** Recipe or batch changed: plan almost at once (it reads no pixels). */
  schedulePlan() {
    if (!this.running && this.summary) { this.summary = null; this.reports = {}; }
    clearTimeout(this.#timers.plan);
    this.#timers.plan = setTimeout(() => this.#replan(), 40);
  }

  /** Something that changes pixels or bytes changed: measure again once edits settle. */
  scheduleEstimate() {
    clearTimeout(this.#timers.est);
    this.#timers.est = setTimeout(() => this.#reestimate(), 450);
  }

  /** The selection moved (show its preview at once) or the recipe changed (wait for the edit
   *  to settle). */
  schedulePreview(moved = false) {
    clearTimeout(this.#timers.prev);
    this.#timers.prev = setTimeout(() => this.#repreview(), moved ? 0 : 150);
  }

  /** The images either side of the selection in the current tab. */
  #neighbours(): string[] {
    const list = this.shown, at = list.indexOf(this.selected);
    return [list[at + 1], list[at - 1]].filter(i => i != null).map(i => this.items[i].source);
  }

  async #replan() {
    if (!recipe.value || !this.sources.length) { this.plan = null; return; }
    const seq = ++this.#planSeq;
    try {
      const p = await backend.plan(recipe.snapshot());
      if (seq === this.#planSeq) this.plan = p;
    } catch (e) {
      if (seq === this.#planSeq) ui.say(String(e), true);
    }
  }

  async #reestimate() {
    if (!recipe.value || !this.sources.length) { this.estimate = null; return; }
    const seq = ++this.#estSeq;
    this.estimating = true;
    try {
      const e = await backend.estimate(recipe.snapshot());
      if (seq === this.#estSeq) this.estimate = e;
    } catch (e) {
      if (String(e) !== 'superseded' && seq === this.#estSeq) ui.say(`Couldn't estimate sizes: ${e}`, true);
    } finally {
      if (seq === this.#estSeq) this.estimating = false;
    }
  }

  /** A quick draft first, then the exact result, then the neighbours' ahead of time, so the
   *  next arrow press finds its preview ready. */
  async #repreview() {
    const item = this.current;
    if (!recipe.value || !item || item.action.kind === 'skip') { this.preview = null; this.previewPath = null; return; }
    const seq = ++this.#prevSeq;
    const r = recipe.snapshot();
    const near = this.#neighbours();
    void backend.previewFocus([item.source, ...near], r);
    this.previewing = true;
    this.previewError = null;
    try {
      let p = await backend.preview(item.source, r, false);
      if (seq !== this.#prevSeq) return;
      this.preview = p;
      this.previewPath = item.source;
      if (!p.exact) {
        p = await backend.preview(item.source, r, true);
        if (seq !== this.#prevSeq) return;
        this.preview = p;
      }
      for (const n of near) backend.preview(n, r, true).catch(() => {});
    } catch (e) {
      const msg = String(e);
      if (seq === this.#prevSeq && msg !== 'stopped' && msg !== 'superseded') { this.previewError = msg; this.preview = null; }
    } finally {
      if (seq === this.#prevSeq) this.previewing = false;
    }
  }

  /** Run the batch, or only some of it ("Run just this one"), keeping the others' results. */
  async run(only?: string[]) {
    if (this.running || !recipe.value || !this.items.length) return;
    this.running = true;
    if (only) {
      const ids = new Set(this.items.filter(i => only.includes(i.source)).map(i => i.id));
      this.reports = Object.fromEntries(Object.entries(this.reports).filter(([id]) => !ids.has(Number(id))));
    } else {
      this.reports = {};
    }
    this.summary = null;
    this.progress = { total: only?.length ?? this.items.length, done: 0, bytesIn: 0, bytesOut: 0 };
    // Files finish faster than frames are drawn: gather them and apply them once per frame.
    let waiting: ItemReport[] = [];
    let frame = 0;
    const apply = () => {
      frame = 0;
      if (!waiting.length) return;
      const reports = { ...this.reports };
      const p = { ...this.progress };
      for (const e of waiting) {
        reports[e.id] = e;
        p.done++;
        p.bytesIn += e.bytes_in;
        p.bytesOut += e.bytes_out;
      }
      waiting = [];
      this.reports = reports;
      this.progress = p;
    };
    try {
      const s = await backend.run(recipe.snapshot(), e => {
        if (e.type === 'started') this.progress = { ...this.progress, total: e.total };
        else if (e.type === 'item') {
          waiting.push(e);
          frame ||= requestAnimationFrame(apply);
        }
      }, only);
      cancelAnimationFrame(frame);
      apply();
      this.summary = s;
      if (s.done + s.kept > 0) this.undoable = s.run_id;
      const problems = s.failed + s.over_limit;
      ui.say(
        s.cancelled ? `Stopped after ${plural(s.done + s.kept, 'image')}. Undo puts them back.`
          : `${plural(s.done, 'image')} saved, ${bytes(s.bytes_in)} → ${bytes(s.bytes_out)} (${change(s.bytes_in, s.bytes_out)}) in ${s.seconds.toFixed(1)} s.${problems ? ` ${problems} need a look.` : ''}`,
        problems > 0,
      );
      if (s.failed) this.tab = 'failed';
    } catch (e) {
      ui.say(String(e), true);
    } finally {
      this.running = false;
    }
  }

  cancel() {
    return backend.cancel();
  }

  async undo(id = this.undoable) {
    if (!id) return;
    try {
      const r = await backend.undo(id);
      if (id === this.undoable) this.undoable = null;
      this.reports = {};
      this.summary = null;
      ui.say(`Undone: ${plural(r.restored, 'original')} restored${r.removed ? `, ${plural(r.removed, 'new file')} removed` : ''}.${r.skipped.length ? ` ${r.skipped.length} left alone (changed since).` : ''}`, r.skipped.length > 0);
      this.version++;
    } catch (e) {
      ui.say(`Couldn't undo: ${e}`, true);
    }
  }
}

export const batch = new BatchState();
