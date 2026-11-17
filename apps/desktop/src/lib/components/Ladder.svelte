<!-- SPDX-License-Identifier: AGPL-3.0-or-later
  An LED ladder: a row of segments, `lit` of them on. Decorative; the reading is said in text. -->
<script lang="ts">
  let { lit, segs = 24, kind = 'in' }: { lit: number; segs?: number; kind?: 'in' | 'out' } = $props();
  const n = $derived(Math.max(0, Math.min(segs, Math.round(lit))));
</script>

<div class="ladder ladder--{kind}" style:--segs={segs} aria-hidden="true">
  {#each { length: segs } as _, i (i)}<i class:on={i < n}></i>{/each}
</div>
