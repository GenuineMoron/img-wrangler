<!-- SPDX-License-Identifier: AGPL-3.0-or-later
  A key: the one button. It latches (pressed), acts as a radio (checked) or just presses.
  Takes its colour from --accent on itself or any ancestor. -->
<script lang="ts">
  import type { Snippet } from 'svelte';
  import type { HTMLButtonAttributes } from 'svelte/elements';
  import Icon from './Icon.svelte';
  import type { IconName } from './icons';

  interface Props extends Omit<HTMLButtonAttributes, 'children'> {
    label?: string;
    icon?: IconName;
    /** A latching key (aria-pressed). */
    pressed?: boolean;
    /** A key in a radio group (role=radio). */
    checked?: boolean;
    dark?: boolean;
    tall?: boolean;
    /** Inserts something instead of latching; printed with a +. */
    insert?: boolean;
    children?: Snippet;
    el?: HTMLButtonElement;
  }

  let { label, icon, pressed, checked, dark, tall, insert, children, el = $bindable(), class: cls = '', ...rest }: Props = $props();
  const iconOnly = $derived(!!icon && !label && !children);
  // Each key gets its own patch of grain so a row of keys never looks stamped.
  const grain = $derived(((label ?? icon ?? '').length * 37) % 256);
</script>

<button
  bind:this={el}
  type="button"
  class="key {cls}"
  class:key--dark={dark}
  class:key--tall={tall}
  class:key--icon={iconOnly}
  class:key--insert={insert}
  role={checked !== undefined ? 'radio' : undefined}
  aria-checked={checked}
  aria-pressed={pressed}
  style:--gx="{grain}px"
  style:--gy="{(grain * 2.4) % 256}px"
  {...rest}
>
  {#if icon}<Icon name={icon} />{/if}
  {#if label}<span>{label}</span>{/if}
  {@render children?.()}
</button>
