<!-- SPDX-License-Identifier: AGPL-3.0-or-later
  Tabs printed on a screen, each with a count. Alarm tabs light their count red. -->
<script lang="ts" generics="T extends string">
  let { label, items, value, onchange }: {
    label: string;
    items: { id: T; label: string; n?: number; alarm?: boolean }[];
    value: T;
    onchange: (v: T) => void;
  } = $props();

  let els: HTMLButtonElement[] = $state([]);

  function keydown(e: KeyboardEvent, i: number) {
    const step = { ArrowRight: 1, ArrowLeft: -1 }[e.key];
    if (!step) return;
    e.preventDefault();
    const j = (i + step + items.length) % items.length;
    els[j]?.focus();
    onchange(items[j].id);
  }
</script>

<div class="tabs" role="tablist" aria-label={label}>
  {#each items as t, i (t.id)}
    <button
      bind:this={els[i]}
      type="button"
      role="tab"
      class="tab"
      class:tab--alarm={t.alarm && !!t.n}
      aria-selected={t.id === value}
      tabindex={t.id === value ? 0 : -1}
      onclick={() => onchange(t.id)}
      onkeydown={e => keydown(e, i)}
    >{t.label}{#if t.n != null}<b>{t.n.toLocaleString('en')}</b>{/if}</button>
  {/each}
</div>
