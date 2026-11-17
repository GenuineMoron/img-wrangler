<!-- SPDX-License-Identifier: AGPL-3.0-or-later -->
<script lang="ts">
  import ColourKey from '$lib/components/ColourKey.svelte';
  import Key from '$lib/components/Key.svelte';
  import { catalog } from '$lib/state/catalog.svelte';
  import { recipe } from '$lib/state/recipe.svelte';

  const f = $derived(recipe.value!.format);
  const opaque = $derived(f.id !== 'same' && !f.auto && catalog.info(f.id)?.alpha === false);
</script>

<div class="keys" style:--n="2" style:--accent="var(--accent-format)" style:--on-accent="var(--on-format)">
  <Key label="Auto per image" pressed={f.auto} onclick={() => (f.auto = !f.auto)} title="WebP for photos, PNG for graphics and screenshots" />
  <ColourKey label="Matte" value={f.matte} onchange={hex => (f.matte = hex)} />
</div>
{#if opaque}<span class="sr-only">Transparent areas are filled with the matte colour.</span>{/if}
