<!-- SPDX-License-Identifier: AGPL-3.0-or-later -->
<script lang="ts">
  import ColourKey from '$lib/components/ColourKey.svelte';
  import Switch from '$lib/components/Switch.svelte';
  import { catalog } from '$lib/state/catalog.svelte';
  import { recipe } from '$lib/state/recipe.svelte';

  const f = $derived(recipe.value!.format);
  const opaque = $derived(f.id !== 'same' && !f.auto && catalog.info(f.id)?.alpha === false);
</script>

<Switch plate label="Pick per image" on={f.auto} onchange={on => (f.auto = on)} title="Chooses for each image: WebP for photos, PNG for graphics and screenshots" />
<ColourKey plate label="Behind clear areas" value={f.matte} onchange={hex => (f.matte = hex)} title="The colour behind see-through areas when the format has none, like JPEG" />
{#if opaque}<span class="sr-only">Transparent areas are filled with this colour.</span>{/if}
