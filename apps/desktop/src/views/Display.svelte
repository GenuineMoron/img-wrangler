<!-- SPDX-License-Identifier: AGPL-3.0-or-later
  The big screen: where the batch is, the sheet of results, the monitor, and a status line that
  shows keyboard hints until there's something to say. -->
<script lang="ts">
  import Screen from '$lib/components/Screen.svelte';
  import Tabs from '$lib/components/Tabs.svelte';
  import { backend } from '$lib/ipc';
  import { batch, type Tab } from '$lib/state/batch.svelte';
  import { ui } from '$lib/state/ui.svelte';
  import { bytes, commonFolder, count } from '$lib/text';
  import Empty from './Empty.svelte';
  import Monitor from './Monitor.svelte';
  import Sheet from './Sheet.svelte';

  const mod = backend.platform === 'mac' ? '⌘' : 'Ctrl ';
  const folder = $derived(commonFolder(batch.items.map(i => i.source)));
  const parent = $derived(folder.slice(0, folder.lastIndexOf('/') + 1));
  const leaf = $derived(folder.slice(folder.lastIndexOf('/') + 1));
</script>

<section class="display" aria-label="Batch">
  <Screen class="main">
    {#if batch.empty}
      <Empty />
    {:else}
      <div class="sbar">
        <span class="path" title={folder}>{parent}<b>{leaf}</b> · {count(batch.items.length)} · {bytes(batch.bytesIn)}</span>
        <Tabs label="Show" items={batch.tabs.map(t => ({ ...t, id: t.id as Tab }))} value={batch.tab} onchange={t => (batch.tab = t)} />
      </div>
      <div class="sbody">
        <Sheet />
        <Monitor />
      </div>
    {/if}
    <div class="hints" role="status" aria-live="polite">
      {#if ui.message}
        <span class="said" class:alarm={ui.message.alarm}>{ui.message.text}</span>
      {:else if batch.scanning}
        <span>reading folders…</span>
      {:else if !batch.empty}
        <span><kbd>←→</kbd>select</span><span><kbd>space</kbd>original</span><span><kbd>1-4</kbd>channels</span><span><kbd>{mod}↵</kbd>run</span><span><kbd>{mod}K</kbd>commands</span>
        <span class="live">{batch.estimating ? 'measuring…' : batch.estimate ? `sizes from ${batch.estimate.sampled} samples` : ''}</span>
      {/if}
    </div>
  </Screen>
</section>

<style>
  .display { border-radius: var(--r-panel); padding: var(--sp-4); background: var(--surface-bezel);
    box-shadow: 0 0 0 1px rgba(0, 0, 0, .4), 0 1px 0 var(--seam-light), inset 0 1px 0 rgba(255, 255, 255, .06); }
  .display :global(.main) { height: 100%; display: grid; grid-template-rows: 36px minmax(0, 1fr) 30px; }
  .sbar { display: flex; align-items: center; gap: var(--sp-8); padding: 0 var(--sp-6); border-bottom: 1px solid var(--screen-line); white-space: nowrap; position: relative; z-index: 4; }
  .path { color: var(--screen-2); overflow: hidden; text-overflow: ellipsis; min-width: 0; }
  .path b { font-weight: 400; color: var(--screen-1); }
  .sbar :global(.tabs) { margin-left: auto; }
  .sbody { display: grid; grid-template-columns: minmax(0, 1fr) clamp(300px, 30%, 380px); min-height: 0; }
  .hints { grid-row: 3; padding: 0 var(--sp-6); border-top: 1px solid var(--screen-line); position: relative; z-index: 4; overflow: hidden; }
  .said { color: var(--screen-1); overflow: hidden; text-overflow: ellipsis; }
  .said.alarm { color: var(--status-alarm); }
  .live { margin-left: auto; color: var(--screen-2); }
</style>
