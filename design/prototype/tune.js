// SPDX-License-Identifier: AGPL-3.0-or-later
// Tune drawer: live controls for every design token worth adjusting, with copy-out.
// Plain CSS variables are set on :root; texture controls rebuild the grain tiles on a canvas.

const TUNE = [
  { group: 'Material', items: [
    { id: 'grain', label: 'Grain amount', min: 0, max: 30, step: 0.5, value: 9, tex: true },
    { id: 'grainSize', label: 'Grain size', min: 0.5, max: 4, step: 0.1, value: 1, tex: true },
    { id: '--tex-fine-size', label: 'Grain tile scale', min: 128, max: 512, step: 16, value: 256, unit: 'px' },
    { id: 'scratch', label: 'Scratches', min: 0, max: 4, step: 0.1, value: 1, tex: true },
    { id: '--tex-scratch-size', label: 'Scratch scale', min: 256, max: 2048, step: 32, value: 1024, unit: 'px' },
    { id: 'dust', label: 'Dust and wear', min: 0, max: 4, step: 0.1, value: 1, tex: true },
    { id: '--tex-dust-size', label: 'Dust scale', min: 256, max: 2048, step: 32, value: 1024, unit: 'px' },
    { id: '--light-amt', label: 'Light from above', min: 0, max: 0.7, step: 0.01, value: 0.32 },
    { id: 'caseL', label: 'Case lightness', min: 70, max: 96, step: 0.5, value: 89.5, color: true },
    { id: 'caseWarm', label: 'Case warmth', min: -12, max: 12, step: 0.5, value: -1, color: true },
  ] },
  { group: 'Keys', items: [
    { id: '--r-key', label: 'Key corner radius', min: 2, max: 15, step: 0.5, value: 7, unit: 'px' },
    { id: '--key-depth', label: 'Key depth', min: 0, max: 4, step: 0.25, value: 1.5, unit: 'px' },
    { id: '--key-soft', label: 'Key shadow softness', min: 0, max: 20, step: 1, value: 8, unit: 'px' },
    { id: '--key-h', label: 'Key height', min: 26, max: 38, step: 1, value: 30, unit: 'px' },
    { id: '--fs-key', label: 'Key label size', min: 9, max: 13, step: 0.5, value: 10.5, unit: 'px' },
  ] },
  { group: 'Screens', items: [
    { id: '--px-grid', label: 'Pixel grid', min: 0, max: 0.5, step: 0.01, value: 0.14 },
    { id: '--px-pitch', label: 'Pixel pitch', min: 2, max: 6, step: 1, value: 3, unit: 'px' },
    { id: '--glass', label: 'Glass reflection', min: 0, max: 0.2, step: 0.005, value: 0.055 },
    { id: '--bloom', label: 'Glow radius', min: 0, max: 10, step: 0.5, value: 3, unit: 'px' },
    { id: '--bloom-amt', label: 'Glow strength', min: 0, max: 100, step: 1, value: 55, unit: '%' },
    { id: '--fs-value', label: 'Big numerals', min: 22, max: 44, step: 1, value: 32, unit: 'px' },
    { id: '--value-weight', label: 'Numeral weight', min: 200, max: 500, step: 10, value: 200 },
    { id: '--fs-small', label: 'Screen text', min: 10, max: 13, step: 0.5, value: 11, unit: 'px' },
  ] },
  { group: 'LED meter', items: [
    { id: '--fs-led', label: 'Readout size', min: 28, max: 56, step: 1, value: 40, unit: 'px' },
    { id: '--led-glow', label: 'Readout glow', min: 0, max: 24, step: 1, value: 10, unit: 'px' },
    { id: '--led-rond', label: 'Dot roundness', min: 0, max: 100, step: 1, value: 0 },
  ] },
  { group: 'Type and layout', items: [
    { id: '--fs-legend', label: 'Printed labels', min: 9, max: 12, step: 0.5, value: 10, unit: 'px' },
    { id: '--tracking-legend', label: 'Label tracking', min: 0, max: 0.2, step: 0.005, value: 0.09, unit: 'em' },
    { id: '--gutter', label: 'Gutter', min: 8, max: 24, step: 1, value: 14, unit: 'px' },
    { id: '--r-panel', label: 'Panel radius', min: 4, max: 24, step: 1, value: 14, unit: 'px' },
    { id: '--r-screen', label: 'Screen radius', min: 0, max: 16, step: 1, value: 8, unit: 'px' },
  ] },
  { group: 'Colour', items: [
    { id: '--accent-size', label: 'Size', type: 'color', value: '#3d5cff' },
    { id: '--accent-format', label: 'Format', type: 'color', value: '#18c06a' },
    { id: '--accent-quality', label: 'Quality', type: 'color', value: '#ffc61a' },
    { id: '--accent-name', label: 'Name', type: 'color', value: '#ff4f9a' },
    { id: '--accent-trim', label: 'Trim', type: 'color', value: '#16b5a2' },
    { id: '--accent-canvas', label: 'Canvas', type: 'color', value: '#7c4dff' },
    { id: '--output', label: 'Output and Run', type: 'color', value: '#ff5a1f' },
    { id: '--status-alarm', label: 'Alarm', type: 'color', value: '#ff4438' },
  ] },
];

