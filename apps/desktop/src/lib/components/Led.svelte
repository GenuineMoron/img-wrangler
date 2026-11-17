<!-- SPDX-License-Identifier: AGPL-3.0-or-later
  A status LED set into the case. Decorative: always say the state in text beside it. -->
<script lang="ts">
  let { on = false, color }: { on?: boolean; color?: string } = $props();
</script>

<span class="led" class:led--on={on} style:--led={color} aria-hidden="true"></span>
