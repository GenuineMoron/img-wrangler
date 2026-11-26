<!-- SPDX-License-Identifier: AGPL-3.0-or-later -->
<script lang="ts">
  import Key from '$lib/components/Key.svelte';
  import Row from '$lib/components/Row.svelte';
  import Slider from '$lib/components/Slider.svelte';
  import { recipe } from '$lib/state/recipe.svelte';

  const q = $derived(recipe.value!.quality);
  const fixed = $derived(typeof q.look === 'number');
</script>

<div class="details" style:--accent="var(--accent-quality)" style:--on-accent="var(--on-quality)">
  <Row label="Over the limit" note={q.max_kb ? `With Max size on, quality drops until each file is under ${q.max_kb} KB. If that isn’t enough, this makes the image smaller too.` : 'Needs Max size, on the Quality screen.'}>
    <Key label="Shrink to fit" pressed={q.allow_downscale} disabled={!q.max_kb} onclick={() => (q.allow_downscale = !q.allow_downscale)} />
  </Row>
  <Row label="Never bigger" note="Keeps the original when the new file wouldn’t be smaller.">
    <Key label="Keep originals" pressed={q.never_bigger} onclick={() => (q.never_bigger = !q.never_bigger)} />
  </Row>
  <Row label="Fixed quality" note={fixed ? 'A raw encoder setting instead of a measured look.' : undefined}>
    <Key label="Set by hand" pressed={fixed} onclick={() => (q.look = fixed ? 'balanced' : 80)} />
  </Row>
  {#if typeof q.look === 'number'}
    <Row label="Encoder">
      <Slider label="Encoder quality" min={1} max={100} value={q.look} onchange={v => (q.look = v)}
        stops={[{ value: 1, label: '1' }, { value: 50, label: '50' }, { value: 80, label: '80' }, { value: 100, label: '100' }]} />
    </Row>
  {/if}
  <Row label="JPG loading">
    <Key label="Progressive" pressed={q.progressive} onclick={() => (q.progressive = !q.progressive)} title="Loads blurry to sharp, and is usually a little smaller" />
  </Row>
</div>
