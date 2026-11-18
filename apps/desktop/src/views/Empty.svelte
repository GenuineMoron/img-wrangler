<!-- SPDX-License-Identifier: AGPL-3.0-or-later
  Before anything is added: what to do, the two ways to start, and the last few runs. -->
<script lang="ts">
  import { onMount } from 'svelte';
  import HatScreen from '$lib/components/HatScreen.svelte';
  import { backend } from '$lib/ipc';
  import { batch } from '$lib/state/batch.svelte';
  import { ui } from '$lib/state/ui.svelte';
  import { bytes, change, plural } from '$lib/text';
  import type { RunInfo } from '$lib/types';

  let runs = $state<RunInfo[]>([]);
  onMount(async () => { runs = (await backend.history()).slice(0, 3); });

  async function add(kind: 'files' | 'folder') {
    const paths = await backend.pick(kind);
    if (paths.length) await batch.add(paths);
  }
  const ago = (iso: string) => {
    const m = Math.round((Date.now() - Date.parse(iso)) / 60000);
    return m < 60 ? `${Math.max(1, m)} min ago` : m < 1440 ? `${Math.round(m / 60)} h ago` : `${Math.round(m / 1440)} d ago`;
  };
</script>

<div class="empty" class:hot={ui.dropping}>
  <HatScreen size={96} width={150} />
  <h2 class="title">Drop images or folders anywhere on the unit</h2>
  <p class="sub">The recipe on the right decides what happens. Nothing is written until you press Run.</p>
  <div class="start">
    <button type="button" onclick={() => add('files')}>+ add images</button>
    <button type="button" onclick={() => add('folder')}>+ add a folder</button>
  </div>
  {#if runs.length}
    <div class="runs">
      <span class="scr-label">LAST RUNS</span>
      {#each runs as r (r.id)}
        <p><span>{r.recipe}</span> · {plural(r.files, 'image')} · {bytes(r.bytes_in)} → {bytes(r.bytes_out)} <b>{change(r.bytes_in, r.bytes_out)}</b> · {ago(r.started)}{r.undone ? ' · undone' : ''}</p>
      {/each}
    </div>
  {/if}
  <button type="button" class="tip" onclick={() => (ui.dialog = 'settings')}>Tip: add Wrangler to the right-click menu of your file manager ›</button>
</div>

<style>
  .empty { grid-row: 1 / 3; display: grid; align-content: center; justify-items: center; gap: var(--sp-6); text-align: center; padding: var(--sp-9); position: relative; z-index: 4; }
  .empty :global(.hat-screen) { box-shadow: none; background: none; margin-bottom: var(--sp-2); transition: color var(--dur-screen); }
  .hot :global(.hat-screen) { color: var(--output); }
  .title { font: 400 15px/1.3 var(--font-data); color: var(--screen-1); }
  .sub { color: var(--screen-2); max-width: 52ch; }
  .start { display: flex; gap: var(--sp-5); margin-top: var(--sp-3); }
  .start button, .tip { all: unset; cursor: pointer; font: var(--fs-small)/1 var(--font-data); padding: 7px 10px; border-radius: 3px; color: var(--screen-1); box-shadow: inset 0 0 0 1px #3a3a3a; }
  .start button:hover { background: var(--screen-1); color: #000; }
  .start button:focus-visible, .tip:focus-visible { outline: var(--focus-on-screen); outline-offset: 2px; }
  .runs { display: grid; gap: 4px; margin-top: var(--sp-8); color: var(--screen-2); }
  .runs p span { color: var(--screen-1); }
  .runs b { font-weight: 400; color: var(--output); }
  .tip { box-shadow: none; color: var(--screen-3); margin-top: var(--sp-6); }
  .tip:hover { color: var(--screen-1); }
</style>
