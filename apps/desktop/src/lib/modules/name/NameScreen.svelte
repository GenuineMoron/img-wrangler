<!-- SPDX-License-Identifier: AGPL-3.0-or-later
  Build the name from pieces (or type the find and replace) on the screen. The selected image's
  new name and the first and last of the batch update as you go. -->
<script lang="ts">
  import { batch } from '$lib/state/batch.svelte';
  import { recipe } from '$lib/state/recipe.svelte';
  import { fileName } from '$lib/text';
  import { ui } from '$lib/state/ui.svelte';
  import NameBuild from './NameBuild.svelte';

  const r = $derived(recipe.value!.rename);
  const item = $derived(batch.current);
  const items = $derived(batch.items);

  const lines = $derived(r.list.split('\n').filter(l => l.trim() && !l.trim().startsWith('#')).length);
  /** One line under the names: a clash, how a list matched, or nothing worth saying. */
  const status = $derived.by(() => {
    const p = batch.plan;
    if (p?.clashes) return { text: `${p.clashes} ${p.clashes === 1 ? 'name is' : 'names are'} taken: -2 is added`, alarm: true };
    if (r.mode === 'list' && p) return p.unlisted ? { text: `${p.unlisted} not on the list`, alarm: true } : { text: 'every file matched', alarm: false };
    return r.mode === 'pattern' ? null : { text: 'the ending follows the format', alarm: false };
  });
  const first = $derived(items[0] ? fileName(items[0].output) : '');
  const last = $derived(items.length > 1 ? fileName(items[items.length - 1].output) : '');
</script>

<div class="w-name">
  {#if r.mode === 'pattern'}
    <NameBuild />
  {:else if r.mode === 'replace' && r.replace[0]}
    <div class="pat two">
      <input bind:value={r.replace[0].find} aria-label="Find" placeholder="find" spellcheck="false" autocomplete="off" />
      <span aria-hidden="true">→</span>
      <input bind:value={r.replace[0].with} aria-label="Replace with" placeholder="with" spellcheck="false" autocomplete="off" />
    </div>
  {:else if r.mode === 'list'}
    <div class="pat"><button type="button" class="paste" onclick={() => (ui.more = 'name')}>{lines && batch.plan ? `${batch.items.length - batch.plan.unlisted} of ${batch.items.length} from your list ›` : 'paste a list or spreadsheet ›'}</button></div>
  {:else}
    <div class="pat"><span class="scr-label">names stay as they are</span></div>
  {/if}
  <div class="preview" aria-live="polite" data-long>
    {#if item}
      <!-- Building a name takes the room, so only the new name shows. The rest is in its tooltip. -->
      {#if r.mode !== 'pattern'}<span class="src">{fileName(item.source)}</span>{/if}
      <span class="out glow" title={r.mode === 'pattern' && last ? `${fileName(item.source)} becomes ${fileName(item.output)}. The batch runs from ${first} to ${last}` : undefined}>→ {fileName(item.output)}</span>
      {#if r.mode === 'replace' && last}<span class="seq">{first} … <b>{last}</b></span>{/if}
    {:else}
      <span class="src">IMG_0001.HEIC</span><span class="out glow">→ add images to see names</span>
    {/if}
  </div>
  {#if status}<div class="foot" class:alarm={status.alarm}>{status.text}</div>{/if}
</div>

<style>
  .w-name { display: grid; grid-template-columns: minmax(0, 1fr); grid-template-rows: auto 1fr; grid-auto-rows: auto; gap: 6px; height: 100%; font-size: var(--fs-small); --accent: var(--accent-name); }
  .pat { display: flex; align-items: center; gap: 6px; border-bottom: 1px solid var(--screen-line); padding-bottom: 5px; min-height: 24px; text-wrap: balance; }
  /* width 0 and grow: an input's own width would push the screen wider than its channel */
  .pat input { all: unset; flex: 1 1 0; width: 0; min-width: 0; font: 12.5px/1.4 var(--font-data); color: var(--screen-1); }
  .pat input::placeholder { color: var(--screen-3); }
  .pat input:focus-visible { box-shadow: 0 1px 0 var(--accent-name); }
  .pat.two span { color: var(--screen-3); }
  .paste { all: unset; cursor: pointer; font: 12px/1.4 var(--font-data); color: var(--accent-name); border-radius: 2px; }
  .paste:focus-visible { outline: var(--focus-on-screen); }
  .preview { display: grid; gap: 2px; align-content: center; min-width: 0; }
  .src { color: var(--screen-3); white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .out { font-size: 12.5px; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; color: var(--accent-name); }
  .seq { color: var(--screen-2); font-size: 10.5px; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .seq b { font-weight: 400; color: var(--accent-name); }
  .preview { order: 2; }
  .foot { order: 4; font-size: 10.5px; color: var(--screen-2); white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .foot.alarm { color: var(--status-alarm); }
</style>
