<!-- SPDX-License-Identifier: AGPL-3.0-or-later
  Everything about Size that isn't on its panel: any other shape (a ratio, or exact pixels),
  trimming edges, the backdrop and how much of the frame the picture fills, and where it sits.
  (How big, the usual shapes, crop, pad or inside, and enlarging are on the panel.) -->
<script lang="ts">
  import AnchorPad from '$lib/components/AnchorPad.svelte';
  import ColourKey from '$lib/components/ColourKey.svelte';
  import Field from '$lib/components/Field.svelte';
  import Key from '$lib/components/Key.svelte';
  import Row from '$lib/components/Row.svelte';
  import Segmented from '$lib/components/Segmented.svelte';
  import Slider from '$lib/components/Slider.svelte';
  import Switch from '$lib/components/Switch.svelte';
  import { parseShape } from '$lib/geometry';
  import { recipe } from '$lib/state/recipe.svelte';

  const size = $derived(recipe.value!.size);
  const shape = $derived(parseShape(size.shape) ?? { kind: 'original' as const });
  const pick = $derived(size.pad.backdrop.startsWith('#') && size.pad.backdrop !== '#ffffff' ? size.pad.backdrop : '#c9d8e6');

  // The shape as two numbers. Small numbers are a ratio, and the size on the screen says how
  // big. Numbers from 100 up are pixels, and that is the size.
  const PIXELS = 100;
  let w = $state(''), h = $state('');
  $effect(() => {
    [w, h] = shape.kind === 'ratio' ? [String(shape.a), String(shape.b)] : shape.kind === 'exact' ? [String(shape.w), String(shape.h)] : ['', ''];
  });

  function setShape(a: number, b: number) {
    size.shape = a >= PIXELS && b >= PIXELS ? `${a}x${b}` : `${a}:${b}`;
  }
  function apply() {
    const [a, b] = [Math.round(Number(w)), Math.round(Number(h))];
    if (w === '' && h === '') size.shape = 'original';
    else if (a > 0 && b > 0) setShape(a, b);
  }
  const reading = $derived(
    shape.kind === 'exact' ? `Every image comes out exactly ${shape.w} × ${shape.h} pixels.`
    : shape.kind === 'ratio' ? `Every image comes out ${shape.a}:${shape.b}. The size on the screen says how big.`
    : 'Each image keeps its own shape. Type a ratio like 2 : 3, or pixels like 1920 : 1080 for an exact size.',
  );

  const RATIOS: [number, number][] = [[4, 3], [5, 4], [2, 3], [9, 16], [21, 9], [1920, 1080], [1080, 1350]];
  const BACKDROPS = [
    { id: 'auto', label: 'Auto', aria: 'Auto: the photo’s own background colour', title: 'Matches the photo’s own background' },
    { id: '#ffffff', label: 'White' },
    { id: 'transparent', label: 'Clear', aria: 'Clear: transparent, where the format allows' },
    { id: 'pick', label: 'Colour', aria: 'A colour you choose' },
  ];
  const backdrop = $derived(BACKDROPS.some(b => b.id === size.pad.backdrop) ? size.pad.backdrop : 'pick');
</script>

<div class="details" style:--accent="var(--accent-size)" style:--on-accent="var(--on-size)">
  <Row label="Shape" note={reading}>
    <Field label="Shape: width" inputmode="numeric" placeholder="W" bind:value={w} onchange={apply} />
    <span class="colon" aria-hidden="true">:</span>
    <Field label="Shape: height" inputmode="numeric" placeholder="H" bind:value={h} onchange={apply} />
    <Key icon="swap" aria-label="Swap width and height" title="Turn it on its side" disabled={shape.kind === 'original'}
      onclick={() => shape.kind === 'ratio' ? setShape(shape.b, shape.a) : shape.kind === 'exact' && setShape(shape.h, shape.w)} />
  </Row>
  <div class="quick" role="group" aria-label="Common shapes">
    {#each RATIOS as [a, b] (`${a}:${b}`)}
      <Key label={a >= PIXELS ? `${a}×${b}` : `${a}:${b}`} onclick={() => setShape(a, b)} />
    {/each}
  </div>
</div>

<div class="details" style:--accent="var(--accent-trim)" style:--on-accent="var(--on-trim)">
  <Switch plate icon="trim" label="Trim empty edges first" on={!!size.trim} title="Cut the empty background around the subject before sizing"
    onchange={on => (size.trim = on ? { tolerance: 4, margin: 0 } : null)} />
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

{#if shape.kind !== 'original' && size.fit !== 'inside'}
  <div class="details" style:--accent="var(--accent-canvas)" style:--on-accent="var(--on-canvas)">
    {#if size.fit === 'pad'}
      <Row label="Backdrop">
        <Segmented label="Backdrop" options={BACKDROPS} value={backdrop} onchange={id => (size.pad.backdrop = id === 'pick' ? pick : id)} />
        {#if backdrop === 'pick'}<ColourKey label="" value={pick} onchange={hex => (size.pad.backdrop = hex)} />{/if}
      </Row>
      <Row label="Fills">
        <Slider label="How much of the frame the picture fills" min={50} max={100} value={Math.round(size.pad.fill)} unit="%" onchange={v => (size.pad.fill = v)}
          stops={[{ value: 50, label: '50%' }, { value: 70, label: '70' }, { value: 85, label: '85' }, { value: 100, label: '100%' }]} />
      </Row>
    {/if}
    <Row label={size.fit === 'crop' ? 'Keep' : 'Position'}>
      <AnchorPad label={size.fit === 'crop' ? 'Which part to keep' : 'Where the picture sits'} value={size.anchor} onchange={a => (size.anchor = a)} />
    </Row>
  </div>
{/if}

<style>
  .colon { font: 600 var(--fs-body)/1 var(--font-ui); color: var(--ink-2); }
  .quick { display: flex; flex-wrap: wrap; gap: var(--sp-3); margin-left: calc(84px + var(--sp-5)); }
  .quick :global(.key) { padding: 0 var(--sp-4); letter-spacing: 0; }
</style>
