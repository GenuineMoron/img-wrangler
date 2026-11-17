<!-- SPDX-License-Identifier: AGPL-3.0-or-later
  The selected image, processed exactly as the run will: the result, the original, or both
  split. Click to see actual pixels and move the pointer to look around. -->
<script lang="ts">
  import ScreenChoice from '$lib/components/ScreenChoice.svelte';
  import { MODULES } from '$lib/modules/registry';
  import { batch } from '$lib/state/batch.svelte';
  import { catalog } from '$lib/state/catalog.svelte';
  import { recipe } from '$lib/state/recipe.svelte';
  import { ui, type View } from '$lib/state/ui.svelte';
  import { lookText } from '$lib/modules/quality';
  import { change, fileName, kb } from '$lib/text';

  const item = $derived(batch.current);
  const p = $derived(batch.previewPath === item?.source ? batch.preview : null);
  // Split only makes sense when the framing is unchanged.
  const reframes = $derived(!!recipe.value && MODULES.some(m => (m.id === 'size' || m.id === 'rotate') && m.active(recipe.value!)));
  const view = $derived<Exclude<View, 'auto'>>(ui.view === 'auto' ? (reframes ? 'result' : 'split') : ui.view);

  let zoom = $state(false);
  let focus = $state({ x: 50, y: 50 });
  let el: HTMLDivElement;

  function pan(e: PointerEvent) {
    const r = el.getBoundingClientRect();
    const x = ((e.clientX - r.left) / r.width) * 100, y = ((e.clientY - r.top) / r.height) * 100;
    if (zoom) focus = { x, y };
    if (view === 'split' && e.buttons === 1 && !zoom) ui.split = Math.round(Math.max(0, Math.min(100, x)));
  }

  function key(e: KeyboardEvent) {
    const d = { ArrowLeft: -5, ArrowRight: 5 }[e.key];
    if (d && view === 'split') { e.preventDefault(); ui.split = Math.max(0, Math.min(100, ui.split + d)); }
    if (e.key === 'Enter' || e.key === 'z') { e.preventDefault(); zoom = !zoom; }
  }

  const modes: { id: Exclude<View, 'auto'>; label: string; title?: string }[] = $derived([
    { id: 'result', label: 'RESULT' }, { id: 'original', label: 'ORIGINAL' },
    { id: 'split', label: 'SPLIT', title: reframes ? 'Split lines up best when framing is unchanged' : undefined },
  ]);
  const fmt = $derived(p ? catalog.info(p.format) : item ? catalog.info(item.format) : null);
</script>

