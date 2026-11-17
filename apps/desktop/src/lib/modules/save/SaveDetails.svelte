<!-- SPDX-License-Identifier: AGPL-3.0-or-later -->
<script lang="ts">
  import Field from '$lib/components/Field.svelte';
  import Key from '$lib/components/Key.svelte';
  import KeyGroup from '$lib/components/KeyGroup.svelte';
  import Row from '$lib/components/Row.svelte';
  import { backend } from '$lib/ipc';
  import { recipe } from '$lib/state/recipe.svelte';

  const d = $derived(recipe.value!.destination);

  async function choose() {
    const f = await backend.pickFolder();
    if (f) { d.folder = f; d.mode = 'folder'; }
  }
</script>

<div class="details">
  {#if d.mode === 'beside'}
    <Row label="Add to name" note="Only when the new name would match the original.">
      <Field label="Suffix added to the name" bind:value={d.suffix} placeholder="-web" />
    </Row>
  {:else if d.mode === 'subfolder'}
    <Row label="Subfolder" note="Made beside each original.">
      <Field label="Subfolder name" value={d.folder ?? 'Wrangled'} oninput={e => (d.folder = e.currentTarget.value || 'Wrangled')} />
    </Row>
  {:else if d.mode === 'folder'}
    <Row label="Folder" note="Subfolders are kept inside it.">
      <Field label="Folder to save into" value={d.folder ?? ''} readonly placeholder="choose…" />
      <Key icon="folder" label="Choose" onclick={choose} />
    </Row>
  {:else}
    <p class="note">Originals are replaced in place. They stay in the undo store for 30 days.</p>
  {/if}
  <Row label="Name taken">
    <KeyGroup label="When a name is taken" neutral value={d.on_conflict} onchange={v => (d.on_conflict = v)}
      options={[{ id: 'number', label: 'Add -2' }, { id: 'skip', label: 'Skip' }, { id: 'overwrite', label: 'Replace' }]} />
  </Row>
  <Row label="Dates">
    <Key label="Keep file dates" pressed={d.keep_dates} onclick={() => (d.keep_dates = !d.keep_dates)} title="New files get the original’s modified date" />
  </Row>
</div>
