<!-- SPDX-License-Identifier: AGPL-3.0-or-later
  The four everyday channels. Each is a screen you act on directly, with keys beneath. -->
<script lang="ts">
  import Screen from '$lib/components/Screen.svelte';
  import { CHANNELS } from '$lib/modules/registry';
  import { recipe } from '$lib/state/recipe.svelte';
  import { ui } from '$lib/state/ui.svelte';
</script>

<div class="channels" style:--count={CHANNELS.length}>
  {#each CHANNELS as m (m.id)}
    {@const C = m.channel}
    <section class="channel panel" id="chan-{m.id}" aria-labelledby="ch-{m.id}" style:--accent="var(--accent-{m.accent})" style:--on-accent="var(--on-{m.accent})">
      <div class="channel__head">
        <span class="legend head" id="ch-{m.id}"><span class="glyph" aria-hidden="true">{C.glyph}</span>{m.label}<span class="sr-only">, key {C.slot}</span></span>
        {#if m.Details}
          <button type="button" class="channel__summary more" onclick={() => ui.reveal(m.id)} title="More {m.label.toLowerCase()} settings">{recipe.value ? m.summary(recipe.value) : ''} ›</button>
        {:else}
          <span class="channel__summary">{recipe.value ? m.summary(recipe.value) : ''}</span>
        {/if}
      </div>
      <Screen><C.Screen /></Screen>
      <div class="channel__controls"><C.Keys /></div>
    </section>
  {/each}
</div>

<style>
  .channels { display: grid; grid-template-columns: repeat(var(--count), minmax(0, 1fr)); gap: var(--gutter); }
  .channels :global(.channel) { grid-template-rows: auto 136px auto; min-width: 0; container: channel / inline-size; }
  .channels :global(.channel .screen) { padding: 8px 10px; }
  .head { display: flex; gap: 7px; align-items: center; }
  .glyph { color: var(--accent); font-size: 11px; }
  .channels :global(.channel__summary) { white-space: nowrap; overflow: hidden; text-overflow: ellipsis; min-width: 0; }
  .channels :global(.channel__head) { min-width: 0; }
  .more { all: unset; cursor: pointer; border-radius: 2px; margin-left: auto; font: var(--fs-small)/1 var(--font-data); color: var(--ink-2); }
  .more:hover { color: var(--ink-1); }
  .more:focus-visible { outline: var(--focus); outline-offset: 2px; }
  /* A narrow window: four keys to a row still fit their labels. */
  @container channel (max-width: 210px) {
    .channels :global(.channel__controls .keys) { gap: 4px; }
    .channels :global(.channel__controls .keys > .key) { padding: 0 1px; letter-spacing: .02em; }
  }
</style>
