<!-- SPDX-License-Identifier: AGPL-3.0-or-later
  Type the pattern (or the find and replace) on the screen; the selected image's new name and
  the first and last of the batch update as you type. -->
<script lang="ts">
  import { batch } from '$lib/state/batch.svelte';
  import { recipe } from '$lib/state/recipe.svelte';
  import { fileName } from '$lib/text';
  import { ui } from '$lib/state/ui.svelte';

  const r = $derived(recipe.value!.rename);
  const item = $derived(batch.current);
  const items = $derived(batch.items);
  let field: HTMLInputElement | undefined = $state();

  function insert(token: string) {
    if (!field) { r.pattern += token; return; }
    const a = field.selectionStart ?? r.pattern.length, b = field.selectionEnd ?? a;
    r.pattern = r.pattern.slice(0, a) + token + r.pattern.slice(b);
    queueMicrotask(() => { field?.focus(); field?.setSelectionRange(a + token.length, a + token.length); });
  }

  const lines = $derived(r.list.split('\n').filter(l => l.trim() && !l.trim().startsWith('#')).length);
  const first = $derived(items[0] ? fileName(items[0].output) : '');
  const last = $derived(items.length > 1 ? fileName(items[items.length - 1].output) : '');
</script>

<div class="w-name">
  {#if r.mode === 'pattern'}
    <div class="pat"><input bind:this={field} bind:value={r.pattern} aria-label="Name pattern" spellcheck="false" autocomplete="off" /></div>
  {:else if r.mode === 'replace' && r.replace[0]}
    <div class="pat two">
      <input bind:value={r.replace[0].find} aria-label="Find" placeholder="find" spellcheck="false" autocomplete="off" />
      <span aria-hidden="true">→</span>
      <input bind:value={r.replace[0].with} aria-label="Replace with" placeholder="with" spellcheck="false" autocomplete="off" />
    </div>
  {:else if r.mode === 'list'}
    <div class="pat"><button type="button" class="paste" onclick={() => ui.reveal('name')}>{lines && batch.plan ? `${batch.items.length - batch.plan.unlisted} of ${batch.items.length} from your list ›` : 'paste a list or spreadsheet ›'}</button></div>
  {:else}
    <div class="pat"><span class="scr-label">names stay as they are</span></div>
  {/if}
  <div class="preview" aria-live="polite">
    {#if item}
      <span class="src">{fileName(item.source)}</span>
      <span class="out glow">→ {fileName(item.output)}</span>
      {#if r.mode !== 'keep' && last}<span class="seq">{first} … <b>{last}</b></span>{/if}
    {:else}
      <span class="src">IMG_0001.HEIC</span><span class="out glow">→ add images to see names</span>
    {/if}
  </div>
  <div class="foot">
    {#if r.mode === 'pattern'}
      <span class="ins">
        <button type="button" onclick={() => insert('{n:000}')} aria-label="Insert a number">+#</button>
        <button type="button" onclick={() => insert('{name}')} aria-label="Insert the original name">+name</button>
        <button type="button" onclick={() => insert('{date}')} aria-label="Insert the date">+date</button>
      </span>
    {:else}<span></span>{/if}
    <span class:alarm={!!batch.plan?.clashes || (r.mode === 'list' && !!batch.plan?.unlisted)}>{batch.plan?.clashes ? `${batch.plan.clashes} taken → -2` : r.mode === 'list' && batch.plan ? (batch.plan.unlisted ? `${batch.plan.unlisted} not on the list` : 'every file matched') : r.mode === 'pattern' ? '' : 'ext follows format'}</span>
  </div>
</div>

<style>
  .w-name { display: grid; grid-template-rows: auto 1fr auto; gap: 6px; height: 100%; font-size: var(--fs-small); --accent: var(--accent-name); }
  .pat { display: flex; align-items: center; gap: 6px; border-bottom: 1px solid var(--screen-line); padding-bottom: 5px; min-height: 24px; }
  .pat input { all: unset; flex: 1; min-width: 0; font: 12.5px/1.4 var(--font-data); color: var(--screen-1); }
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
  .foot { display: flex; justify-content: space-between; align-items: center; gap: 6px; font-size: 10.5px; color: var(--screen-2); }
  .foot .alarm { color: var(--status-alarm); }
  .ins { display: flex; gap: 3px; }
  .ins button { all: unset; cursor: pointer; padding: 0 4px; border-radius: 2px; font-size: 10.5px; line-height: 16px; color: var(--accent-name); box-shadow: inset 0 0 0 1px color-mix(in srgb, var(--accent-name) 50%, transparent); }
  .ins button:hover { background: color-mix(in srgb, var(--accent-name) 18%, transparent); }
  .ins button:focus-visible { outline: var(--focus-on-screen); }
</style>