const STORE = 'wrangler-tune-v1';
const tuneValues = Object.fromEntries(TUNE.flatMap(g => g.items.map(i => [i.id, i.value])));
const defaults = { ...tuneValues };
try { Object.assign(tuneValues, JSON.parse(localStorage.getItem(STORE) || '{}')); } catch { /* first run */ }

const root = document.documentElement;
const unitOf = id => TUNE.flatMap(g => g.items).find(i => i.id === id)?.unit ?? '';

function applyVar(id) {
  if (!id.startsWith('--')) return;
  root.style.setProperty(id, `${tuneValues[id]}${unitOf(id)}`);
}

function applyCase() {
  // case colour from lightness and warmth (warmth shifts towards cream or towards blue-grey)
  const L = tuneValues.caseL, w = tuneValues.caseWarm;
  root.style.setProperty('--surface-case', `oklch(${L}% ${Math.abs(w) * 0.0022 + 0.002} ${w >= 0 ? 85 : 255})`);
}

// ---------- texture tiles, rebuilt on a canvas ----------
const scans = {};
function loadScan(name) {
  return scans[name] ??= new Promise(res => { const im = new Image(); im.onload = () => res(im); im.onerror = () => res(null); im.src = `../system/tex/${name}`; });
}
function gaussian() { let u = 0, v = 0; while (!u) u = Math.random(); while (!v) v = Math.random(); return Math.sqrt(-2 * Math.log(u)) * Math.cos(2 * Math.PI * v); }

function grainTile(amount, size) {
  const N = 256, n = Math.max(16, Math.round(N / size));
  const small = document.createElement('canvas'); small.width = small.height = n;
  const sctx = small.getContext('2d'), img = sctx.createImageData(n, n);
  for (let i = 0; i < n * n; i++) { const g = Math.max(0, Math.min(255, 128 + gaussian() * amount)); img.data.set([g, g, g, 255], i * 4); }
  sctx.putImageData(img, 0, 0);
  const c = document.createElement('canvas'); c.width = c.height = N;
  const ctx = c.getContext('2d'); ctx.imageSmoothingEnabled = size > 1; ctx.drawImage(small, 0, 0, N, N);
  return c.toDataURL('image/png');
}

async function scanTile(name, k) {
  const im = await loadScan(name); if (!im) return null;
  const c = document.createElement('canvas'); c.width = im.width; c.height = im.height;
  const ctx = c.getContext('2d'); ctx.drawImage(im, 0, 0);
  try {
    const d = ctx.getImageData(0, 0, c.width, c.height);
    for (let i = 0; i < d.data.length; i += 4) { const v = Math.max(0, Math.min(255, 128 + (d.data[i] - 128) * k)); d.data[i] = d.data[i + 1] = d.data[i + 2] = v; }
    ctx.putImageData(d, 0, 0);
    return c.toDataURL('image/jpeg', 0.86);
  } catch { return null; } // file:// pages cannot read pixels; serve over http to tune scans
}

