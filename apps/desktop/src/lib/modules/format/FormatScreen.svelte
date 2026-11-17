<!-- SPDX-License-Identifier: AGPL-3.0-or-later
  The format list is the control: the main formats, More for the rest. Beside it, what the chosen
  format can do and how big it tends to be against JPG. -->
<script lang="ts">
  import ScreenChoice from '$lib/components/ScreenChoice.svelte';
  import { catalog } from '$lib/state/catalog.svelte';
  import { recipe } from '$lib/state/recipe.svelte';

  const f = $derived(recipe.value!.format);
  const cur = $derived(f.id === 'same' ? null : catalog.info(f.id));
  let browsing = $state(false);

  const items = $derived.by(() => {
    if (browsing) return [...catalog.writable.filter(x => !x.primary).map(x => ({ id: x.ext, label: x.label })), { id: '‹', label: '‹ back', aria: 'Back to the main formats' }];
    const main = [{ id: 'same', label: 'Same', aria: 'Same: each file keeps its format' }, ...catalog.writable.filter(x => x.primary).map(x => ({ id: x.ext, label: x.label }))];
    return [...main, { id: '›', label: cur && !cur.primary ? cur.label : 'more ›', aria: 'More formats' }];
  });

  function choose(id: string) {
    if (id === '›') { browsing = true; return; }
    if (id === '‹') { browsing = false; return; }
    f.id = id;
    f.auto = false;
    browsing = false;
  }

  const vs = $derived(cur ? Math.round((cur.size_vs_jpeg - 1) * 100) : 0);
</script>

{#if browsing}
  <div class="w-more">
    <ScreenChoice label="More formats" class="fgrid" {items} value={f.auto ? undefined : f.id} onchange={choose} />
    <span class="scr-label">{cur && !cur.primary ? cur.hint : 'formats for print, icons and older software'}</span>
  </div>
{:else}
<div class="w-fmt" class:auto={f.auto}>
  <ScreenChoice label="Output format" layout="column" class="flist" {items} value={f.auto ? undefined : f.id === 'same' ? 'same' : cur?.primary ? f.id : '›'} onchange={choose} />
  <div class="spec">
    {#if f.auto}
      <div class="scr-value glow">auto</div>
      <span class="scr-label">WebP for photos<br />PNG for graphics</span>
    {:else if !cur}
      <div class="scr-value glow">same</div>
      <span class="scr-label">each file keeps its format<br />HEIC → JPG · JPEG XL → PNG</span>
    {:else}
      <div class="scr-value glow" title={cur.hint}>{cur.ext}</div>
      <div class="caps" aria-label="Can hold">
        <span class="cap" class:on={cur.alpha} title="Transparency">alpha</span><span class="cap" class:on={cur.animation} title="Animation">anim</span><span class="cap" class:on={cur.lossless} title="Lossless mode">lossless</span>
      </div>
      <div class="vs">
        <div class="bar"><i class="glow" style:width="{Math.min(100, cur.size_vs_jpeg * 50)}%"></i></div>
        <span><b>{vs === 0 ? 'same as jpg' : `${vs > 0 ? '+' : '−'}${Math.abs(vs)}% vs jpg`}</b></span>
      </div>
      <span class="scr-label hint">{cur.browsers ? `${cur.browsers}% of browsers` : 'not for the web'}</span>
    {/if}
  </div>
</div>
{/if}

<svelte:window onkeydown={e => { if (browsing && e.key === 'Escape') { browsing = false; } }} />

<style>
  .w-fmt { display: grid; grid-template-columns: 66px minmax(0, 1fr); gap: var(--sp-6); height: 100%; --accent: var(--accent-format); --on-accent: var(--on-format); color: var(--accent-format); }
  .w-fmt :global(.flist) { align-self: center; border-right: 1px solid var(--screen-line); padding-right: 6px; }
  .w-fmt.auto :global(.flist) { opacity: .55; }
  .spec { display: grid; gap: 5px; align-content: center; min-width: 0; }
  .scr-value { line-height: .9; }
  .caps { display: flex; gap: 3px; }
  .cap { padding: 0 2px; border-radius: 2px; font-size: 9.5px; line-height: 14px; color: var(--screen-3); box-shadow: inset 0 0 0 1px #333; }
  .cap.on { color: var(--accent-format); box-shadow: inset 0 0 0 1px var(--accent-format); }
  .vs { display: grid; gap: 3px; font-size: 10.5px; color: var(--screen-2); white-space: nowrap; }
  .hint { white-space: nowrap; overflow: hidden; text-overflow: ellipsis; font-size: 10.5px; }
  .vs .bar { height: 6px; border-radius: 1px; background: var(--screen-dim); position: relative; }
  .vs .bar::after { content: ''; position: absolute; left: 50%; top: -2px; bottom: -2px; width: 1px; background: var(--screen-2); }
  .vs .bar i { display: block; height: 100%; border-radius: 1px; background: var(--accent-format); }
  .vs b { color: var(--accent-format); font-weight: 400; }
  .w-more { display: grid; grid-template-rows: 1fr auto; gap: 6px; height: 100%; --accent: var(--accent-format); --on-accent: var(--on-format); }
  .w-more :global(.fgrid) { display: grid; grid-template-columns: repeat(3, minmax(0, 1fr)); gap: 2px 6px; align-content: center; }
  .w-more :global(.fgrid button) { font-size: var(--fs-small); color: var(--screen-2); }
  @container channel (max-width: 210px) {
    .w-fmt { grid-template-columns: 54px minmax(0, 1fr); gap: var(--sp-5); }
    .scr-value { font-size: 23px; }
    .caps { display: none; }
  }
</style>
