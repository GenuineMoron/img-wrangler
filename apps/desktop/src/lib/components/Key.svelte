<!-- SPDX-License-Identifier: AGPL-3.0-or-later
  A key: you press it and something happens. It never stays down. A setting that's on or off
  is a Switch, and a choice of one from several is a Segmented selector. -->
<script lang="ts">
  import type { Snippet } from 'svelte';
  import type { HTMLButtonAttributes } from 'svelte/elements';
  import Icon from './Icon.svelte';
  import type { IconName } from './icons';

  interface Props extends Omit<HTMLButtonAttributes, 'children'> {
    label?: string;
    icon?: IconName;
    dark?: boolean;
    tall?: boolean;
    /** Adds something; printed with a +. */
    insert?: boolean;
    children?: Snippet;
    el?: HTMLButtonElement;
  }

  let { label, icon, dark, tall, insert, children, el = $bindable(), class: cls = '', ...rest }: Props = $props();
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
  style:--gx="{grain}px"
  style:--gy="{(grain * 2.4) % 256}px"
  {...rest}
>
  {#if icon}<Icon name={icon} />{/if}
  {#if label}<span>{label}</span>{/if}
  {@render children?.()}
</button>
