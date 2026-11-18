<!-- SPDX-License-Identifier: AGPL-3.0-or-later
  The bars are the looks, each as tall as the file it would make for the selected image. Press a
  bar to choose it; drag the red line to set a size limit. -->
<script lang="ts">
  import { batch } from '$lib/state/batch.svelte';
  import { catalog } from '$lib/state/catalog.svelte';
  import { recipe } from '$lib/state/recipe.svelte';
  import { LOOKS } from './index';

  const H = 60;
  const q = $derived(recipe.value!.quality);
  const item = $derived(batch.current);
  const lookAt = $derived(LOOKS.findIndex(l => l.id === q.look));
  const lossless = $derived(q.look === 'lossless' || q.look === 'untouched');

  /** KB per look for this image: measured for the current look, scaled for the others. */
  const kbs = $derived.by(() => {
    const factor = catalog.info(item?.format ?? recipe.value!.format.id)?.size_vs_jpeg ?? 1;
    const nominal = LOOKS.map(l => l.kb * factor * (item?.planned ? (item.planned.dims.w * item.planned.dims.h) / (2048 * 2048) : 1));
    const measured = item ? batch.outBytes(item) : null;
    // A measured size held down by the limit says nothing about the other looks.
    const capped = q.max_kb != null && measured != null && measured >= q.max_kb * 1024 * 0.9;
    if (measured && lookAt >= 0 && !capped) return nominal.map(n => (n / nominal[lookAt]) * (measured / 1000));
    return nominal;
  });
  // Headroom above the tallest bar so the limit line can be dragged higher.
  const max = $derived(Math.max(...kbs, q.max_kb ?? 0) * 1.35);
  const h = (kb: number) => Math.max(3, (kb / max) * H);
  const here = $derived(item ? batch.outBytes(item) : null);

  let chart: HTMLDivElement;
  function drag(e: PointerEvent) {
    const el = e.currentTarget as HTMLElement;
    el.focus();
    el.setPointerCapture(e.pointerId);
    // Hold the scale still while dragging, or raising the line would stretch the chart under it.
    const scaleKb = max;
    const set = (ev: PointerEvent) => {
      const r = chart.getBoundingClientRect();
      const kb = ((r.bottom - 16 - ev.clientY) / H) * scaleKb;
      q.max_kb = Math.max(10, Math.min(Math.round(scaleKb), Math.round(kb / 10) * 10));
    };
    const move = (ev: Event) => set(ev as PointerEvent);
    el.addEventListener('pointermove', move);
    el.addEventListener('pointerup', () => el.removeEventListener('pointermove', move), { once: true });
  }
  function limitKey(e: KeyboardEvent) {
    const d = { ArrowUp: 1, ArrowRight: 1, ArrowDown: -1, ArrowLeft: -1, PageUp: 10, PageDown: -10 }[e.key];
    if (!d || q.max_kb == null) return;
    e.preventDefault();
    q.max_kb = Math.max(10, q.max_kb + d * (e.shiftKey ? 50 : 10));
  }
</script>

