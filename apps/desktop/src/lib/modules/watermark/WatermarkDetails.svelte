<!-- SPDX-License-Identifier: AGPL-3.0-or-later -->
<script lang="ts">
  import AnchorPad from '$lib/components/AnchorPad.svelte';
  import Field from '$lib/components/Field.svelte';
  import Key from '$lib/components/Key.svelte';
  import Row from '$lib/components/Row.svelte';
  import Slider from '$lib/components/Slider.svelte';
  import { backend } from '$lib/ipc';
  import { fileName } from '$lib/text';
  import type { Watermark } from '$lib/types';

  let { value }: { value: Watermark & { kind: 'watermark' } } = $props();

  async function choose() {
    const path = await backend.pickImage();
    if (path) value.image = path;
  }
</script>

<div class="details">
  <Row label="Logo" note={value.image ? undefined : 'A PNG or SVG with a transparent background works best.'}>
    <Field label="Logo file" value={value.image ? fileName(value.image) : ''} placeholder="no logo yet" readonly title={value.image ?? undefined} />
    <Key icon="image" label="Choose" onclick={choose} />
  </Row>
  <Row label="Position">
    <AnchorPad label="Where the logo goes" value={value.anchor} onchange={a => (value.anchor = a)} />
  </Row>
  <Row label="Size" note="Of each image’s shorter side, so portraits and landscapes match.">
    <Slider label="Logo size" min={5} max={60} value={Math.round(value.size)} unit="%" onchange={v => (value.size = v)}
      stops={[{ value: 5, label: '5%' }, { value: 20, label: '20' }, { value: 40, label: '40' }, { value: 60, label: '60%' }]} />
  </Row>
  <Row label="Opacity">
    <Slider label="Logo opacity" min={10} max={100} step={5} value={Math.round(value.opacity)} unit="%" onchange={v => (value.opacity = v)} />
  </Row>
  <Row label="Margin">
    <Slider label="Gap from the edges" min={0} max={10} step={0.5} value={value.margin} unit="%" onchange={v => (value.margin = v)} />
  </Row>
</div>
