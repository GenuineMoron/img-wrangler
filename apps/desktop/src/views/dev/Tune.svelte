<!-- SPDX-License-Identifier: AGPL-3.0-or-later
  Dev builds only (open with ?tune): live controls for the design tokens worth adjusting, with
  copy-out. Starts from the values in design/system/tokens.css. Texture controls rebuild the
  grain and scan tiles on a canvas. -->
<script lang="ts">
  import scratchUrl from '@design/tex/scratch-l.jpg?url';
  import dustUrl from '@design/tex/dust-l.jpg?url';

  type Item = { id: string; label: string; min?: number; max?: number; step?: number; unit?: string; color?: boolean; tex?: number; value?: number };
  const GROUPS: { group: string; items: Item[] }[] = [
    { group: 'Material', items: [
      { id: 'grain', label: 'Grain amount', min: 0, max: 30, step: 0.5, value: 16.5, tex: 1 },
      { id: 'grainSize', label: 'Grain size', min: 0.5, max: 4, step: 0.1, value: 0.8, tex: 1 },
      { id: '--tex-fine-size', label: 'Grain tile scale', min: 128, max: 512, step: 16, unit: 'px' },
      { id: 'scratch', label: 'Scratches', min: 0, max: 4, step: 0.1, value: 0.8, tex: 2 },
      { id: '--tex-scratch-size', label: 'Scratch scale', min: 256, max: 2048, step: 32, unit: 'px' },
      { id: 'dust', label: 'Dust and wear', min: 0, max: 4, step: 0.1, value: 1, tex: 3 },
      { id: '--tex-dust-size', label: 'Dust scale', min: 256, max: 2048, step: 32, unit: 'px' },
      { id: '--light-amt', label: 'Light from above', min: 0, max: 0.7, step: 0.01 },
      { id: 'caseL', label: 'Case lightness', min: 70, max: 96, step: 0.5, value: 93.5 },
      { id: 'caseWarm', label: 'Case warmth', min: -12, max: 12, step: 0.5, value: 1.5 },
    ] },
    { group: 'Keys', items: [
      { id: '--r-key', label: 'Corner radius', min: 2, max: 15, step: 0.5, unit: 'px' },
      { id: '--key-depth', label: 'Depth', min: 0, max: 4, step: 0.25, unit: 'px' },
      { id: '--key-soft', label: 'Shadow softness', min: 0, max: 20, step: 1, unit: 'px' },
      { id: '--key-h', label: 'Height', min: 24, max: 38, step: 1, unit: 'px' },
      { id: '--fs-key', label: 'Label size', min: 9, max: 13, step: 0.5, unit: 'px' },
    ] },
    { group: 'Screens', items: [
      { id: '--px-grid', label: 'Pixel grid', min: 0, max: 0.5, step: 0.01 },
      { id: '--px-pitch', label: 'Pixel pitch', min: 2, max: 6, step: 1, unit: 'px' },
      { id: '--glass', label: 'Glass reflection', min: 0, max: 0.2, step: 0.005 },
      { id: '--bloom', label: 'Glow radius', min: 0, max: 10, step: 0.5, unit: 'px' },
      { id: '--bloom-amt', label: 'Glow strength', min: 0, max: 100, step: 1, unit: '%' },
      { id: '--fs-value', label: 'Big numerals', min: 22, max: 44, step: 1, unit: 'px' },
      { id: '--value-weight', label: 'Numeral weight', min: 200, max: 500, step: 10 },
      { id: '--fs-small', label: 'Screen text', min: 10, max: 13, step: 0.5, unit: 'px' },
    ] },
    { group: 'LED meter', items: [
      { id: '--fs-led', label: 'Readout size', min: 28, max: 56, step: 1, unit: 'px' },
      { id: '--led-glow', label: 'Readout glow', min: 0, max: 24, step: 1, unit: 'px' },
      { id: '--led-rond', label: 'Dot roundness', min: 0, max: 100, step: 1 },
    ] },
    { group: 'Type and layout', items: [
      { id: '--fs-legend', label: 'Printed labels', min: 9, max: 12, step: 0.5, unit: 'px' },
      { id: '--tracking-legend', label: 'Label tracking', min: 0, max: 0.2, step: 0.005, unit: 'em' },
      { id: '--gutter', label: 'Gutter', min: 8, max: 24, step: 1, unit: 'px' },
      { id: '--r-panel', label: 'Panel radius', min: 4, max: 24, step: 1, unit: 'px' },
      { id: '--r-screen', label: 'Screen radius', min: 0, max: 16, step: 1, unit: 'px' },
    ] },
    { group: 'Colour', items: ['size', 'format', 'quality', 'name', 'trim', 'canvas'].map(a => ({ id: `--accent-${a}`, label: a[0].toUpperCase() + a.slice(1), color: true }))
      .concat([{ id: '--output', label: 'Output and Run', color: true }, { id: '--status-alarm', label: 'Alarm', color: true }]) },
  ];
  const ITEMS = GROUPS.flatMap(g => g.items);
  const root = document.documentElement;
  const css = getComputedStyle(root);
  const start = Object.fromEntries(ITEMS.map(i => [i.id, i.id.startsWith('--') ? (i.color ? css.getPropertyValue(i.id).trim() : parseFloat(css.getPropertyValue(i.id))) : i.value!]));
  const STORE = 'wrangler-tune-v2';
  let values = $state<Record<string, number | string>>({ ...start, ...JSON.parse(localStorage.getItem(STORE) ?? '{}') });
  let open = $state(true);
  let copied = $state(false);

  function gaussian() { let u = 0, v = 0; while (!u) u = Math.random(); while (!v) v = Math.random(); return Math.sqrt(-2 * Math.log(u)) * Math.cos(2 * Math.PI * v); }
  function grainTile(amount: number, size: number) {
    const N = 256, n = Math.max(16, Math.round(N / size));
    const small = Object.assign(document.createElement('canvas'), { width: n, height: n });
    const sx = small.getContext('2d')!, img = sx.createImageData(n, n);
    for (let i = 0; i < n * n; i++) { const g = Math.max(0, Math.min(255, 128 + gaussian() * amount)); img.data.set([g, g, g, 255], i * 4); }
    sx.putImageData(img, 0, 0);
    const c = Object.assign(document.createElement('canvas'), { width: N, height: N });
    const cx = c.getContext('2d')!; cx.imageSmoothingEnabled = size > 1; cx.drawImage(small, 0, 0, N, N);
    return c.toDataURL('image/png');
  }
  async function scanTile(url: string, k: number) {
    const im = await new Promise<HTMLImageElement>(ok => { const i = new Image(); i.onload = () => ok(i); i.src = url; });
    const c = Object.assign(document.createElement('canvas'), { width: im.width, height: im.height });
    const cx = c.getContext('2d')!; cx.drawImage(im, 0, 0);
    const d = cx.getImageData(0, 0, c.width, c.height);
    for (let i = 0; i < d.data.length; i += 4) { const v = Math.max(0, Math.min(255, 128 + (d.data[i] - 128) * k)); d.data[i] = d.data[i + 1] = d.data[i + 2] = v; }
    cx.putImageData(d, 0, 0);
    return c.toDataURL('image/jpeg', 0.86);
  }

  const changed = (id: string) => values[id] !== start[id];
  async function apply(item: Item) {
    const v = values[item.id];
    if (item.id.startsWith('--')) root.style.setProperty(item.id, `${v}${item.unit ?? ''}`);
    else if (item.id === 'caseL' || item.id === 'caseWarm') {
      const w = Number(values.caseWarm);
      root.style.setProperty('--surface-case', `oklch(${values.caseL}% ${(Math.abs(w) * 0.0022 + 0.002).toFixed(4)} ${w >= 0 ? 85 : 255})`);
    } else if (item.tex === 1) root.style.setProperty('--tex-fine', `url(${grainTile(Number(values.grain), Number(values.grainSize))})`);
    else if (item.tex === 2) root.style.setProperty('--tex-scratch', `url(${await scanTile(scratchUrl, Number(values.scratch))})`);
    else if (item.tex === 3) root.style.setProperty('--tex-dust', `url(${await scanTile(dustUrl, Number(values.dust))})`);
    localStorage.setItem(STORE, JSON.stringify(values));
  }
  for (const i of ITEMS) if (changed(i.id)) apply(i);

  const exported = $derived(`:root {\n  /* material: grain ${values.grain}, grain size ${values.grainSize}, scratches ×${values.scratch}, dust ×${values.dust}, case L ${values.caseL}%, warmth ${values.caseWarm} */\n${ITEMS.filter(i => i.id.startsWith('--')).map(i => `  ${i.id}: ${values[i.id]}${i.unit ?? ''};`).join('\n')}\n}`);

  async function copy() { await navigator.clipboard.writeText(exported); copied = true; setTimeout(() => (copied = false), 1200); }
  function reset() { localStorage.removeItem(STORE); location.reload(); }
