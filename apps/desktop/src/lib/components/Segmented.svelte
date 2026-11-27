<!-- SPDX-License-Identifier: AGPL-3.0-or-later
  A selector: one of several. The places sit in one recessed track and a raised cap marks the
  chosen one, so it never reads as a row of separate keys. Arrow keys move and choose, as a
  radio group should. The cap takes the group's accent. -->
<script lang="ts" module>
  import type { IconName } from './icons';

  export interface Option<V> { id: V; label?: string; icon?: IconName; aria?: string; title?: string; disabled?: boolean }
</script>

<script lang="ts" generics="T extends string | number">
  import Icon from './Icon.svelte';

  let { label, options, value, onchange, accent, neutral = false, class: cls = '' }: {
    label: string;
    options: Option<T>[];
    /** Nothing chosen when null or not among the options. */
    value: T | null;
    onchange: (v: T) => void;
    /** An accent name from tokens.css: size, format, quality, name, trim, canvas, tool. */
    accent?: string;
    /** A dark cap, for choices that belong to no channel. */
    neutral?: boolean;
    class?: string;
  } = $props();

  let els: HTMLButtonElement[] = $state([]);
  const at = $derived(options.findIndex(o => o.id === value));

  function onkeydown(e: KeyboardEvent, i: number) {
    const step = { ArrowRight: 1, ArrowDown: 1, ArrowLeft: -1, ArrowUp: -1 }[e.key];
    if (!step) return;
    e.preventDefault();
    for (let n = 1; n <= options.length; n++) {
      const j = (i + step * n + options.length) % options.length;
      if (!options[j].disabled) { els[j]?.focus(); onchange(options[j].id); return; }
    }
  }
</script>

<div
  class="seg {cls}"
  class:seg--neutral={neutral}
  role="radiogroup"
  aria-label={label}
  style:--n={options.length}
  style:--at={at}
  style:--accent={accent ? `var(--accent-${accent})` : undefined}
  style:--on-accent={accent ? `var(--on-${accent})` : undefined}
>
  {#if at >= 0}<i class="seg__cap" aria-hidden="true"></i>{/if}
  {#each options as o, i (o.id)}
    <button
      bind:this={els[i]}
      type="button"
      role="radio"
      aria-checked={i === at}
      aria-label={o.aria}
      title={o.title}
      disabled={o.disabled}
      tabindex={i === at || (at < 0 && i === 0) ? 0 : -1}
      onclick={() => onchange(o.id)}
      onkeydown={e => onkeydown(e, i)}
    >{#if o.icon}<Icon name={o.icon} />{/if}{#if o.label}<span>{o.label}</span>{/if}</button>
  {/each}
</div>
