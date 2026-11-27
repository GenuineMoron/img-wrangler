<!-- SPDX-License-Identifier: AGPL-3.0-or-later -->
<script lang="ts">
  import Segmented from '$lib/components/Segmented.svelte';
  import Row from '$lib/components/Row.svelte';
  import { recipe } from '$lib/state/recipe.svelte';
  import type { KeepMeta } from '$lib/types';

  const m = $derived(recipe.value!.metadata);
  const NOTES: Record<KeepMeta, string> = {
    none: 'Strips everything: smallest files, nothing about the camera or place.',
    safe: 'Keeps camera, copyright and date. Removes location and serial numbers.',
    all: 'Keeps everything, including where the photo was taken.',
  };
</script>

<div class="details">
  <Row label="Camera data" note={NOTES[m.keep]}>
    <Segmented label="Metadata to keep" neutral value={m.keep} onchange={v => (m.keep = v)}
      options={[{ id: 'none', label: 'None' }, { id: 'safe', label: 'Safe' }, { id: 'all', label: 'All' }]} />
  </Row>
</div>
