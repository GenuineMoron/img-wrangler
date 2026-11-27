<!-- SPDX-License-Identifier: AGPL-3.0-or-later
  A labelled line in a details panel: a printed legend, the control, an optional note. -->
<script lang="ts">
  import type { Snippet } from 'svelte';

  let { label, note, children, wide = false }: { label: string; note?: string; children: Snippet; wide?: boolean } = $props();
</script>

<div class="row" class:row--wide={wide}>
  <span class="legend legend--quiet row__label">{label}</span>
  <div class="row__control">{@render children()}</div>
  {#if note}<p class="note row__note">{note}</p>{/if}
</div>

<style>
  .row { display: grid; grid-template-columns: 84px minmax(0, 1fr); align-items: center; gap: var(--sp-3) var(--sp-5); }
  .row--wide { grid-template-columns: minmax(0, 1fr); }
  .row__label { line-height: 1.25; }
  .row__control { min-width: 0; display: flex; gap: var(--sp-3); align-items: center; }
  .row__control > :global(.keys), .row__control > :global(.seg), .row__control > :global(.slider-wrap), .row__control > :global(.field) { flex: 1; }
  /* A key keeps room for its whole label; the field beside it gives way instead. */
  .row__control > :global(.key) { flex: none; }
  .row__control > :global(.field) { min-width: 64px; }
  .row__note { grid-column: 2; margin-top: -2px; }
  .row--wide .row__note { grid-column: 1; }
</style>
