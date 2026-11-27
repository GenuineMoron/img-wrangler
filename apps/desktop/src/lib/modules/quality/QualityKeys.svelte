<!-- SPDX-License-Identifier: AGPL-3.0-or-later
  Under the Quality screen: the size limit, with the limit itself typed beside its switch (in KB
  or MB), and lossless. -->
<script lang="ts">
  import Switch from '$lib/components/Switch.svelte';
  import { recipe } from '$lib/state/recipe.svelte';
  import { kbText, parseKb } from './index';

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

  // The field shows the limit until it's typed in. A size it can't read is marked, not applied.
  let text = $state('');
  let bad = $state(false);
  let editing = false;
  $effect(() => { if (!editing) text = q.max_kb ? kbText(q.max_kb) : ''; });

  function apply() {
    const kb = parseKb(text);
    bad = !kb;
    if (kb) q.max_kb = kb;
  }
</script>

<div class="plate" title="Keep every file under a size. Type it here, or drag the red line on the screen">
  <span class="plate__label">Size limit</span>
  {#if q.max_kb != null}
    <input class="plate__field" aria-label="Size limit, like 400 KB or 1.5 MB" spellcheck="false" autocomplete="off" inputmode="decimal" aria-invalid={bad}
      bind:value={text} onfocus={e => { editing = true; e.currentTarget.select(); }} oninput={() => (bad = false)}
      onblur={() => { editing = false; apply(); if (!bad) text = kbText(q.max_kb!); }}
      onkeydown={e => { if (e.key === 'Enter') e.currentTarget.blur(); if (e.key === 'Escape') { bad = false; text = kbText(q.max_kb!); e.currentTarget.blur(); } }} />
  {/if}
  <Switch quiet label="Size limit" on={q.max_kb != null} onchange={limit} />
</div>
<Switch plate label="Lossless" on={q.look === 'lossless'} onchange={lossless} title="No quality loss, where the format allows" />