<div class="w-qual">
  <div class="qtop">
    <span class="here">{lossless ? q.look : here ? `≈${Math.round(here / 1000)} KB here` : typeof q.look === 'number' ? `quality ${q.look}` : '--'}</span>
    <button type="button" class="nb" aria-pressed={q.never_bigger} onclick={() => (q.never_bigger = !q.never_bigger)} title="Keep the original when the new file wouldn't be smaller">{q.never_bigger ? '✓ ' : ''}never bigger</button>
  </div>
  <div class="chart" bind:this={chart} role="radiogroup" aria-label="Look">
    {#each LOOKS as look, i (look.id)}
      {@const on = i === lookAt}
      {@const over = q.max_kb != null && kbs[i] > q.max_kb}
      <button type="button" role="radio" aria-checked={on} tabindex={on || (lookAt < 0 && i === 1) ? 0 : -1} disabled={lossless} style:--at={i / 3}
        aria-label="{look.label}, about {Math.round(kbs[i])} KB{over ? ', over the limit' : ''}"
        onclick={() => (q.look = look.id)}
        onkeydown={e => { const d = { ArrowRight: 1, ArrowLeft: -1 }[e.key]; if (d) { e.preventDefault(); q.look = LOOKS[Math.max(0, Math.min(3, i + d))].id; } }}>
        <span class="kb">{Math.round(kbs[i])}</span>
        <span class="bar" class:on class:over style:height="{h(kbs[i])}px"></span>
        <span class="lab">{look.label.toLowerCase()}</span>
      </button>
    {/each}
    {#if q.max_kb != null && !lossless}
      <span class="limit" role="slider" tabindex="0" aria-orientation="vertical" aria-label="Size limit. Drag it or use the arrow keys"
        aria-valuemin="10" aria-valuemax="20000" aria-valuenow={q.max_kb} aria-valuetext="{q.max_kb} KB"
        style:bottom="calc(16px + {h(q.max_kb)}px)" onpointerdown={drag} onkeydown={limitKey}><span>max {q.max_kb} KB</span></span>
    {/if}
  </div>
</div>

<style>
  .w-qual { position: relative; height: 100%; min-width: 0; display: grid; grid-template-rows: auto 1fr; --inset: 22px; }
  .here { overflow: hidden; text-overflow: ellipsis; min-width: 0; }
  .qtop { display: flex; justify-content: space-between; gap: 6px; font-size: 10.5px; color: var(--screen-2); white-space: nowrap; }
  .nb { all: unset; cursor: pointer; color: var(--screen-2); border-radius: 2px; }
  .nb[aria-pressed='false'] { color: var(--screen-3); text-decoration: line-through; }
  .nb:focus-visible { outline: var(--focus-on-screen); }
  .chart { position: relative; }
  .chart button { all: unset; position: absolute; bottom: 0; top: 0; width: 46px; margin-left: -23px; left: calc(var(--inset) + (100% - 2 * var(--inset)) * var(--at));
    cursor: pointer; display: flex; flex-direction: column; justify-content: flex-end; align-items: center; gap: 2px; }
  .chart button:disabled { cursor: default; opacity: .45; }
  .chart button:focus-visible .lab { outline: var(--focus-on-screen); border-radius: 2px; }
  .bar { width: 22px; border-radius: 1px; background: var(--screen-dim); transition: height var(--dur-latch) var(--ease-out); }
  .chart button:hover:not(:disabled) .bar { background: #444; }
  .bar.on { background: var(--accent-quality); box-shadow: 0 0 var(--bloom) color-mix(in srgb, var(--accent-quality) var(--bloom-amt), transparent); }
  .bar.over { background: transparent; box-shadow: inset 0 0 0 1px var(--status-alarm); }
  .kb { position: relative; z-index: 1; font-size: 10.5px; color: var(--screen-3); background: var(--surface-screen); padding: 0 2px; }
  .lab { font-size: 10.5px; line-height: 14px; height: 14px; color: var(--screen-3); }
  .chart button[aria-checked='true'] .kb, .chart button[aria-checked='true'] .lab { color: var(--accent-quality); }
  @container channel (max-width: 210px) {
    .w-qual { --inset: 18px; }
    .here { display: none; }
    .qtop { justify-content: flex-end; }
    .chart button { width: 36px; margin-left: -18px; }
    .bar { width: 18px; }
    .lab { visibility: hidden; }
    .chart button[aria-checked='true'] .lab { visibility: visible; }
  }
  .limit { position: absolute; left: -4px; right: -4px; height: 9px; margin-bottom: -4px; cursor: ns-resize; touch-action: none; z-index: 2; }
  .limit::before { content: ''; position: absolute; left: 0; right: 0; top: 4px; border-top: 1px dashed var(--status-alarm); }
  .limit span { position: absolute; left: 0; top: -12px; font-size: 10.5px; color: var(--status-alarm); background: var(--surface-screen); padding: 0 3px; }
  .limit:hover::before, .limit:focus-visible::before { border-top-style: solid; }
  .limit:focus-visible { outline: none; }
  .limit:focus-visible span { outline: var(--focus-on-screen); }
</style>
