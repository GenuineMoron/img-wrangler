<!-- SPDX-License-Identifier: AGPL-3.0-or-later
  Nine positions in a 3×3 pad: where the image sits in its frame, or which part a crop keeps. -->
<script lang="ts">
  import type { Anchor } from '$lib/types';

  let { value, onchange, label = 'Position' }: { value: Anchor; onchange: (a: Anchor) => void; label?: string } = $props();

  const GRID: { id: Anchor; name: string }[] = [
    { id: 'top_left', name: 'Top left' }, { id: 'top', name: 'Top' }, { id: 'top_right', name: 'Top right' },
    { id: 'left', name: 'Left' }, { id: 'center', name: 'Centre' }, { id: 'right', name: 'Right' },
    { id: 'bottom_left', name: 'Bottom left' }, { id: 'bottom', name: 'Bottom' }, { id: 'bottom_right', name: 'Bottom right' },
  ];
  let els: HTMLButtonElement[] = $state([]);
  const at = $derived(GRID.findIndex(g => g.id === value));

  function keydown(e: KeyboardEvent, i: number) {
    const d = { ArrowRight: 1, ArrowLeft: -1, ArrowDown: 3, ArrowUp: -3 }[e.key];
    if (d == null) return;
    e.preventDefault();
    const j = i + d;
    if (j < 0 || j > 8 || (Math.abs(d) === 1 && Math.floor(j / 3) !== Math.floor(i / 3))) return;
    els[j]?.focus();
    onchange(GRID[j].id);
  }
</script>

<div class="pad" role="radiogroup" aria-label={label}>
  {#each GRID as g, i (g.id)}
    <button bind:this={els[i]} type="button" role="radio" aria-checked={i === at} aria-label={g.name} tabindex={i === at ? 0 : -1}
      onclick={() => onchange(g.id)} onkeydown={e => keydown(e, i)}><i></i></button>
  {/each}
</div>

<style>
  .pad { display: grid; grid-template-columns: repeat(3, 20px); gap: 3px; padding: 4px; border-radius: var(--r-key); background: var(--surface-screen); box-shadow: inset 0 1px 3px rgba(0, 0, 0, .6), 0 1px 0 var(--seam-light); width: max-content; }
  button { all: unset; width: 20px; height: 20px; display: grid; place-items: center; cursor: pointer; border-radius: 2px; }
  i { width: 6px; height: 6px; border-radius: 1px; background: var(--screen-dim); }
  button:hover i { background: var(--screen-2); }
  button[aria-checked='true'] i { width: 10px; height: 10px; background: var(--accent); box-shadow: 0 0 var(--bloom) color-mix(in srgb, var(--accent) var(--bloom-amt), transparent); }
  button:focus-visible { outline: var(--focus-on-screen); }
</style>
