<!-- SPDX-License-Identifier: AGPL-3.0-or-later -->
<script lang="ts">
  import Segmented from '$lib/components/Segmented.svelte';
  import Switch from '$lib/components/Switch.svelte';
  import Row from '$lib/components/Row.svelte';
  import type { Rotate } from '$lib/types';

  let { value }: { value: Rotate & { kind: 'rotate' } } = $props();
</script>

<div class="details">
  <Row label="Turn">
    <Segmented label="Turn clockwise" neutral value={value.turn} onchange={v => (value.turn = v)}
      options={[{ id: 0, label: '0°' }, { id: 90, label: '90°', aria: '90° clockwise' }, { id: 180, label: '180°' }, { id: 270, label: '270°', aria: '90° anticlockwise' }]} />
  </Row>
  <Switch plate icon="flipH" label="Flip left to right" on={value.flip_horizontal} onchange={on => (value.flip_horizontal = on)} />
  <Switch plate icon="flipV" label="Flip top to bottom" on={value.flip_vertical} onchange={on => (value.flip_vertical = on)} />
</div>
