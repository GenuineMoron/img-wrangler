<!-- SPDX-License-Identifier: AGPL-3.0-or-later
  Past runs, newest first, each with Undo while its originals are still kept. -->
<script lang="ts">
  import Dialog from '$lib/components/Dialog.svelte';
  import Key from '$lib/components/Key.svelte';
  import { backend } from '$lib/ipc';
  import { batch } from '$lib/state/batch.svelte';
  import { ui } from '$lib/state/ui.svelte';
  import { bytes, change, plural } from '$lib/text';
  import type { RunInfo } from '$lib/types';

  let runs = $state<RunInfo[]>([]);
  const load = async () => { runs = await backend.history(); };
  $effect(() => { if (ui.dialog === 'history') load(); });

  async function undo(id: string) {
    await batch.undo(id);
    await load();
  }
  const when = (iso: string) => new Date(iso).toLocaleString(undefined, { dateStyle: 'medium', timeStyle: 'short' });
</script>

<Dialog open={ui.dialog === 'history'} onclose={() => (ui.dialog = null)} title="Past runs" wide>
  {#if runs.length}
    <table>
      <thead><tr><th>When</th><th>Recipe</th><th>Images</th><th>Size</th><th><span class="sr-only">Undo</span></th></tr></thead>
      <tbody>
        {#each runs as r (r.id)}
          <tr class:undone={r.undone}>
            <td>{when(r.started)}</td>
            <td>{r.recipe}</td>
            <td>{plural(r.files, 'image')}</td>
            <td>{bytes(r.bytes_in)} → {bytes(r.bytes_out)} <b>{change(r.bytes_in, r.bytes_out)}</b></td>
            <td>{#if r.undone}<span class="note">undone</span>{:else}<Key icon="undo" label="Undo" onclick={() => undo(r.id)} />{/if}</td>
          </tr>
        {/each}
      </tbody>
    </table>
  {:else}
    <p class="note">No runs yet. After a run, its originals are kept here for 30 days so it can be undone.</p>
  {/if}
</Dialog>

<style>
  table { width: 100%; border-collapse: collapse; font: 12.5px/1.3 var(--font-ui); }
  th { text-align: left; font: 600 var(--fs-legend)/1 var(--font-ui); letter-spacing: var(--tracking-legend); text-transform: uppercase; color: var(--ink-2); padding: 0 8px 8px 0; }
  td { padding: 6px 8px 6px 0; border-top: 1px solid var(--seam-dark); font-variant-numeric: tabular-nums; }
  td:last-child { text-align: right; }
  b { font-weight: 600; color: var(--output-wall); }
  .undone td { color: var(--ink-2); }
</style>
