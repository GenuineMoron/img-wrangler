<!-- SPDX-License-Identifier: AGPL-3.0-or-later -->
<script lang="ts">
  import Switch from '$lib/components/Switch.svelte';
  import { recipe } from '$lib/state/recipe.svelte';

  const q = $derived(recipe.value!.quality);
  let lastLook: string | number = 'balanced';
  let lastLimit = 400;

  function lossless(on: boolean) {
    if (on) { lastLook = q.look === 'lossless' ? lastLook : q.look; q.look = 'lossless'; } else q.look = lastLook;
  }
  function limit(on: boolean) {
    if (on) q.max_kb = lastLimit;
    else { lastLimit = q.max_kb ?? lastLimit; q.max_kb = null; }
  }
</script>

<Switch plate label="Size limit" on={q.max_kb != null} onchange={limit} title="Keep every file under a size. Drag the red line on the screen to set it, or type it there" />
<Switch plate label="Lossless" on={q.look === 'lossless'} onchange={lossless} title="No quality loss, where the format allows" />
