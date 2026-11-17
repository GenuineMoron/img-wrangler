<!-- SPDX-License-Identifier: AGPL-3.0-or-later
  The contact sheet: every image in the current tab, drawn only where it can be seen so a batch
  of thousands scrolls as lightly as a dozen. Arrow keys move, Delete removes from the batch. -->
<script lang="ts">
  import { batch } from '$lib/state/batch.svelte';
  import { ui } from '$lib/state/ui.svelte';
  import Tile from './Tile.svelte';

  const GAP = 12, PAD = 14, MIN = 122, META = 38;
  let box: HTMLDivElement;
  let width = $state(600);
  let height = $state(400);
  let top = $state(0);

  const cols = $derived(Math.max(2, Math.floor((width - PAD * 2 + GAP) / (MIN + GAP))));
  const tileW = $derived((width - PAD * 2 - GAP * (cols - 1)) / cols);
  const rowH = $derived(tileW + META + GAP);
  const list = $derived(batch.shown);
  const rows = $derived(Math.ceil(list.length / cols));
  const first = $derived(Math.max(0, Math.floor((top - PAD) / rowH) - 1));
  const last = $derived(Math.min(rows, Math.ceil((top + height) / rowH) + 1));
  const visible = $derived(list.slice(first * cols, last * cols));

  function reveal(pos: number) {
    const y = PAD + Math.floor(pos / cols) * rowH;
    if (y < box.scrollTop) box.scrollTop = y - PAD;
    else if (y + rowH > box.scrollTop + height) box.scrollTop = y + rowH - height + PAD;
    queueMicrotask(() => box.querySelector<HTMLElement>('[aria-selected="true"]')?.focus({ preventScroll: true }));
  }

  function keydown(e: KeyboardEvent) {
    const pos = list.indexOf(batch.selected);
    const step = { ArrowRight: 1, ArrowLeft: -1, ArrowDown: cols, ArrowUp: -cols, Home: -1e9, End: 1e9 }[e.key];
    if (step != null) {
      e.preventDefault();
      const next = Math.max(0, Math.min(list.length - 1, pos + step));
      batch.select(batch.items[list[next]]?.source ?? null);
      reveal(next);
    } else if ((e.key === 'Delete' || e.key === 'Backspace') && batch.current && !batch.running) {
      e.preventDefault();
      const gone = batch.current.source;
      const nextIdx = list[Math.min(list.length - 1, pos + 1)] ?? list[pos - 1];
      batch.select(batch.items[nextIdx]?.source ?? null);
      batch.remove([gone]);
      ui.say('Removed from the batch. The file itself is untouched.');
    } else if (e.key === ' ') {
      e.preventDefault();
      ui.view = ui.view === 'original' ? 'auto' : 'original';
    }
  }
</script>

<div
  class="sheet"
  bind:this={box}
  bind:clientWidth={width}
  bind:clientHeight={height}
  onscroll={() => (top = box.scrollTop)}
  role="listbox"
  tabindex="-1"
  aria-label="Images, as they will come out"
  onkeydown={keydown}
>
  {#if list.length}
    <div class="spacer" style:height="{rows * rowH + PAD * 2 - GAP}px">
      {#each visible as idx, k (batch.items[idx].source)}
        {@const pos = first * cols + k}
        <Tile
          item={batch.items[idx]}
          status={batch.statuses[idx]}
          selected={idx === batch.selected}
          index={pos}
          total={list.length}
          left={PAD + (pos % cols) * (tileW + GAP)}
          top={PAD + Math.floor(pos / cols) * rowH}
          width={tileW}
          onselect={() => batch.select(batch.items[idx].source)}
        />
      {/each}
    </div>
  {:else}
    <p class="none">Nothing in this group.</p>
  {/if}
</div>

<style>
  .sheet { position: relative; overflow-y: auto; overflow-x: hidden; border-right: 1px solid var(--screen-line); scrollbar-width: thin; scrollbar-color: #333 transparent; z-index: 4; }
  .spacer { position: relative; }
  .none { padding: var(--sp-7); color: var(--screen-2); }
</style>
