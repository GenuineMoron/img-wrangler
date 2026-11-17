<!-- SPDX-License-Identifier: AGPL-3.0-or-later
  Everything about Size that isn't on its channel: trimming edges, the backdrop and how much of
  the frame the picture fills, where it sits, other shapes, and enlarging small images. -->
<script lang="ts">
  import AnchorPad from '$lib/components/AnchorPad.svelte';
  import ColourKey from '$lib/components/ColourKey.svelte';
  import Field from '$lib/components/Field.svelte';
  import Key from '$lib/components/Key.svelte';
  import KeyGroup from '$lib/components/KeyGroup.svelte';
  import Row from '$lib/components/Row.svelte';
  import Slider from '$lib/components/Slider.svelte';
  import { parseShape, shapeText } from '$lib/geometry';
  import { recipe } from '$lib/state/recipe.svelte';
  import type { Fit } from '$lib/types';
  import { FITS } from './index';

  const size = $derived(recipe.value!.size);
  const shape = $derived(parseShape(size.shape) ?? { kind: 'original' as const });
  const pick = $derived(size.pad.backdrop.startsWith('#') && size.pad.backdrop !== '#ffffff' ? size.pad.backdrop : '#c9d8e6');

  let shapeText_ = $state('');
  let bad = $state(false);
  $effect(() => { shapeText_ = shape.kind === 'original' ? '' : shapeText(shape).replace('x', '×'); });

  function applyShape() {
    const parsed = parseShape(shapeText_);
    bad = !parsed;
    if (parsed) size.shape = shapeText(parsed);
  }

  const BACKDROPS = [
    { id: 'auto', label: 'Auto', aria: 'Auto: the photo’s own background colour', title: 'Matches the photo’s own background' },
    { id: '#ffffff', label: 'White' },
    { id: 'transparent', label: 'Clear', aria: 'Clear: transparent, where the format allows' },
  ];
</script>

<div class="details" style:--accent="var(--accent-trim)" style:--on-accent="var(--on-trim)">
  <Row label="Trim">
    <Key icon="trim" label="Trim empty edges" pressed={!!size.trim} title="Cut the empty background around the subject before sizing"
      onclick={() => (size.trim = size.trim ? null : { tolerance: 4, margin: 0 })} />
  </Row>
  {#if size.trim}
    <Row label="Tolerance">
      <Slider label="Trim tolerance" min={0} max={30} value={size.trim.tolerance} unit="%" onchange={v => (size.trim!.tolerance = v)}
        stops={[{ value: 0, label: 'exact' }, { value: 10, label: '10' }, { value: 20, label: '20' }, { value: 30, label: 'loose' }]} />
    </Row>
    <Row label="Keep margin">
      <Field label="Margin to keep around the subject, in pixels" type="number" min="0" max="2000" unit="px" value={size.trim.margin}
        oninput={e => (size.trim!.margin = Math.max(0, Number(e.currentTarget.value) || 0))} />
    </Row>
  {/if}
</div>

{#if shape.kind !== 'original'}
  <div class="details" style:--accent="var(--accent-canvas)" style:--on-accent="var(--on-canvas)">
    <Row label="Meets frame">
      <KeyGroup label="How the picture meets the frame" options={FITS.map(f => ({ id: f.id, label: f.label, aria: f.aria }))} value={size.fit} accent="size" onchange={(v: Fit) => (size.fit = v)} />
    </Row>
    {#if size.fit === 'pad'}
      <Row label="Backdrop">
        <div class="keys" role="radiogroup" aria-label="Backdrop" style:--n="4">
          {#each BACKDROPS as b (b.id)}
            <Key label={b.label} checked={size.pad.backdrop === b.id} aria-label={b.aria} title={b.title} onclick={() => (size.pad.backdrop = b.id)} />
          {/each}
          <ColourKey label="Pick" value={pick} checked={size.pad.backdrop === pick} onchange={hex => (size.pad.backdrop = hex)} />
        </div>
      </Row>
      <Row label="Fills">
        <Slider label="How much of the frame the picture fills" min={50} max={100} value={Math.round(size.pad.fill)} unit="%" onchange={v => (size.pad.fill = v)}
          stops={[{ value: 50, label: '50%' }, { value: 70, label: '70' }, { value: 85, label: '85' }, { value: 100, label: '100%' }]} />
      </Row>
    {/if}
    {#if size.fit !== 'inside'}
      <Row label={size.fit === 'crop' ? 'Keep' : 'Position'}>
        <AnchorPad label={size.fit === 'crop' ? 'Which part to keep' : 'Where the picture sits'} value={size.anchor} onchange={a => (size.anchor = a)} />
      </Row>
    {/if}
  </div>
{/if}

<div class="details" style:--accent="var(--accent-size)" style:--on-accent="var(--on-size)">
  <Row label="Other shape" note={bad ? 'Use a ratio like 2:3 or a size like 1200×628.' : undefined}>
    <Field label="Other shape: a ratio or exact size" placeholder="2:3 or 1200×628" bind:value={shapeText_} aria-invalid={bad}
      onkeydown={e => e.key === 'Enter' && applyShape()} onblur={applyShape} />
  </Row>
  <Row label="Small images">
    <Key label="Enlarge to size" pressed={size.upscale} onclick={() => (size.upscale = !size.upscale)} title="Images smaller than the size are enlarged; otherwise they stay as they are" />
  </Row>
</div>
