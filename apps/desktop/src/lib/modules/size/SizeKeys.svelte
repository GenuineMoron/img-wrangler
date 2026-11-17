<!-- SPDX-License-Identifier: AGPL-3.0-or-later -->
<script lang="ts">
  import KeyGroup from '$lib/components/KeyGroup.svelte';
  import { recipe } from '$lib/state/recipe.svelte';
  import { SHAPES } from './index';

  const size = $derived(recipe.value!.size);
</script>

<KeyGroup label="Shape" options={SHAPES} value={size.shape} accent="size" onchange={v => (size.shape = v)} />
