<!-- SPDX-License-Identifier: AGPL-3.0-or-later -->
<script lang="ts">
  import Key from '$lib/components/Key.svelte';
  import KeyGroup from '$lib/components/KeyGroup.svelte';
  import Row from '$lib/components/Row.svelte';
  import type { Rotate } from '$lib/types';

  let { value }: { value: Rotate & { kind: 'rotate' } } = $props();
</script>

<div class="details">
  <Row label="Turn">
    <KeyGroup label="Turn clockwise" neutral value={value.turn} onchange={v => (value.turn = v)}
      options={[{ id: 0, label: '0°' }, { id: 90, label: '90°', aria: '90° clockwise' }, { id: 180, label: '180°' }, { id: 270, label: '270°', aria: '90° anticlockwise' }]} />
  </Row>
  <Row label="Flip">
    <Key icon="flipH" label="Across" pressed={value.flip_horizontal} onclick={() => (value.flip_horizontal = !value.flip_horizontal)} aria-label="Flip left to right" />
    <Key icon="flipV" label="Down" pressed={value.flip_vertical} onclick={() => (value.flip_vertical = !value.flip_vertical)} aria-label="Flip top to bottom" />
  </Row>
</div>
