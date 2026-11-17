<!-- SPDX-License-Identifier: AGPL-3.0-or-later -->
<script lang="ts">
  import { ICONS, type IconName } from './icons';

  let { name, label, size = 16 }: { name: IconName; label?: string; size?: number } = $props();
</script>

<svg viewBox="0 0 16 16" width={size} height={size} class="icon" role={label ? 'img' : undefined} aria-label={label} aria-hidden={label ? undefined : 'true'}>
  {@html ICONS[name]}
</svg>

<style>
  .icon { flex: none; fill: none; stroke: currentColor; stroke-width: 1.5; stroke-linecap: round; stroke-linejoin: round; }
</style>