<div class="mon">
  <!-- svelte-ignore a11y_no_noninteractive_tabindex (the role is slider or button, both interactive) -->
  <div
    bind:this={el}
    class="view"
    class:split={view === 'split'}
    class:zoom
    class:busy={batch.previewing}
    style:--split="{ui.split}%"
    style:--fx="{focus.x}%"
    style:--fy="{focus.y}%"
    role={view === 'split' ? 'slider' : 'button'}
    aria-pressed={view === 'split' ? undefined : zoom}
    tabindex="0"
    aria-label={view === 'split' ? 'Before and after; arrow keys move the split, Enter zooms' : `${view === 'result' ? 'Result' : 'Original'} preview; press for actual pixels`}
    aria-valuemin={view === 'split' ? 0 : undefined}
    aria-valuemax={view === 'split' ? 100 : undefined}
    aria-valuenow={view === 'split' ? ui.split : undefined}
    onpointermove={pan}
    onpointerdown={pan}
    onclick={() => view !== 'split' && (zoom = !zoom)}
    ondblclick={() => (zoom = !zoom)}
    onkeydown={key}
  >
    {#if p}
      {#if view === 'original'}
        <img src={p.before} alt="" draggable="false" />
      {:else if view === 'result'}
        <span class="fit" style:--ar={p.dims.w / p.dims.h}><img src={p.after} alt="" draggable="false" /></span>
      {:else}
        <img src={p.before} alt="" draggable="false" />
        <img src={p.after} alt="" draggable="false" class="after" />
        <span class="handle"></span>
        <span class="chip l">BEFORE</span><span class="chip r">AFTER</span>
      {/if}
      {#if view !== 'split'}<span class="chip r">{view === 'result' ? 'RESULT' : 'ORIGINAL'}{zoom ? ' · 1:1' : ''}</span>{/if}
    {:else if batch.previewError}
      <p class="msg alarm">{batch.previewError}</p>
    {:else if item}
      <p class="msg">rendering…</p>
    {/if}
  </div>

  <ScreenChoice label="Monitor view" class="modes" items={modes} value={view} onchange={v => (ui.view = v)} />

  {#if item}
    <dl class="facts">
      <dt>FILE</dt><dd><span class="from">{fileName(item.source)} →</span> <span class="name">{fileName(item.output)}</span></dd>
      <dt>SIZE</dt><dd><span class="from">{item.probe?.dims ? `${item.probe.dims.w}×${item.probe.dims.h}` : '?'} →</span>
        <span class="size">{p ? `${p.dims.w}×${p.dims.h}` : item.planned ? `${item.planned.exact ? '' : '≈'}${item.planned.dims.w}×${item.planned.dims.h}` : '…'}</span></dd>
      <dt>WEIGHT</dt><dd><span class="from">{kb(item.probe?.bytes ?? 0)} →</span>
        <span class="weight">{p ? kb(p.after_bytes) : '…'}</span>
        {#if p}{p.kept_original ? ' · already smaller, kept' : ` · ${change(p.before_bytes, p.after_bytes)}`}{p.over_limit ? ' · over limit' : ''}{p.downscaled ? ' · shrunk to fit' : ''}{/if}</dd>
      <dt>LOOK</dt><dd><span class="fmt">{fmt?.label ?? '—'}</span> · {recipe.value ? lookText(recipe.value) : ''}{p?.score ? `, scored ${Math.round(p.score)}` : ''}{p?.quality ? ` (q${p.quality})` : ''}</dd>
    </dl>
  {/if}
</div>

<style>
  .mon { padding: var(--sp-7); display: grid; grid-template-rows: minmax(0, 1fr) auto auto; gap: var(--sp-5); min-height: 0; position: relative; z-index: 4; }
  .view { position: relative; width: 100%; height: 100%; min-height: 160px; border-radius: var(--r-chip); overflow: hidden; background: #111; cursor: zoom-in; outline-offset: 2px; }
  .view.zoom { cursor: zoom-out; }
  .view.split { cursor: ew-resize; touch-action: none; }
  .view.busy img { opacity: .82; transition: opacity 200ms; }
  .view img { position: absolute; inset: 0; width: 100%; height: 100%; object-fit: contain; display: block; }
  .view.zoom img { object-fit: none; object-position: var(--fx) var(--fy); }
  .view { container-type: size; }
  /* the result's own box, so the transparency checker sits only behind the image */
  .fit { position: absolute; inset: 0; margin: auto; aspect-ratio: var(--ar); width: min(100cqw, calc(100cqh * var(--ar))); height: auto;
    background: repeating-conic-gradient(#2a2a2a 0 25%, #1d1d1d 0 50%) 0 0 / 14px 14px; }
  .view.zoom .fit { width: 100%; height: 100%; aspect-ratio: auto; background: none; }
  .after { clip-path: inset(0 0 0 var(--split)); }
  .handle { position: absolute; top: 0; bottom: 0; left: var(--split); width: 2px; margin-left: -1px; background: var(--screen-1); pointer-events: none; }
  .chip { position: absolute; top: var(--sp-4); padding: 2px 6px; font: var(--fs-small)/1.2 var(--font-data); color: #000; background: rgba(236, 235, 231, .92); border-radius: 2px; pointer-events: none; }
  .chip.l { left: var(--sp-4); } .chip.r { right: var(--sp-4); }
  .msg { position: absolute; inset: 0; display: grid; place-items: center; color: var(--screen-2); padding: var(--sp-8); text-align: center; }
  .msg.alarm { color: var(--status-alarm); }
  .mon :global(.modes) { justify-content: center; --accent: var(--screen-1); --on-accent: #000; }
  .facts { display: grid; grid-template-columns: 58px minmax(0, 1fr); gap: 3px var(--sp-5); font: var(--fs-small)/1.4 var(--font-data); }
  .facts dt { color: var(--screen-3); }
  .facts dd { color: var(--screen-1); white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .from { color: var(--screen-3); }
  .name { color: var(--accent-name); } .size { color: var(--accent-size-text); } .weight { color: var(--accent-quality); } .fmt { color: var(--accent-format); }
</style>