let texTimer;
function rebuildTextures() {
  clearTimeout(texTimer);
  texTimer = setTimeout(async () => {
    root.style.setProperty('--tex-fine', `url(${grainTile(tuneValues.grain, tuneValues.grainSize)})`);
    const sc = await scanTile('scratch-l.jpg', tuneValues.scratch); if (sc) root.style.setProperty('--tex-scratch', `url(${sc})`);
    const du = await scanTile('dust-l.jpg', tuneValues.dust); if (du) root.style.setProperty('--tex-dust', `url(${du})`);
  }, 120);
}

// ---------- drawer ----------
function exportCss() {
  const lines = TUNE.flatMap(g => g.items.filter(i => i.id.startsWith('--')).map(i => `  ${i.id}: ${tuneValues[i.id]}${i.unit ?? ''};`));
  const texture = `  /* material: grain ${tuneValues.grain}, grain size ${tuneValues.grainSize}, scratches ×${tuneValues.scratch}, dust ×${tuneValues.dust}, case L ${tuneValues.caseL}%, warmth ${tuneValues.caseWarm} */`;
  return `:root {\n${texture}\n${lines.join('\n')}\n}`;
}

function renderTune() {
  const el = document.getElementById('tune');
  el.innerHTML = `<p class="hint">Changes apply live and are remembered in this browser. When it looks right, press Copy and paste the block back.</p>
    ${TUNE.map(g => `<h3>${g.group}</h3>${g.items.map(i => i.type === 'color'
      ? `<div class="ctl"><label for="t-${i.id}">${i.label}</label><input type="color" id="t-${i.id}" data-tune="${i.id}" value="${tuneValues[i.id]}"></div>`
      : `<div class="ctl"><label for="t-${i.id}">${i.label}</label><output id="o-${i.id}">${tuneValues[i.id]}${i.unit ?? ''}</output>
         <input type="range" id="t-${i.id}" data-tune="${i.id}" min="${i.min}" max="${i.max}" step="${i.step}" value="${tuneValues[i.id]}"></div>`).join('')}`).join('')}
    <div class="actions"><button type="button" class="primary" data-tune-act="copy">Copy</button><button type="button" data-tune-act="json">JSON</button><button type="button" data-tune-act="reset">Reset</button></div>
    <textarea id="tuneOut" readonly aria-label="Exported values">${exportCss()}</textarea>`;
}

document.getElementById('tune').addEventListener('input', e => {
  const id = e.target.dataset.tune; if (!id) return;
  tuneValues[id] = e.target.type === 'color' ? e.target.value : Number(e.target.value);
  const item = TUNE.flatMap(g => g.items).find(i => i.id === id);
  const out = document.getElementById(`o-${id}`); if (out) out.textContent = `${tuneValues[id]}${item.unit ?? ''}`;
  if (item.tex) rebuildTextures(); else if (item.color) applyCase(); else applyVar(id);
  localStorage.setItem(STORE, JSON.stringify(tuneValues));
  document.getElementById('tuneOut').value = exportCss();
});

document.getElementById('tune').addEventListener('click', async e => {
  const a = e.target.dataset.tuneAct; if (!a) return;
  const out = document.getElementById('tuneOut');
  if (a === 'copy' || a === 'json') {
    out.value = a === 'json' ? JSON.stringify(tuneValues, null, 2) : exportCss();
    out.select();
    try { await navigator.clipboard.writeText(out.value); e.target.textContent = 'Copied'; } catch { document.execCommand('copy'); e.target.textContent = 'Copied'; }
    setTimeout(() => e.target.textContent = a === 'json' ? 'JSON' : 'Copy', 1200);
  }
  if (a === 'reset') { Object.assign(tuneValues, defaults); localStorage.removeItem(STORE); applyAll(); renderTune();
if (location.hash === '#tune') toggle.click(); }
});

const toggle = document.getElementById('tuneToggle');
toggle.addEventListener('click', () => {
  const el = document.getElementById('tune'); const open = el.dataset.open !== 'true';
  el.dataset.open = open; toggle.setAttribute('aria-expanded', open); toggle.textContent = open ? 'Close' : 'Tune';
});

function applyAll() {
  TUNE.flatMap(g => g.items).forEach(i => { if (i.id.startsWith('--')) applyVar(i.id); });
  applyCase(); rebuildTextures();
}
applyAll(); renderTune();
if (location.hash === '#tune') toggle.click();
