<!-- SPDX-License-Identifier: AGPL-3.0-or-later
  The output side: the meter (what goes in, what comes out, the saving), where it's saved, and
  Run. While running, the OUT ladder and the readout follow the files as they finish. The four
  places to save are keys here, and the rest of Save's settings open from the line above them. -->
<script lang="ts">
  import Key from '$lib/components/Key.svelte';
  import Segmented from '$lib/components/Segmented.svelte';
  import Ladder from '$lib/components/Ladder.svelte';
  import Screen from '$lib/components/Screen.svelte';
  import { backend } from '$lib/ipc';
  import save from '$lib/modules/save';
  import { batch } from '$lib/state/batch.svelte';
  import { recipe } from '$lib/state/recipe.svelte';
  import { ui } from '$lib/state/ui.svelte';
  import { bytes, change, count, fileName } from '$lib/text';
  import type { DestinationMode } from '$lib/types';

  const d = $derived(recipe.value?.destination);
  const n = $derived(batch.items.length);
  const t = $derived(batch.totals);
  const saving = $derived(t.bytesOut != null && t.bytesIn ? 1 - t.bytesOut / t.bytesIn : null);
  const lit = $derived(saving == null ? 0 : Math.max(1, Math.round(24 * (1 - saving))));
  const progress = $derived(batch.progress.total ? batch.progress.done / batch.progress.total : 0);
  const counts = $derived(batch.tabs.filter(x => x.id !== 'all' && x.id !== 'ready' && x.n));

  const MODES: { id: DestinationMode; label: string; aria: string }[] = [
    { id: 'beside', label: 'Beside', aria: 'Beside the originals' },
    { id: 'subfolder', label: 'Subfolder', aria: 'Into a subfolder beside each original' },
    { id: 'folder', label: 'Folder…', aria: 'Into a folder you choose' },
    { id: 'replace', label: 'Replace', aria: 'Replace the originals' },
  ];

  async function setMode(m: DestinationMode) {
    if (!d) return;
    // Pressing Folder… always asks which, so this key is also how to change it.
    if (m === 'folder') {
      const f = await backend.pickFolder();
      if (!f) return;
      d.folder = f;
    }
    if (m === 'subfolder' && (!d.folder || d.folder.includes('/') || d.folder.includes('\\'))) d.folder = 'Wrangled';
    d.mode = m;
  }

  const where = $derived.by(() => {
    if (!d) return '';
    const files = `${count(n)} ${n === 1 ? 'file' : 'files'}`;
    return {
      beside: `Saves ${files} beside the originals`,
      subfolder: `Saves ${files} into ${d.folder || 'Wrangled'}/ beside each original`,
      folder: `Saves ${files} into ${fileName(d.folder || '…')}`,
      replace: `Replaces ${count(n)} originals · undo for 30 days`,
    }[d.mode];
  });
</script>

