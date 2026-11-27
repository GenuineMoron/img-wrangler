<!-- SPDX-License-Identifier: AGPL-3.0-or-later
  A slide switch: on or off, and it looks it. Every yes-or-no setting is one of these, so a key
  is always something you press to do a thing. `plate` fills its cell with the label on the left
  and the switch on the right, to sit in a row with keys. -->
<script lang="ts">
  import type { HTMLButtonAttributes } from 'svelte/elements';
  import Icon from './Icon.svelte';
  import type { IconName } from './icons';

  let { label, on, onchange, icon, plate = false, quiet = false, class: cls = '', ...rest }: Omit<HTMLButtonAttributes, 'onchange'> & {
    label: string;
    on: boolean;
    onchange: (on: boolean) => void;
    icon?: IconName;
    plate?: boolean;
    /** The label is for screen readers only, as something beside the switch says it. */
    quiet?: boolean;
  } = $props();
</script>

<button type="button" class="switch {cls}" class:switch--plate={plate} role="switch" aria-checked={on} onclick={() => onchange(!on)} {...rest}>
  {#if icon}<Icon name={icon} />{/if}
  <span class="switch__label" class:sr-only={quiet}>{label}</span>
  <span class="switch__slot" aria-hidden="true"><i class="switch__cap"></i></span>
</button>
