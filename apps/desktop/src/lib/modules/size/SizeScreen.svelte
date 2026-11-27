<!-- SPDX-License-Identifier: AGPL-3.0-or-later
  The Size screen is the control: drag the frame's corner to resize, type the number, pick a stop
  on the ruler, and choose how the picture meets the frame. The diagram draws the selected image
  to scale: its original outline, the frame it goes into, and where the picture lands. With
  "Enlarge smaller images" on, the diagram zooms out so the frame can be dragged past the
  original, up to twice its size. Any size can be typed. -->
<script lang="ts">
  import ScreenChoice from '$lib/components/ScreenChoice.svelte';
  import { layout, parseShape, specOf } from '$lib/geometry';
  import { batch } from '$lib/state/batch.svelte';
  import { recipe } from '$lib/state/recipe.svelte';
  import type { Fit } from '$lib/types';
  import { FITS, STOPS } from './index';

  const BOX = 84;
  const size = $derived(recipe.value!.size);
  const src = $derived(batch.current?.planned && batch.current.probe?.dims ? batch.current.probe.dims : { w: 6000, h: 4000 });
  const spec = $derived(specOf(size));
  const shape = $derived(spec.shape);
  const l = $derived(layout(src, spec));
  const maxSrc = $derived(Math.max(src.w, src.h));
  const outLong = $derived(Math.max(l.canvas.w, l.canvas.h));
  // How far the box reaches, in originals: just the original, or twice it when enlarging.
  const reach = $derived(size.upscale ? 2 : 1);
  // Drawn to an area scale: the frame's area shows how much smaller or bigger it is (a third of
  // the pixels looks like a third). Linear scale made common sizes too small to read.
  const scale = $derived((BOX * Math.min(1, Math.sqrt(outLong / maxSrc / reach))) / outLong);
  const srcScale = $derived((BOX * Math.sqrt(1 / reach)) / maxSrc);
  /** This image is smaller than the size asked for, and isn't being enlarged. */
  const short = $derived(!size.upscale && shape.kind !== 'exact' && size.long_side != null && maxSrc < size.long_side);

  // Frame (the output canvas), anchored bottom-left.
  const fw = $derived(Math.max(6, l.canvas.w * scale));
  const fh = $derived(Math.max(6, l.canvas.h * scale));
  const fy = $derived(BOX - fh);
  // Where the picture lands inside the frame.
  const pic = $derived.by(() => {
    if (l.crop) {
      const k = (l.canvas.w / l.crop.w) * scale;
      return { x: -l.crop.x * k, y: fy - l.crop.y * k, w: src.w * k, h: src.h * k };
    }
    return { x: l.offset[0] * scale, y: fy + l.offset[1] * scale, w: l.scaled.w * scale, h: l.scaled.h * scale };
  });
  const padded = $derived(size.fit === 'pad' && shape.kind !== 'original' && (l.scaled.w < l.canvas.w || l.scaled.h < l.canvas.h));
  const backdrop = $derived(size.pad.backdrop);
  const swatch = $derived(backdrop === 'auto' ? 'var(--accent-canvas)' : backdrop === 'transparent' ? 'url(#clear)' : backdrop);

  let typing = $state(false);
  let input: HTMLInputElement | undefined = $state();

  function setLong(px: number | null) {
    size.long_side = px;
  }

  /** Long side from a pointer over the diagram, snapping to the ruler. */
  function fromPointer(e: PointerEvent, box: DOMRect) {
    const frac = Math.max((e.clientX - box.left) / box.width, (box.bottom - e.clientY) / box.height);
    // Inverse of the area scale above.
    const px = Math.max(100, Math.round((Math.min(1, Math.max(0.1, frac)) ** 2 * reach * maxSrc) / 10) * 10);
    const snap = STOPS.find(s => s.px && Math.abs(s.px - px) < maxSrc * 0.035);
    // The original's own size is "full": no resizing at all.
    setLong(snap?.px ?? (Math.abs(px - maxSrc) < maxSrc * 0.035 ? null : px));
  }

  let svg: SVGSVGElement;

  function drag(e: PointerEvent) {
    if (shape.kind === 'exact') return;
    const el = e.currentTarget as Element;
    const box = svg.getBoundingClientRect();
    e.preventDefault();
    el.setPointerCapture(e.pointerId);
    fromPointer(e, box);
    const move = (ev: Event) => fromPointer(ev as PointerEvent, box);
    el.addEventListener('pointermove', move);
    el.addEventListener('pointerup', () => el.removeEventListener('pointermove', move), { once: true });
  }

  function handleKey(e: KeyboardEvent) {
    if (e.key === 'Enter') { e.preventDefault(); startTyping(); return; }
    const dir = { ArrowUp: 1, ArrowRight: 1, ArrowDown: -1, ArrowLeft: -1 }[e.key];
    if (!dir) return;
    e.preventDefault();
    const cur = size.long_side ?? maxSrc;
    const list = [...new Set([...STOPS.map(s => s.px ?? maxSrc), maxSrc, ...(size.upscale ? [maxSrc * 2] : [])])].filter(px => px <= maxSrc * reach).sort((a, b) => a - b);
    const next = dir > 0 ? list.find(px => px > cur) : [...list].reverse().find(px => px < cur);
    if (next) setLong(next === maxSrc ? null : next);
  }

  function startTyping() {
    if (shape.kind === 'exact') return;
    typing = true;
    queueMicrotask(() => { input?.focus(); input?.select(); });
  }

  function finish(apply: boolean) {
    if (!typing) return;
    const v = Number(input?.value);
    if (apply && input?.value === '') setLong(null);
    else if (apply && v >= 16 && v <= 30000) setLong(Math.round(v));
    typing = false;
  }

  const bigText = $derived(shape.kind === 'exact' ? `${shape.w}×${shape.h}` : size.long_side ? String(size.long_side) : 'full');
  /** What the big number is, in a few words. */
  const unit = $derived(shape.kind === 'exact' ? 'exact pixels' : size.long_side ? 'px long side' : 'size, not resized');
  const approx = $derived(size.trim ? '≈ ' : '');