<section class="output panel" aria-label="Output" data-more-anchor>
  <Screen class="meter" role="status" aria-label={saving != null ? `${bytes(t.bytesIn)} in, about ${bytes(t.bytesOut ?? 0)} out` : 'No estimate yet'}>
    <div class="row"><span>IN</span><Ladder lit={n ? 24 : 0} kind="in" /><span>{n ? bytes(t.bytesIn) : '--'}</span></div>
    <div class="row"><span>OUT</span><Ladder lit={batch.running ? Math.round(24 * progress) : lit} kind="out" /><span>{t.bytesOut != null ? `${t.measured ? '' : '≈'}${bytes(t.bytesOut)}` : '--'}</span></div>
    <div class="readout">
      {#if batch.running}
        <span class="led-readout">{Math.round(progress * 100)}%</span><small>{count(batch.progress.done)} of {count(batch.progress.total)}</small>
      {:else}
        <span class="led-readout">{saving != null && t.bytesOut != null ? change(t.bytesIn, t.bytesOut) : '−−%'}</span>
        <small>{saving != null && t.bytesOut != null ? `${saving >= 0 ? 'saves' : 'adds'} ${bytes(Math.abs(t.bytesIn - t.bytesOut))}` : batch.estimating ? 'measuring…' : 'add images'}</small>
      {/if}
    </div>
    <div class="stats">
      <span><b>{count(n)}</b> images</span>
      {#each counts as c (c.id)}<span><b class:alarm={c.alarm}>{count(c.n)}</b> {c.label.toLowerCase()}</span>{/each}
    </div>
  </Screen>

  <div class="dest">
    <div class="dest__head">
      <span class="legend">Save to</span>
      <button type="button" class="more" id="more-save" aria-expanded={ui.more === 'save'} aria-haspopup="dialog" onclick={() => ui.toggleMore('save')}
        title="More save settings: the folder's name, names already taken, file dates">{recipe.value ? save.summary(recipe.value) : ''} ›</button>
    </div>
    {#if d}<Segmented label="Save to" neutral options={MODES} value={d.mode} onchange={setMode} />{/if}
  </div>

  <div class="go">
    <Key tall icon="undo" label="Undo" disabled={!batch.undoable || batch.running} onclick={() => batch.undo()}
      aria-label={batch.undoable ? 'Undo the last run' : 'Undo the last run: nothing to undo yet'} />
    {#if batch.running}
      <button type="button" class="run run--stop" onclick={() => batch.cancel()}>Stop</button>
    {:else}
      <button type="button" class="run" disabled={!n} onclick={() => batch.run()} aria-describedby="run-note">Run</button>
    {/if}
    <span class="note" id="run-note">{n ? where : 'Add images to run this recipe'}</span>
  </div>
</section>

<style>
  .output { grid-column: 2; grid-row: 3 / 5; padding: var(--sp-6) var(--sp-7); display: grid; grid-template-rows: minmax(0, 1fr) auto auto; gap: var(--sp-5); container: output / inline-size; }
  .output :global(.meter) { padding: var(--sp-5) var(--sp-6); display: grid; grid-template-columns: minmax(0, 1fr) auto; grid-template-rows: auto auto auto; align-content: center; column-gap: var(--sp-7); row-gap: 9px; }
  .row { display: grid; grid-template-columns: 26px minmax(0, 1fr) 62px; gap: var(--sp-4); align-items: center; font-variant-numeric: tabular-nums; }
  .row span:first-child { color: var(--screen-2); }
  .row span:last-child { text-align: right; white-space: nowrap; }
  .readout { grid-column: 2; grid-row: 1 / 3; display: grid; justify-items: end; align-content: center; gap: 3px; min-width: 104px; }
  .readout small { font-size: 10.5px; color: var(--screen-2); white-space: nowrap; }
  .stats { grid-column: 1 / -1; font-size: 10.5px; color: var(--screen-2); display: flex; gap: 10px; flex-wrap: wrap; }
  .stats b { font-weight: 400; color: var(--screen-1); }
  .stats b.alarm { color: var(--status-alarm); }
  @container output (max-width: 330px) {
    .output :global(.meter) { grid-template-columns: minmax(0, 1fr); }
    .readout { grid-column: 1; grid-row: 1; display: flex; align-items: baseline; justify-content: space-between; min-width: 0; }
    .readout :global(.led-readout) { font-size: 30px; }
    .row { grid-template-columns: 26px minmax(0, 1fr) 58px; }
    .stats { display: none; }
  }
  .dest { display: grid; gap: var(--sp-3); }
  .dest__head { display: flex; align-items: baseline; justify-content: space-between; gap: var(--sp-5); min-width: 0; }
  .more { all: unset; cursor: pointer; border-radius: 2px; min-width: 0; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; font: var(--fs-small)/1.3 var(--font-data); color: var(--ink-2); }
  .more:hover, .more[aria-expanded='true'] { color: var(--ink-1); }
  .more:focus-visible { outline: var(--focus); outline-offset: 2px; }
  .go { display: grid; grid-template-columns: 1fr 1.7fr; gap: var(--sp-5); align-content: start; }
  .go .note { grid-column: 1 / -1; text-align: center; }
  /* Greyed by mixing the colours, not with a filter (a filter is costly to repaint). */
  .run:disabled { cursor: default; box-shadow: 0 1px 0 var(--output-wall); transform: none;
    background-color: color-mix(in srgb, var(--output) 40%, #9a9590);
    background-image: var(--tex-grain-mid), linear-gradient(180deg, color-mix(in srgb, var(--output-hi) 40%, #a29d98), color-mix(in srgb, var(--output) 40%, #9a9590) 50%, color-mix(in srgb, var(--output-lo) 40%, #928d88)); }
  .run--stop { background-color: var(--surface-dark); background-image: var(--tex-grain-dark), linear-gradient(180deg, var(--surface-dark-hi), var(--surface-dark) 55%, #27282b);
    box-shadow: inset 0 1px 0 rgba(255, 255, 255, .1), 0 3px 0 var(--surface-dark-wall), 0 4px 2px rgba(0, 0, 0, .2); }
</style>
