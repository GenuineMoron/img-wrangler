<!-- SPDX-License-Identifier: AGPL-3.0-or-later
  The steps that aren't screens, as a row of keys in the order they run: which files to take,
  rotate, sharpen, watermark, metadata. A key opens its settings. An add-on's key carries a
  lamp that's lit while the step is on, and pressing one that's off switches it on. Switching
  it off again is the switch at the top of its settings. -->
<script lang="ts">
  import Icon from '$lib/components/Icon.svelte';
  import Led from '$lib/components/Led.svelte';
  import { addonOf, KEYS } from '$lib/modules/registry';
  import type { UiModule } from '$lib/modules/types';
  import { recipe } from '$lib/state/recipe.svelte';
  import { ui } from '$lib/state/ui.svelte';

  const r = $derived(recipe.value);

  function press(m: UiModule) {
    if (m.kind === 'addon' && r) {
      const a = addonOf(r, m);
      if (!a) { r.addons.push(m.create()); ui.more = m.id; return; }
      if (!a.enabled) { a.enabled = true; ui.more = m.id; return; }
    }
    ui.toggleMore(m.id);
  }
</script>

<nav class="steps" aria-label="More steps">
  {#each KEYS as m (m.id)}
    {@const on = m.kind === 'addon' ? !!addonOf(r, m)?.enabled : undefined}
    <button type="button" class="key step" id="more-{m.id}" aria-expanded={ui.more === m.id} aria-haspopup="dialog"
      title={m.blurb} onclick={() => press(m)}>
      <Icon name={m.icon} />
      <span>{m.label}</span>
      {#if on !== undefined}<Led {on} /><span class="sr-only">{on ? 'on' : 'off'}</span>{/if}
      {#if r && on !== false}<span class="sum" data-long>{m.summary(r)}</span>{/if}
    </button>
  {/each}
</nav>

<style>
  .steps { grid-column: 1; grid-row: 3; display: flex; gap: var(--sp-4); min-width: 0; }
  .step { flex: 0 1 auto; min-width: 0; gap: var(--sp-4); padding: 0 var(--sp-6) 0 var(--sp-5); }
  .sum { min-width: 0; overflow: hidden; text-overflow: ellipsis; font: var(--fs-small)/1 var(--font-data); letter-spacing: 0; text-transform: none; opacity: .72; }
  .step[aria-expanded='true'] { outline: 2px solid var(--ink-1); outline-offset: 1px; }
</style>
