<!-- SPDX-License-Identifier: AGPL-3.0-or-later -->
<script lang="ts">
  import Switch from '$lib/components/Switch.svelte';
  import Row from '$lib/components/Row.svelte';
  import Slider from '$lib/components/Slider.svelte';
  import { recipe } from '$lib/state/recipe.svelte';

  const q = $derived(recipe.value!.quality);
  const fixed = $derived(typeof q.look === 'number');
</script>

<div class="details" style:--accent="var(--accent-quality)" style:--on-accent="var(--on-quality)">
  <Switch plate label="Shrink the image to meet the limit" on={q.allow_downscale} disabled={!q.max_kb} onchange={on => (q.allow_downscale = on)} />
  <p class="note">{q.max_kb ? `Quality drops until each file is under ${q.max_kb} KB. If that isn’t enough, this makes the image smaller too.` : 'Needs the size limit, switched on under the Quality screen.'}</p>
  <Switch plate label="Keep the original if it’s smaller" on={q.never_bigger} onchange={on => (q.never_bigger = on)} />
  <p class="note">When the new file wouldn’t be smaller, the original is kept as it is.</p>
  <Switch plate label="Set the quality by hand" on={fixed} onchange={on => (q.look = on ? 80 : 'balanced')} />
  {#if fixed}<p class="note">A raw encoder setting instead of a measured look.</p>{/if}
  {#if typeof q.look === 'number'}
    <Row label="Encoder">
      <Slider label="Encoder quality" min={1} max={100} value={q.look} onchange={v => (q.look = v)}
        stops={[{ value: 1, label: '1' }, { value: 50, label: '50' }, { value: 80, label: '80' }, { value: 100, label: '100' }]} />
    </Row>
  {/if}
  <Switch plate label="Progressive JPEG" on={q.progressive} onchange={on => (q.progressive = on)} title="Loads blurry to sharp, and is usually a little smaller" />
</div>
