<!-- SPDX-License-Identifier: AGPL-3.0-or-later
  An OLED screen: black glass with a pixel grid, glow and a reflection. Photos shown on a screen
  sit above the effects so nothing is ever drawn over an image. -->
<script lang="ts">
  import type { Snippet } from 'svelte';
  import type { HTMLAttributes } from 'svelte/elements';

  let { fx = true, children, class: cls = '', ...rest }: HTMLAttributes<HTMLDivElement> & { fx?: boolean; children?: Snippet } = $props();
</script>

<div class="screen {cls}" class:screen--fx={fx} {...rest}>
  {@render children?.()}
</div>
