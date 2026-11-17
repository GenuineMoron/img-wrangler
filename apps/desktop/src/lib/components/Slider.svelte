<!-- SPDX-License-Identifier: AGPL-3.0-or-later
  A slider in a recessed slot. Drag, click, arrow keys (Shift for bigger steps), Home and End.
  With `stops`, the printed labels under it are presses too. -->
<script lang="ts">
  let { label, value, min, max, step = 1, onchange, unit = '', stops, valueText }: {
    label: string;
    value: number;
    min: number;
    max: number;
    step?: number;
    onchange: (v: number) => void;
    unit?: string;
    /** Printed stops under the slot; pressing one jumps there. */
    stops?: { value: number; label: string }[];
    valueText?: (v: number) => string;
  } = $props();

  let el: HTMLDivElement;
  const frac = $derived((value - min) / (max - min || 1));
  const snap = (v: number) => Math.min(max, Math.max(min, Math.round(v / step) * step));

  function fromPointer(e: PointerEvent) {
    const r = el.getBoundingClientRect();
    const inset = 10;
    const x = Math.min(1, Math.max(0, (e.clientX - r.left - inset) / (r.width - 2 * inset)));
    const v = snap(min + x * (max - min));
    if (v !== value) onchange(v);
  }

  function pointerdown(e: PointerEvent) {
    e.preventDefault();
    el.focus({ preventScroll: true });
    el.setPointerCapture(e.pointerId);
    fromPointer(e);
  }

  function keydown(e: KeyboardEvent) {
    const big = e.shiftKey ? 10 : 1;
    const d = { ArrowRight: step * big, ArrowUp: step * big, ArrowLeft: -step * big, ArrowDown: -step * big, PageUp: step * 10, PageDown: -step * 10 }[e.key];
    if (d != null) { e.preventDefault(); onchange(snap(value + d)); }
    else if (e.key === 'Home') { e.preventDefault(); onchange(min); }
    else if (e.key === 'End') { e.preventDefault(); onchange(max); }
  }
</script>

<div class="slider-wrap">
  <div
    bind:this={el}
    class="slider"
    role="slider"
    tabindex="0"
    aria-label={label}
    aria-valuemin={min}
    aria-valuemax={max}
    aria-valuenow={value}
    aria-valuetext={valueText ? valueText(value) : `${value}${unit}`}
    style:--v={frac}
    onpointerdown={pointerdown}
    onpointermove={e => el.hasPointerCapture(e.pointerId) && fromPointer(e)}
    onkeydown={keydown}
  >
    <span class="slider__slot"></span>
    <span class="slider__fill"></span>
    {#each stops ?? [0, 0.25, 0.5, 0.75, 1].map(a => ({ value: min + a * (max - min), label: '' })) as s (s.value)}
      <span class="slider__tick" style:--at={(s.value - min) / (max - min)}></span>
    {/each}
    <span class="slider__cap"></span>
  </div>
  {#if stops}
    <div class="slider__stops">
      {#each stops as s (s.value)}
        <button type="button" class="slider__stop" style:--at={(s.value - min) / (max - min)} aria-current={s.value === value} tabindex="-1" onclick={() => onchange(s.value)}>{s.label}</button>
      {/each}
    </div>
  {/if}
</div>