</script>

<button type="button" class="toggle" aria-expanded={open} onclick={() => (open = !open)}>{open ? 'Close' : 'Tune'}</button>
{#if open}
  <aside class="tune" aria-label="Tune design values">
    <p class="hint">Live, and remembered in this browser. When it looks right, copy the block into design/system/tokens.css.</p>
    {#each GROUPS as g (g.group)}
      <h3>{g.group}</h3>
      {#each g.items as i (i.id)}
        <label class="ctl" class:changed={changed(i.id)}>
          <span>{i.label}</span>
          {#if i.color}
            <input type="color" value={values[i.id]} oninput={e => { values[i.id] = e.currentTarget.value; apply(i); }} />
          {:else}
            <output>{values[i.id]}{i.unit ?? ''}</output>
            <input type="range" min={i.min} max={i.max} step={i.step} value={values[i.id]} oninput={e => { values[i.id] = Number(e.currentTarget.value); apply(i); }} />
          {/if}
        </label>
      {/each}
    {/each}
    <div class="actions"><button type="button" class="primary" onclick={copy}>{copied ? 'Copied' : 'Copy CSS'}</button><button type="button" onclick={reset}>Reset</button></div>
    <textarea readonly aria-label="Exported values">{exported}</textarea>
  </aside>
{/if}

<style>
  .toggle { position: fixed; right: 18px; bottom: 18px; z-index: 60; border: 0; border-radius: 6px; padding: 8px 12px; cursor: pointer; font: 600 11px/1 var(--font-ui); letter-spacing: .05em; text-transform: uppercase; background: #151618; color: #eee; }
  .tune { position: fixed; top: 0; right: 0; bottom: 0; width: 330px; z-index: 50; overflow-y: auto; padding: 18px 18px 70px; background: #151618; color: #e6e6e3; box-shadow: -20px 0 40px rgba(0, 0, 0, .45); font: 13px/1.4 var(--font-ui); -webkit-user-select: text; user-select: text; }
  h3 { font: 600 11px/1 var(--font-ui); letter-spacing: .09em; text-transform: uppercase; color: #9b9b97; margin: 18px 0 8px; }
  .ctl { display: grid; grid-template-columns: 1fr 64px; gap: 2px 8px; align-items: center; margin-bottom: 8px; color: #d6d6d2; }
  .ctl.changed span { color: var(--output); }
  output { text-align: right; font: 11px var(--font-data); color: #9b9b97; }
  input[type='range'] { grid-column: 1 / -1; width: 100%; accent-color: var(--output); }
  input[type='color'] { width: 64px; height: 24px; border: 0; background: none; padding: 0; }
  .actions { display: grid; grid-template-columns: 1fr 1fr; gap: 6px; margin-top: 18px; }
  .actions button { border: 0; border-radius: 6px; padding: 8px 6px; cursor: pointer; font: 600 11px/1 var(--font-ui); letter-spacing: .05em; text-transform: uppercase; background: #2b2c2f; color: #eee; }
  .actions .primary { background: var(--output); color: #fff; }
  textarea { width: 100%; height: 160px; margin-top: 10px; background: #0b0b0c; color: #cfcfcb; border: 1px solid #2b2c2f; border-radius: 6px; padding: 8px; font: 11px/1.4 var(--font-data); }
  .hint { color: #8a8a86; font-size: 12px; }
</style>
