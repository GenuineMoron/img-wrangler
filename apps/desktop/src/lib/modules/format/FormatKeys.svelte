<!-- SPDX-License-Identifier: AGPL-3.0-or-later -->
<script lang="ts">
  import ColourKey from '$lib/components/ColourKey.svelte';
  import Switch from '$lib/components/Switch.svelte';
  import { catalog } from '$lib/state/catalog.svelte';
  import { recipe } from '$lib/state/recipe.svelte';

  const f = $derived(recipe.value!.format);
  const info = $derived(f.id === 'same' ? undefined : catalog.info(f.id));
  // Picking per image chooses WebP or PNG, and both keep clear areas clear. "Same" depends on
  // each image, so the colour stays ready for the ones that can't.
  const keepsClear = $derived(f.auto || info?.alpha === true);
</script>

<Switch plate label="Pick per image" on={f.auto} onchange={on => (f.auto = on)} title="Chooses for each image: WebP for photos, PNG for graphics and screenshots" />
<ColourKey plate label="Behind clear areas" value={f.matte} onchange={hex => (f.matte = hex)} disabled={keepsClear}
  title={keepsClear ? `${f.auto ? 'WebP and PNG keep' : `${info?.label ?? 'This format'} keeps`} see-through areas see-through, so nothing goes behind them` : 'The colour behind see-through areas, for formats like JPEG that have none'} />
