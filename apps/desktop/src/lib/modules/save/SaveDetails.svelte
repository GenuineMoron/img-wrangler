<!-- SPDX-License-Identifier: AGPL-3.0-or-later -->
<script lang="ts">
  import Field from '$lib/components/Field.svelte';
  import Segmented from '$lib/components/Segmented.svelte';
  import Switch from '$lib/components/Switch.svelte';
  import Row from '$lib/components/Row.svelte';
  import { recipe } from '$lib/state/recipe.svelte';

  const d = $derived(recipe.value!.destination);
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
    <Row label="Folder" note="Subfolders are kept inside it. Press Folder… again to choose another.">
      <Field label="Folder to save into" value={d.folder ?? ''} readonly />
    </Row>
  {:else}
    <p class="note">Originals are replaced in place. They stay in the undo store for 30 days.</p>
  {/if}
  <Row label="Name taken">
    <Segmented label="When a name is taken" neutral value={d.on_conflict} onchange={v => (d.on_conflict = v)}
      options={[{ id: 'number', label: 'Add -2' }, { id: 'skip', label: 'Skip' }, { id: 'overwrite', label: 'Replace' }]} />
  </Row>
  <Switch plate label="Keep each file’s date" on={d.keep_dates} onchange={on => (d.keep_dates = on)} title="New files get the original’s modified date" />
</div>
