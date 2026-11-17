<!-- SPDX-License-Identifier: AGPL-3.0-or-later -->
<script lang="ts">
  import Row from '$lib/components/Row.svelte';
  import Slider from '$lib/components/Slider.svelte';
  import type { Sharpen } from '$lib/types';

  let { value }: { value: Sharpen & { kind: 'sharpen' } } = $props();
</script>

<div class="details">
  <Row label="Amount">
    <Slider label="Sharpen amount" min={0} max={200} step={5} value={value.amount} unit="%" onchange={v => (value.amount = v)}
      stops={[{ value: 0, label: '0' }, { value: 40, label: '40' }, { value: 100, label: '100' }, { value: 200, label: '200%' }]} />
  </Row>
  <Row label="Radius">
    <Slider label="Sharpen radius" min={0.3} max={3} step={0.1} value={value.radius} unit=" px" valueText={v => `${v.toFixed(1)} pixels`} onchange={v => (value.radius = Math.round(v * 10) / 10)}
      stops={[{ value: 0.3, label: 'fine' }, { value: 1, label: '1' }, { value: 2, label: '2' }, { value: 3, label: 'wide' }]} />
  </Row>
  <Row label="Threshold" note="Leaves smooth areas alone, so skin and skies stay clean.">
    <Slider label="Sharpen threshold" min={0} max={40} value={value.threshold} onchange={v => (value.threshold = v)} />
  </Row>
</div>
