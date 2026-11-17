<!-- SPDX-License-Identifier: AGPL-3.0-or-later -->
<script lang="ts">
  import Key from '$lib/components/Key.svelte';
  import { recipe } from '$lib/state/recipe.svelte';

  const q = $derived(recipe.value!.quality);
  let lastLook: string | number = 'balanced';
  let lastLimit = 400;

  function lossless() {
    if (q.look === 'lossless') q.look = lastLook;
    else { lastLook = q.look; q.look = 'lossless'; }
  }
  function limit() {
    if (q.max_kb) { lastLimit = q.max_kb; q.max_kb = null; } else q.max_kb = lastLimit;
  }
</script>

<div class="keys" style:--n="2" style:--accent="var(--accent-quality)" style:--on-accent="var(--on-quality)">
  <Key label="Max size" pressed={q.max_kb != null} onclick={limit} title="Cap every file at a size; drag the red line to set it" />
  <Key label="Lossless" pressed={q.look === 'lossless'} onclick={lossless} title="No quality loss, where the format allows" />
</div>