</script>

<div class="w-size">
  <div class="dia">
  <svg bind:this={svg} class="diagram" viewBox="0 0 {BOX} {BOX}" width={BOX} height={BOX} onpointerdown={drag} class:fixed={shape.kind === 'exact'} aria-hidden="true">
    <defs>
      <pattern id="hatch" width="4" height="4" patternUnits="userSpaceOnUse" patternTransform="rotate(45)"><path d="M0 0v4" stroke="rgba(255,255,255,.07)" /></pattern>
      <pattern id="clear" width="6" height="6" patternUnits="userSpaceOnUse"><rect width="6" height="6" fill="#222" /><path d="M0 0h3v3H0zM3 3h3v3H3z" fill="#3a3a3a" /></pattern>
      <clipPath id="frame"><rect x="0" y={fy} width={fw} height={fh} /></clipPath>
    </defs>
    <!-- the original, to scale -->
    <rect class="src" x=".5" y={BOX - src.h * srcScale + 0.5} width={src.w * srcScale - 1} height={src.h * srcScale - 1} />
    <!-- backdrop where padding shows -->
    {#if padded}<rect x="0" y={fy} width={fw} height={fh} fill={swatch} opacity={backdrop === 'auto' ? 0.28 : 0.9} />{/if}
    <!-- the picture -->
    {#if l.crop}<rect class="cut" x={pic.x} y={pic.y} width={pic.w} height={pic.h} />{/if}
    <rect class="pic" x={pic.x} y={pic.y} width={pic.w} height={pic.h} clip-path="url(#frame)" />
    {#if size.trim}
      <path class="trim" d="M{pic.x + 1} {pic.y + 6}V{pic.y + 1}H{pic.x + 6}M{pic.x + pic.w - 6} {pic.y + pic.h - 1}H{pic.x + pic.w - 1}V{pic.y + pic.h - 6}" clip-path="url(#frame)" />
    {/if}
    <!-- the frame -->
    <rect class="frame glow" x=".75" y={fy + 0.75} width={fw - 1.5} height={fh - 1.5} />
  </svg>
  {#if shape.kind !== 'exact'}
    <span
      class="handle"
      role="slider"
      tabindex="0"
      aria-label="Output size. Drag the corner or use the arrow keys, Enter to type"
      aria-valuemin="16"
      aria-valuemax={maxSrc * reach}
      aria-valuenow={outLong}
      aria-valuetext="{outLong} pixels on the long side"
      style:left="{(fw / BOX) * 100}%"
      style:top="{(fy / BOX) * 100}%"
      onkeydown={handleKey}
      onpointerdown={e => { (e.currentTarget as HTMLElement).focus(); drag(e); }}
    ></span>
  {/if}
  </div>

  <div class="vals">
    {#if typing}
      <input bind:this={input} class="value-input" type="number" min="16" max="30000" step="10" value={size.long_side ?? ''} placeholder="full"
        aria-label="Long side in pixels, empty for full size. Enter to apply"
        onkeydown={e => { if (e.key === 'Enter') finish(true); if (e.key === 'Escape') finish(false); }} onblur={() => finish(true)} />
    {:else}
      <button type="button" class="value-btn glow" class:small={shape.kind === 'exact'} onclick={startTyping} disabled={shape.kind === 'exact'}
        aria-label={shape.kind === 'exact' ? `Exact size ${bigText}` : `Long side ${bigText}. Press to type a size`}>{bigText}</button>
    {/if}
    <span class="scr-label unit">{unit}</span>
    <span class="sub" class:short title="This image comes out at {l.canvas.w} × {l.canvas.h} pixels" data-long>{short ? 'smaller: stays ' : '→ '}{approx}{l.canvas.w}×{l.canvas.h}</span>
    {#if shape.kind !== 'original'}
      <ScreenChoice label="How the picture meets the frame" items={[...FITS]} value={size.fit} onchange={(v: Fit) => (size.fit = v)} />
    {/if}
  </div>

  <ScreenChoice
    class="ruler"
    layout="spread"
    label="Size stops"
    items={STOPS.map(s => ({ id: s.px, label: s.label, aria: s.aria, disabled: shape.kind === 'exact' }))}
    value={shape.kind === 'exact' ? undefined : size.long_side}
    onchange={setLong}
  />
</div>

<style>
  .w-size { position: relative; display: grid; grid-template-columns: 84px minmax(0, 1fr); grid-template-rows: 1fr auto; column-gap: var(--sp-6); height: 100%; }
  .dia { position: relative; width: 84px; height: 84px; align-self: end; margin: 0 0 4px 2px; }
  .diagram { display: block; width: 100%; height: 100%; cursor: nwse-resize; touch-action: none; overflow: hidden; }
  .diagram.fixed { cursor: default; }
  .src { fill: url(#hatch); stroke: var(--screen-3); stroke-width: 1; stroke-dasharray: 2 2; }
  .pic { fill: color-mix(in srgb, var(--screen-1) 16%, transparent); }
  .cut { fill: none; stroke: var(--screen-3); stroke-width: 1; stroke-dasharray: 1.5 2; }
  .trim { fill: none; stroke: var(--accent-trim); stroke-width: 1.5; }
  .frame { fill: color-mix(in srgb, var(--accent-size) 14%, transparent); stroke: var(--accent-size-text); stroke-width: 1.5; color: var(--accent-size); }
  .handle { position: absolute; width: 12px; height: 12px; margin: -6px 0 0 -6px; border-radius: 50%; background: var(--surface-screen);
    box-shadow: 0 0 0 1.5px var(--accent-size-text), 0 0 6px var(--accent-size); cursor: nwse-resize; touch-action: none; }
  .handle:focus-visible { outline: var(--focus-on-screen); outline-offset: 2px; }
  .vals { display: grid; gap: 4px; align-content: center; justify-items: start; min-width: 0; color: var(--accent-size-text); }
  .value-btn { all: unset; cursor: text; font: var(--value-weight) var(--fs-value)/1 var(--font-value); border-radius: 2px; color: var(--accent-size-text); }
  .value-btn.small { font-size: 17px; cursor: default; }
  .value-btn:hover:not(:disabled) { box-shadow: 0 1px 0 currentColor; }
  .value-btn:focus-visible { outline: var(--focus-on-screen); outline-offset: 2px; }
  .value-input { all: unset; width: 100%; font: var(--value-weight) var(--fs-value)/1 var(--font-value); color: var(--accent-size-text); box-shadow: 0 1px 0 var(--accent-size-text); }
  .unit { color: var(--screen-3); margin-top: -1px; white-space: nowrap; }
  .sub { font-size: 12px; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; max-width: 100%; margin-top: 3px; }
  .sub.short { color: var(--accent-quality); }
  .vals :global(.choice) { --accent: var(--accent-size); --on-accent: #fff; margin-left: -5px; }
  @container channel (max-width: 210px) {
    .w-size { grid-template-columns: 62px minmax(0, 1fr); column-gap: var(--sp-7); }
    .dia { width: 62px; height: 62px; }
    .value-btn, .value-input { font-size: 23px; }
    .value-btn.small { font-size: 15px; }
  }
  .w-size :global(.ruler button) { padding: 1px 3px; }
  @container channel (max-width: 210px) {
    .vals :global(.choice) { gap: 0; margin-left: -3px; }
    .vals :global(.choice button) { padding: 1px 2px; font-size: 9.5px; }
    .w-size :global(.ruler) { gap: 1px; }
    .w-size :global(.ruler button) { padding: 1px 1px; font-size: 9.5px; }
  }
  .w-size :global(.ruler) { grid-column: 1 / -1; border-top: 1px solid var(--screen-line); padding-top: 3px; --accent: var(--accent-size); --on-accent: #fff; }
</style>
