<!-- SPDX-License-Identifier: AGPL-3.0-or-later
  A row of radio keys. Arrow keys move and select, as a radio group should; the lit key takes
  the group's accent. -->
<script lang="ts" module>
  import type { IconName } from './icons';

  export interface Option<V> { id: V; label?: string; icon?: IconName; aria?: string; title?: string; disabled?: boolean }
</script>

<script lang="ts" generics="T extends string | number">
  import Key from './Key.svelte';

  let { label, options, value, onchange, accent, neutral = false, columns }: {
    label: string;
    options: Option<T>[];
    value: T | null;
    onchange: (v: T) => void;
    /** An accent name from tokens.css: size, format, quality, name, trim, canvas, tool. */
    accent?: string;
    neutral?: boolean;
    columns?: number;
  } = $props();

  let keys: HTMLButtonElement[] = $state([]);
  const current = $derived(options.findIndex(o => o.id === value));

  function onkeydown(e: KeyboardEvent, i: number) {
    const step = { ArrowRight: 1, ArrowDown: 1, ArrowLeft: -1, ArrowUp: -1 }[e.key];
    if (!step) return;
    e.preventDefault();
    for (let n = 1; n <= options.length; n++) {
      const j = (i + step * n + options.length) % options.length;
      if (!options[j].disabled) { keys[j]?.focus(); onchange(options[j].id); return; }
    }
  }
</script>

<div
  class="keys"
  class:keys--neutral={neutral}
  role="radiogroup"
  aria-label={label}
  style:--n={columns ?? options.length}
  style:--accent={accent ? `var(--accent-${accent})` : undefined}
  style:--on-accent={accent ? `var(--on-${accent})` : undefined}
>
  {#each options as o, i (o.id)}
    <Key
      bind:el={keys[i]}
      label={o.label}
      icon={o.icon}
      checked={o.id === value}
      aria-label={o.aria}
      title={o.title}
      disabled={o.disabled}
      tabindex={i === current || (current < 0 && i === 0) ? 0 : -1}
      onclick={() => onchange(o.id)}
      onkeydown={(e: KeyboardEvent) => onkeydown(e, i)}
    />
  {/each}
</div>
