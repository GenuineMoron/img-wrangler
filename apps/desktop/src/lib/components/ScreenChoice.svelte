<!-- SPDX-License-Identifier: AGPL-3.0-or-later
  A choice printed on a screen (a ruler of sizes, a list of formats): a radio group with arrow
  keys. The lit item takes the screen's --accent. -->
<script lang="ts" module>
  export interface Choice<V> { id: V; label: string; aria?: string; title?: string; disabled?: boolean }
</script>

<script lang="ts" generics="T">

  let { label, items, value, onchange, layout = 'row', class: cls = '' }: {
    label: string;
    items: Choice<T>[];
    /** Nothing lit when undefined or not among the items. */
    value: T | undefined;
    onchange: (v: T) => void;
    layout?: 'row' | 'column' | 'spread';
    class?: string;
  } = $props();

  let els: HTMLButtonElement[] = $state([]);
  const at = $derived(items.findIndex(i => i.id === value));

  function keydown(e: KeyboardEvent, i: number) {
    const step = { ArrowRight: 1, ArrowDown: 1, ArrowLeft: -1, ArrowUp: -1 }[e.key];
    if (!step) return;
    e.preventDefault();
    e.stopPropagation();
    for (let n = 1; n <= items.length; n++) {
      const j = (i + step * n + items.length) % items.length;
      if (!items[j].disabled) { els[j]?.focus(); onchange(items[j].id); return; }
    }
  }
</script>

<div class="choice choice--{layout} {cls}" role="radiogroup" aria-label={label}>
  {#each items as item, i (item.id)}
    <button
      bind:this={els[i]}
      type="button"
      role="radio"
      aria-checked={i === at}
      aria-label={item.aria}
      title={item.title}
      disabled={item.disabled}
      tabindex={i === at || (at < 0 && i === 0) ? 0 : -1}
      onclick={() => onchange(item.id)}
      onkeydown={e => keydown(e, i)}
    >{item.label}</button>
  {/each}
</div>

<style>
  .choice { display: flex; gap: 2px; }
  .choice--column { flex-direction: column; gap: 1px; }
  .choice--spread { justify-content: space-between; }
  button { all: unset; cursor: pointer; padding: 1px 5px; border-radius: 2px; font: 10.5px/1.35 var(--font-data); color: var(--screen-3); white-space: nowrap; }
  .choice--column button { font-size: var(--fs-small); color: var(--screen-2); }
  button:hover { color: var(--screen-1); background: #1a1a1a; }
  button[aria-checked='true'] { background: var(--accent); color: var(--on-accent); }
  button:disabled { cursor: default; opacity: .4; }
  button:focus-visible { outline: var(--focus-on-screen); outline-offset: 1px; }
</style>
