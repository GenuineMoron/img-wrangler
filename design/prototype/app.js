// SPDX-License-Identifier: AGPL-3.0-or-later
// Wrangler prototype: one state object, derived values, and render functions per region.
// Controls carry data-bind="path" and data-value; one delegated handler writes state and re-renders.

const state = structuredClone(DEFAULT_STATE);
const $ = (sel, root = document) => root.querySelector(sel);
const esc = s => String(s).replace(/[&<>"]/g, c => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' }[c]));
const fmtById = id => FORMATS.find(f => f.id === id);
const extraById = id => EXTRAS.find(e => e.id === id);

// ---------- derived values ----------
const srcExt = f => f.src.split('.').pop().toLowerCase();
const stem = f => f.src.replace(/\.[^.]+$/, '');
const SAME_MAP = { jpg: 'jpg', jpeg: 'jpg', png: 'png', heic: 'jpg', tif: 'tiff', tiff: 'tiff', webp: 'webp' };

function outFormat(f) {
  if (state.format.id !== 'same') return fmtById(state.format.id);
  return fmtById(SAME_MAP[srcExt(f)] || 'jpg');
}

function geometryChanges() {
  const x = state.extras;
  return x.trim.on || x.canvas.on || x.rotate.on || state.size.mode === 'exact';
}

function outDims(f) {
  const px = state.size.preset === 'orig' ? null : Number(state.size.preset);
  const c = state.extras.canvas;
  if (c.on) {
    const long = px ?? Math.max(f.w, f.h);
    const [a, b] = c.shape === 'any' ? [f.w, f.h] : c.shape.split(':').map(Number);
    return a >= b ? { w: long, h: Math.round(long * b / a) } : { w: Math.round(long * a / b), h: long };
  }
  if (px == null) return { w: f.w, h: f.h };
  if (state.size.mode === 'exact') return { w: px, h: px };
  const s = Math.min(1, px / Math.max(f.w, f.h));
  return { w: Math.round(f.w * s), h: Math.round(f.h * s) };
}

/** Estimated output size in KB, plus the flag the batch would show for it. */
function estimate(f) {
  const fmt = outFormat(f);
  const d = outDims(f);
  const mp = (d.w * d.h) / (2048 * 2048);
  const q = state.quality;
  const kbAt = i => QUALITY_STOPS[i].kb * fmt.factor * mp * f.detail;
  let kb = q.lossless ? kbAt(3) * 2.2 : kbAt(q.stop);
  let flag = null;
  if (q.limitOn && !q.lossless && kb > q.limitKb) {
    if (kbAt(0) > q.limitKb) flag = 'over';
    else kb = q.limitKb * 0.94;
  }
  const sameShape = d.w === f.w && d.h === f.h;
  if ((f.kept || kb >= f.kb) && fmt.ext === (SAME_MAP[srcExt(f)] || srcExt(f)) && sameShape) { kb = f.kb; flag = 'kept'; }
  else if (f.kept && sameShape) { kb = f.kb; flag = 'kept'; }
  if (f.clash && state.name.mode !== 'keep' && !flag) flag = 'clash';
  return { kb, flag, dims: d, fmt };
}

function renderPattern(pattern, f, i) {
  const n = i + 1;
  return pattern.replace(/\{(\w+)(?::(0+))?\}/g, (m, tok, pad) => {
    if (tok === 'n') return pad ? String(n).padStart(pad.length, '0') : String(n);
    if (tok === 'name') return stem(f);
    if (tok === 'date') return '2026-11-17';
    if (tok === 'w') return outDims(f).w;
    if (tok === 'h') return outDims(f).h;
    return m;
  });
}

function outName(f, i) {
  const fmt = outFormat(f);
  let base = stem(f);
  if (state.name.mode === 'pattern') base = renderPattern(state.name.pattern, f, i);
  if (state.name.mode === 'replace') base = base.split(state.name.find ?? 'DSC_').join(state.name.with ?? 'spring-');
  if (f.clash && state.name.mode !== 'keep') base += '-2';
  return `${base}.${fmt.ext}`;
}

const kbText = kb => kb >= 1000 ? `${(kb / 1024).toFixed(kb >= 10240 ? 0 : 1)}M` : `${Math.round(kb)}K`;
const bytesText = b => b >= 1e9 ? `${(b / 1e9).toFixed(2)} GB` : b >= 1e6 ? `${Math.round(b / 1e6)} MB` : `${Math.round(b / 1e3)} KB`;

function totals() {
  const rows = FILES.map(estimate);
  const ratio = rows.reduce((s, r, i) => s + r.kb, 0) / FILES.reduce((s, f) => s + f.kb, 0);
  const out = BATCH.bytesIn * ratio;
  const over = Math.round(BATCH.count * rows.filter(r => r.flag === 'over').length / FILES.length);
  const kept = rows.some(r => r.flag === 'kept') ? BATCH.kept : 0;
  const clash = state.name.mode === 'keep' ? 0 : BATCH.clashes;
  return { out, saving: 1 - ratio, over, kept, clash, ready: BATCH.count - over - kept - clash, rows };
}

// ---------- component builders (mirror the Svelte components to come) ----------
const accentVars = a => `--accent:var(--accent-${a});--on-accent:var(--on-${a})`;

function keyBtn({ label = '', iconId, pressed, role = 'button', bind, value, act, aria, cls = '', tabindex, fid, disabled }) {
  const attrs = [
    `class="key ${cls}"`,
    role === 'radio' ? `role="radio" aria-checked="${!!pressed}"` : role === 'toggle' ? `aria-pressed="${!!pressed}"` : '',
    bind ? `data-bind="${bind}" data-value="${esc(value)}"` : '', act ? `data-act="${act}"` : '',
    aria ? `aria-label="${esc(aria)}"` : '', tabindex != null ? `tabindex="${tabindex}"` : '',
    fid ? `data-fid="${fid}"` : '', disabled ? 'disabled' : '',
  ].join(' ');
  return `<button type="button" ${attrs}>${iconId ? icon(iconId) : ''}${label}</button>`;
}

function radioGroup({ label, bind, options, value, n, accent, neutral, fid }) {
  const hasChecked = options.some(o => String(o.id) === String(value));
  return `<div class="keys ${neutral ? 'keys--neutral' : ''}" role="radiogroup" aria-label="${esc(label)}" style="--n:${n ?? options.length};${accent ? accentVars(accent) : ''}">${
    options.map((o, i) => {
      const on = String(o.id) === String(value);
      return keyBtn({ label: o.label, iconId: o.icon, role: 'radio', pressed: on, bind, value: o.id, aria: o.aria, tabindex: on || (!hasChecked && i === 0) ? 0 : -1, fid: `${fid}-${o.id}` });
    }).join('')}</div>`;
}

function stepSlider({ label, bind, stops, index, fid, disabled }) {
  const n = stops.length - 1;
  return `<div><div class="slider" role="slider" tabindex="${disabled ? -1 : 0}" aria-label="${esc(label)}" aria-valuemin="0" aria-valuemax="${n}" aria-valuenow="${index}"
      aria-valuetext="${esc(stops[index].label)}" data-slider="${bind}" data-steps="${n}" data-fid="${fid}" style="--v:${index / n}${disabled ? ';opacity:.45' : ''}" ${disabled ? 'aria-disabled="true"' : ''}>
      <span class="slider__slot"></span><span class="slider__fill"></span>${stops.map((s, i) => `<span class="slider__tick" style="--at:${i / n}"></span>`).join('')}<span class="slider__cap"></span></div>
    <div class="slider__stops">${stops.map((s, i) => `<button type="button" class="slider__stop" style="--at:${i / n}" data-bind="${bind}" data-value="${i}" aria-current="${i === index}" tabindex="-1">${s.label}</button>`).join('')}</div></div>`;
}

function rangeSlider({ label, bind, min, max, value, unit = '', fid }) {
  const v = (value - min) / (max - min);
  return `<div class="slider" role="slider" tabindex="0" aria-label="${esc(label)}" aria-valuemin="${min}" aria-valuemax="${max}" aria-valuenow="${value}" aria-valuetext="${value}${unit}"
     data-slider="${bind}" data-min="${min}" data-max="${max}" data-fid="${fid}" style="--v:${v}"><span class="slider__slot"></span><span class="slider__fill"></span>
     ${[0, .2, .4, .6, .8, 1].map(a => `<span class="slider__tick" style="--at:${a}"></span>`).join('')}<span class="slider__cap"></span></div>`;
}

// ---------- regions ----------
function renderHead() {
  const r = state.recipe;
  $('#head').innerHTML = `
    <button class="grille" aria-label="Add images: drop them anywhere, or press to choose" data-fid="grille"></button>
    <div class="word">wrangler <span class="legend legend--quiet">batch image unit</span></div>
    <div class="recipe">
      ${keyBtn({ iconId: 'prev', act: 'recipe-prev', aria: 'Previous recipe', cls: 'key--icon', fid: 'rprev' })}
      <div class="screen screen--fx" role="status" aria-label="Recipe"><span class="num">${String(r.index + 1).padStart(2, '0')}</span><span>${esc(r.names[r.index].toUpperCase())}</span>${r.edited ? '<span class="edited">● edited</span>' : ''}</div>
      ${keyBtn({ iconId: 'next', act: 'recipe-next', aria: 'Next recipe', cls: 'key--icon', fid: 'rnext' })}
      ${keyBtn({ label: 'Save', act: 'save', fid: 'save' })}
    </div>
    <div class="right">${keyBtn({ iconId: 'plus', label: 'Add', cls: 'key--dark', act: 'add', fid: 'add' })}${keyBtn({ iconId: 'command', cls: 'key--dark key--icon', act: 'palette', aria: 'Command palette', fid: 'cmd' })}</div>`;
}

function renderMain() {
  const t = totals();
  const tabs = [
    { id: 'all', label: 'All', n: BATCH.count }, { id: 'ready', label: 'Ready', n: t.ready },
    { id: 'kept', label: 'Kept', n: t.kept }, { id: 'clash', label: 'Clash', n: t.clash, alarm: true },
    { id: 'over', label: 'Over', n: t.over, alarm: true }, { id: 'failed', label: 'Failed', n: 0, alarm: true },
  ];
  const f = FILES[state.selected];
  const e = estimate(f);
  const geo = geometryChanges();
  const view = state.view === 'auto' ? (geo ? 'result' : 'split') : state.view;
  const photo = n => `photos/p${n}.jpg`;
  const shown = state.tab === 'all' ? FILES.map((x, i) => i) : FILES.map((x, i) => i).filter(i => {
    const fl = t.rows[i].flag; return state.tab === 'ready' ? !fl : fl === state.tab;
  });
  const flagText = { kept: 'kept original', clash: 'name taken → -2', over: 'over limit' };
  $('#main').innerHTML = `
    <div class="sbar">
      <span class="path">${esc(BATCH.folder)}<b>${esc(BATCH.name)}</b> · ${bytesText(BATCH.bytesIn)}</span>
      <div class="tabs" role="tablist" aria-label="Show">${tabs.map(x => `<button type="button" role="tab" class="tab ${x.alarm && x.n ? 'tab--alarm' : ''}" aria-selected="${state.tab === x.id}" data-bind="tab" data-value="${x.id}" data-fid="tab-${x.id}">${x.label.toUpperCase()}<b>${x.n.toLocaleString()}</b></button>`).join('')}</div>
    </div>
    <div class="sbody">
      <div class="sheet" role="listbox" aria-label="Images, as they will come out" aria-multiselectable="false">${shown.length ? shown.map(i => {
        const x = FILES[i], r = t.rows[i];
        const pct = Math.round((1 - r.kb / x.kb) * 100);
        const save = r.flag === 'kept' ? 'kept' : pct >= 0 ? `−${pct}%` : `+${-pct}%`;
        return `<button type="button" class="tile" role="option" aria-selected="${i === state.selected}" data-bind="selected" data-value="${i}" data-fid="tile-${i}" tabindex="${i === state.selected ? 0 : -1}"
            aria-label="${esc(outName(x, i))}, ${kbText(x.kb)} to ${kbText(r.kb)}${r.flag ? ', ' + flagText[r.flag] : ''}">
          <div class="tile__img"><img src="${photo(x.photo)}" alt="" loading="lazy">${r.flag ? `<span class="flag ${r.flag === 'kept' ? '' : 'flag--alarm'}">${r.flag === 'kept' ? '= ' : '! '}${r.flag.toUpperCase()}</span>` : ''}</div>
          <div class="tile__name">${esc(outName(x, i))}</div>
          <div class="tile__meta"><span>${kbText(x.kb)} → <b>${kbText(r.kb)}</b></span><span style="color:var(--screen-1)">${save}</span></div>
        </button>`;
      }).join('') : `<p style="grid-column:1/-1;color:var(--screen-2)">No images in this group.</p>`}</div>
      <div class="mon">
        <div class="view ${view === 'split' ? 'split' : ''}" id="view" ${view === 'split' ? `role="slider" tabindex="0" aria-label="Before and after split" aria-valuemin="0" aria-valuemax="100" aria-valuenow="${state.split ?? 50}" data-fid="view"` : ''} style="--split:${state.split ?? 50}%">
          ${view === 'split' ? `
            <img src="${photo(f.photo)}" alt="Original">
            <img class="after" src="${photo(f.photo)}" alt="Result" style="filter:contrast(1.01)"><span class="handle"></span>
            <span class="chip l">BEFORE</span><span class="chip r">AFTER</span>` : view === 'result' ? `
            <img src="${photo(f.photo)}" alt="Result">
            ${state.extras.canvas.on ? '<span class="canvas-guide" aria-hidden="true"></span>' : ''}
            <div class="pip"><img src="${photo(f.photo)}" alt="Original" style="background:#d9d4cc">${state.extras.trim.on ? '<span class="trim-box"></span>' : ''}</div>
            <span class="chip r">RESULT</span>` : `
            <img src="${photo(f.photo)}" alt="Original" style="object-fit:contain;background:#d9d4cc">${state.extras.trim.on ? '<span class="trim-box" style="position:absolute;left:37%;top:20%;width:26%;height:60%;border:1.5px solid var(--accent-trim)"></span>' : ''}
            <span class="chip r">ORIGINAL</span>`}
        </div>
        <div class="tabs mode-keys" role="radiogroup" aria-label="Monitor view">${[['result', 'Result'], ['original', 'Original'], ['split', 'Split']].map(([id, l]) =>
          `<button type="button" class="tab" role="radio" aria-checked="${view === id}" aria-selected="${view === id}" data-bind="view" data-value="${id}" data-fid="v-${id}" ${geo && id === 'split' ? 'title="Split works when framing is unchanged"' : ''}>${l.toUpperCase()}</button>`).join('')}</div>
        <dl class="facts">
          <dt>FILE</dt><dd><span class="from">${esc(f.src)} →</span> <span style="color:var(--accent-name)">${esc(outName(f, state.selected))}</span></dd>
          <dt>SIZE</dt><dd><span class="from">${f.w}×${f.h} →</span> <span style="color:var(--accent-size-text)">${e.dims.w}×${e.dims.h}</span></dd>
          <dt>WEIGHT</dt><dd><span class="from">${kbText(f.kb)} →</span> <span style="color:var(--accent-quality)">${kbText(e.kb)}</span> ${e.flag === 'kept' ? '· already smaller, kept' : (e.kb <= f.kb ? `· −${Math.round((1 - e.kb / f.kb) * 100)}%` : `· +${Math.round((e.kb / f.kb - 1) * 100)}% (bigger canvas)`)}</dd>
          <dt>LOOK</dt><dd><span style="color:var(--accent-format)">${e.fmt.label}</span> · ${state.quality.lossless ? 'lossless' : `${QUALITY_STOPS[state.quality.stop].label.toLowerCase()}, aims ≥ ${QUALITY_STOPS[state.quality.stop].score}`}</dd>
        </dl>
      </div>
    </div>
    <div class="hints shints"><span><kbd>←→</kbd>next</span><span><kbd>B</kbd>original</span><span><kbd>1–4</kbd>channels</span><span><kbd>⌘↵</kbd>run</span><span><kbd>⌘K</kbd>commands</span><span class="live">previews live</span></div>`;
}

function renderExtras() {
  const x = state.extras;
  const body = {
    rotate: () => radioGroup({ label: 'Turn', bind: 'extras.rotate.turn', options: [{ id: 90, label: '90° ↻' }, { id: 180, label: '180°' }, { id: 270, label: '90° ↺' }], value: x.rotate.turn, accent: 'tool', fid: 'rot' }),
    trim: () => `<div><div class="extra__head legend legend--quiet" style="display:flex;justify-content:space-between;margin-bottom:6px"><span>Tolerance</span><span>${x.trim.tolerance}%</span></div>${rangeSlider({ label: 'Trim tolerance', bind: 'extras.trim.tolerance', min: 0, max: 20, value: x.trim.tolerance, unit: '%', fid: 'trimtol' })}</div>`,
    canvas: () => `
      ${radioGroup({ label: 'Canvas shape', bind: 'extras.canvas.shape', options: [{ id: '1:1', label: '1:1' }, { id: '4:5', label: '4:5' }, { id: '3:2', label: '3:2' }, { id: 'any', label: 'Any' }], value: x.canvas.shape, accent: 'canvas', fid: 'cshape' })}
      <div class="canvas-row"><div class="screen mini-canvas" data-dragzone="fill" style="--fill:${x.canvas.fill / 100};--bd:${{ auto: '#d9bca2', white: '#ffffff', clear: 'repeating-conic-gradient(#3a3a3a 0 25%, #222 0 50%) 0 0/10px 10px', pick: '#c9d8e6' }[x.canvas.backdrop]}">
          <span class="prod"></span><span class="handle" role="slider" tabindex="0" data-drag="fill" data-fid="cfill" aria-label="How much of the canvas the product fills" aria-valuemin="50" aria-valuemax="100" aria-valuenow="${x.canvas.fill}" aria-valuetext="${x.canvas.fill} percent"></span></div>
        <div class="canvas-val"><span class="legend legend--quiet">Product fills</span><span class="big" id="fillv">${x.canvas.fill}%</span><span class="note">drag the corner</span></div></div>
      ${radioGroup({ label: 'Backdrop', bind: 'extras.canvas.backdrop', options: [{ id: 'auto', label: 'Auto' }, { id: 'white', label: 'White' }, { id: 'clear', label: 'Clear' }, { id: 'pick', label: 'Pick' }], value: x.canvas.backdrop, accent: 'canvas', fid: 'cback' })}`,
    sharpen: () => `<div><div class="legend legend--quiet" style="display:flex;justify-content:space-between;margin-bottom:6px"><span>Amount</span><span>${x.sharpen.amount}%</span></div>${rangeSlider({ label: 'Sharpen amount', bind: 'extras.sharpen.amount', min: 0, max: 100, value: x.sharpen.amount, unit: '%', fid: 'sharp' })}</div>`,
  };
  const order = ['trim', 'canvas', 'sharpen', 'rotate', 'location'];
  const on = order.filter(id => x[id].on).length;
  const chain = [...CHANNELS.map(c => ({ ...c, kind: 'ch' })), ...EXTRAS.map(e => ({ ...e, kind: 'x' }))]
    .sort((a, b) => a.stage - b.stage)
    .filter(s => s.kind === 'x' ? x[s.id].on : !(s.id === 'size' && state.size.preset === 'orig' && !x.canvas.on) && !(s.id === 'name' && state.name.mode === 'keep') && !(s.id === 'format' && state.format.id === 'same'));
  $('#extras').innerHTML = `
    <div class="extras__head"><span class="legend">Extras</span><span class="legend legend--quiet">${on} on</span></div>
    ${order.map(id => {
      const def = extraById(id), s = x[id];
      return `<div class="extra" style="${accentVars(def.accent)}">
        ${keyBtn({ iconId: def.icon, role: 'toggle', pressed: s.on, bind: `extras.${id}.on`, value: !s.on, aria: id === 'location' ? 'Remove location data' : def.label, cls: 'key--icon', fid: `x-${id}` })}
        <span class="extra__name">${def.label}</span><span class="extra__value">${def.summary(s)}</span>
      </div>${s.on && body[id] && (s.open ?? true) ? `<div class="extra__body" style="${accentVars(def.accent)}">${body[id]()}</div>` : ''}`;
    }).join('')}
    <div class="chain"><div class="legend legend--quiet" style="margin-bottom:6px">Runs in this order</div>
      <div class="screen screen--fx"><span style="display:flex;flex-wrap:wrap;gap:2px 6px;font-size:10.5px">${chain.map((s, i) =>
        `<span style="color:var(--accent-${s.accent === 'tool' ? 'tool' : s.accent}${s.accent === 'size' || s.accent === 'canvas' ? '-text' : ''});${s.accent === 'tool' ? 'color:var(--screen-1)' : ''}">${i ? '<span style="color:var(--screen-3)">› </span>' : ''}${s.label.toLowerCase()}</span>`).join('')}<span style="color:var(--screen-3)">› out</span></span></div></div>`;
}

// ---------- channel screens: each one is also the control ----------
const SIZE_STOPS = SIZE_PRESETS.filter(p => p.px);

function sizeScreen(f, e) {
  const d = e.dims, box = 84, maxSrc = Math.max(f.w, f.h), scale = box / Math.max(maxSrc, d.w, d.h);
  const sw = f.w * scale, sh = f.h * scale, ow = Math.max(8, d.w * scale), oh = Math.max(8, d.h * scale);
  const big = Math.max(d.w, d.h), cur = state.size.preset;
  const note = state.extras.canvas.on ? `<span style="color:var(--accent-canvas-text)">canvas ${state.extras.canvas.shape}</span>`
    : cur === 'orig' ? 'original size' : maxSrc < Number(cur) ? 'smaller, left as is' : `from ${f.w}×${f.h}`;
  return `<div class="w-size">
    <div class="diagram" data-dragzone="size" style="width:${box}px;height:${box}px">
      <span class="src" style="width:${sw}px;height:${sh}px"></span>
      <span class="out glow" style="width:${ow}px;height:${oh}px;color:var(--accent-size)"></span>
      <span class="handle" role="slider" tabindex="0" data-drag="size" data-fid="size-handle" style="left:${ow}px;bottom:${oh}px"
        aria-label="Output size, drag or use arrow keys" aria-valuemin="100" aria-valuemax="${maxSrc}" aria-valuenow="${big}" aria-valuetext="${big} pixels on the long side"></span>
    </div>
    <div class="vals">
      <button type="button" class="scr-value glow value-btn" data-act="edit-size" data-fid="size-type" style="color:var(--accent-size-text)" aria-label="Type a size in pixels, now ${big}">${big}</button>
      <span class="sub">${d.w} × ${d.h} px</span><span class="scr-label">${note}</span>
    </div>
    <div class="ruler" role="radiogroup" aria-label="Size presets">${[{ id: 'orig', label: 'orig', aria: 'Original size' }, ...SIZE_STOPS].map(p => {
      const on = String(p.id) === String(cur);
      return `<button type="button" role="radio" aria-checked="${on}" data-bind="size.preset" data-value="${p.id}" data-fid="ruler-${p.id}" tabindex="${on ? 0 : -1}" ${p.aria ? `aria-label="${p.aria}"` : ''}>${p.label.toLowerCase()}</button>`;
    }).join('')}</div></div>`;
}

function formatScreen() {
  const browsing = state.format.browsing;
  const list = browsing ? FORMATS.filter(x => !x.key) : FORMATS.filter(x => x.key);
  const cur = fmtById(state.format.id);
  const moreOn = cur && !cur.key;
  const rows = list.map(x => {
    const on = state.format.id === x.id;
    return `<button type="button" role="radio" aria-checked="${on}" data-bind="format.id" data-value="${x.id}" data-fid="fmt-${x.id}" tabindex="${on ? 0 : -1}">${x.label}</button>`;
  }).join('') + (browsing
    ? `<button type="button" class="more" data-act="format-more" data-fid="fmt-back" tabindex="-1">‹ back</button>`
    : `<button type="button" role="radio" class="more" aria-checked="${!!moreOn}" data-act="format-more" data-fid="fmt-more" tabindex="${moreOn ? 0 : -1}">${moreOn ? cur.label : 'more ›'}</button>`);
  let spec;
  if (!cur || cur.id === 'same') spec = `<div class="spec"><div class="scr-value glow" style="color:var(--accent-format)">same</div><span class="scr-label">each file keeps its format<br>HEIC → JPG · TIFF stays</span></div>`;
  else {
    const vs = Math.round((cur.factor - 1) * 100);
    spec = `<div class="spec"><div class="scr-value glow" style="color:var(--accent-format)">${cur.name}</div>
      <div class="caps"><span class="cap ${cur.alpha ? 'on' : ''}">alpha</span><span class="cap ${cur.motion ? 'on' : ''}">motion</span><span class="cap ${cur.lossless ? 'on' : ''}">lossless</span></div>
      <div class="vs"><div class="bar"><i class="glow" style="width:${Math.min(100, cur.factor * 50)}%;color:var(--accent-format)"></i></div><span><b>${vs === 0 ? '= JPG' : (vs > 0 ? '+' : '−') + Math.abs(vs) + '% vs JPG'}</b> · ${cur.browsers ? `${cur.browsers}% browsers` : 'not for web'}</span></div>
      ${cur.hint ? `<span class="scr-label">${cur.hint}</span>` : ''}</div>`;
  }
  return `<div class="w-fmt"><div class="flist" role="radiogroup" aria-label="Output format">${rows}</div>${spec}</div>`;
}

function qualityScreen(f, e) {
  const q = state.quality, mp = (e.dims.w * e.dims.h) / (2048 * 2048);
  const kbs = QUALITY_STOPS.map(s => s.kb * e.fmt.factor * mp * f.detail);
  const max = Math.max(...kbs, q.limitKb) * 1.15, H = 62, h = kb => Math.max(3, kb / max * H);
  return `<div class="w-qual" data-dragzone="limit" style="--inset:22px;--chart:${H}px">
    <div class="qtop"><span>${q.lossless ? 'lossless' : `${Math.round(e.kb)} KB here`}${e.flag === 'over' ? ' <b style="color:var(--status-alarm)">! over</b>' : ''}</span><button type="button" class="nb" aria-pressed="${q.neverBigger}" data-bind="quality.neverBigger" data-value="${!q.neverBigger}" data-fid="qnb" aria-label="Never make a file bigger">never bigger ${q.neverBigger ? '✓' : ''}</button></div>
    <div class="chart" role="radiogroup" aria-label="Quality">${kbs.map((kb, i) => {
      const on = i === q.stop && !q.lossless, over = q.limitOn && kb > q.limitKb;
      return `<button type="button" role="radio" aria-checked="${on}" data-bind="quality.stop" data-value="${i}" data-fid="q-${i}" tabindex="${on ? 0 : -1}" style="--at:${i / 3}"
        aria-label="${QUALITY_STOPS[i].label}, about ${Math.round(kb)} KB${over ? ', over the limit' : ''}" ${q.lossless ? 'disabled' : ''}>
        <span class="kb">${Math.round(kb)}</span><span class="bar ${on ? 'on' : ''} ${over ? 'over' : ''}" style="height:${h(kb)}px"></span><span class="lab">${QUALITY_STOPS[i].label.toLowerCase()}</span></button>`;
    }).join('')}
    ${q.limitOn ? `<span class="limit" role="slider" tabindex="0" data-drag="limit" data-fid="q-limit" aria-orientation="vertical" aria-label="Maximum file size, drag or use arrow keys"
        aria-valuemin="20" aria-valuemax="5000" aria-valuenow="${q.limitKb}" aria-valuetext="${q.limitKb} KB" style="bottom:calc(16px + ${h(q.limitKb)}px)"><span>max ${q.limitKb} KB</span></span>` : ''}</div></div>`;
}

function nameScreen(f) {
  const m = state.name.mode;
  const editor = m === 'pattern'
    ? `<div class="pat"><input value="${esc(state.name.pattern)}" data-input="name.pattern" aria-label="Filename pattern" data-fid="npat" spellcheck="false"></div>`
    : m === 'replace'
      ? `<div class="pat two"><input value="${esc(state.name.find ?? 'DSC_')}" data-input="name.find" aria-label="Find" data-fid="nf" spellcheck="false"><span>→</span><input value="${esc(state.name.with ?? 'spring-')}" data-input="name.with" aria-label="Replace with" data-fid="nw" spellcheck="false"></div>`
      : `<div class="pat"><span class="scr-label">names stay as they are</span></div>`;
  const ins = m === 'pattern' ? `<span class="ins">${[['{n:000}', '#'], ['{name}', 'name'], ['{date}', 'date']].map(([t, l]) => `<button type="button" data-act="tok" data-tok="${t}" data-fid="tok-${l}" aria-label="Insert ${l === '#' ? 'number' : l}">+${l}</button>`).join('')}</span>` : '<span></span>';
  return `<div class="w-name">${editor}<div class="preview">${namePreview(f)}</div><div class="foot">${ins}<span>${m === 'keep' ? 'ext follows format' : `${BATCH.clashes} taken → -2`}</span></div></div>`;
}

function namePreview(f) {
  const i = state.selected, m = state.name.mode, nm = outName(f, i);
  const pad = String(i + 1).padStart(3, '0');
  const shown = m === 'pattern' ? esc(nm).replace(pad, `<span class="tok">${pad}</span>`) : esc(nm);
  return `<span class="src">${esc(f.src)}</span><span class="out glow" style="color:var(--accent-name)">→ ${shown}</span>
    <div class="seq">${m === 'keep' ? '' : `<span>${esc(renderPattern(state.name.pattern, f, 0))} … <b>${esc(renderPattern(state.name.pattern, f, BATCH.count - 1))}</b></span>`}</div>`;
}

function channelScreen(id) {
  const f = FILES[state.selected], e = estimate(f);
  return { size: () => sizeScreen(f, e), format: formatScreen, quality: () => qualityScreen(f, e), name: () => nameScreen(f) }[id]();
}

function channelControls(id) {
  const q = state.quality;
  if (id === 'size') return radioGroup({ label: 'Fit', bind: 'size.mode', options: SIZE_MODES, value: state.size.mode, accent: 'size', fid: 'sm' });
  if (id === 'format') return `<div class="keys" style="--n:2;${accentVars('format')}">${keyBtn({ label: 'Auto per image', role: 'toggle', pressed: state.format.auto, bind: 'format.auto', value: !state.format.auto, fid: 'fauto' })}
    ${keyBtn({ label: `<span class="sw" style="background:${state.format.matte}"></span>Matte`, act: 'matte', aria: `Colour behind transparent areas in JPG, now ${state.format.matte === '#ffffff' ? 'white' : 'black'}`, fid: 'fmatte' })}</div>`;
  if (id === 'quality') return `<div class="keys" style="--n:2;${accentVars('quality')}">${keyBtn({ label: 'Max size', role: 'toggle', pressed: q.limitOn, bind: 'quality.limitOn', value: !q.limitOn, fid: 'qlim' })}
    ${keyBtn({ label: 'Lossless', role: 'toggle', pressed: q.lossless, bind: 'quality.lossless', value: !q.lossless, fid: 'qll' })}</div>`;
  if (id === 'name') return radioGroup({ label: 'Names', bind: 'name.mode', options: NAME_MODES, value: state.name.mode, accent: 'name', fid: 'nm' });
}

function channelSummary(id) {
  const f = FILES[state.selected], e = estimate(f);
  if (id === 'size') return state.size.preset === 'orig' ? 'original' : `${SIZE_MODES.find(m => m.id === state.size.mode).label.toLowerCase()} ${state.size.preset}`;
  if (id === 'format') return state.format.id === 'same' ? 'unchanged' : e.fmt.label;
  if (id === 'quality') return state.quality.lossless ? 'lossless' : `${QUALITY_STOPS[state.quality.stop].label.toLowerCase()}${state.quality.limitOn ? ` · ≤${state.quality.limitKb} KB` : ''}`;
  if (id === 'name') return state.name.mode === 'keep' ? 'kept' : state.name.mode;
}

function renderChannels(light) {
  const host = $('#channels');
  if (!light || !host.children.length) {
    host.innerHTML = CHANNELS.map(c => `<section class="channel panel" style="${accentVars(c.accent)}" aria-labelledby="ch-${c.id}" id="chan-${c.id}">
      <div class="channel__head"><span class="legend" id="ch-${c.id}" style="display:flex;gap:7px;align-items:center"><span aria-hidden="true" style="color:var(--accent);font-size:11px">${c.glyph}</span>${c.label}</span><span class="channel__summary" data-sum="${c.id}"></span></div>
      <div class="screen screen--fx" data-scr="${c.id}"></div>
      <div class="channel__controls" data-ctl="${c.id}"></div></section>`).join('');
  }
  CHANNELS.forEach(c => {
    $(`[data-sum="${c.id}"]`).textContent = channelSummary(c.id);
    const scr = $(`[data-scr="${c.id}"]`);
    if (light && scr.contains(document.activeElement) && document.activeElement.matches('input')) {
      const pv = scr.querySelector('.preview'); if (pv) pv.innerHTML = namePreview(FILES[state.selected]);
    } else scr.innerHTML = channelScreen(c.id);
    if (!light) $(`[data-ctl="${c.id}"]`).innerHTML = channelControls(c.id);
  });
}

function renderOutput() {
  const t = totals();
  const dest = { replace: `Replaces ${BATCH.count.toLocaleString()} originals · undo for 30 days`, beside: `Saves ${BATCH.count.toLocaleString()} new files beside the originals`, folder: `Saves ${BATCH.count.toLocaleString()} into ~/Desktop/Wrangled` }[state.dest];
  const lit = Math.max(1, Math.round(24 * (1 - t.saving)));
  $('#output').innerHTML = `
    <div class="screen screen--fx meter" role="status" aria-label="Estimated result: ${bytesText(BATCH.bytesIn)} in, about ${bytesText(t.out)} out">
      <div class="row"><span>IN</span><div class="ladder ladder--in" aria-hidden="true">${'<i class="on"></i>'.repeat(24)}</div><span>${bytesText(BATCH.bytesIn)}</span></div>
      <div class="row"><span>OUT</span><div class="ladder ladder--out" aria-hidden="true">${'<i class="on"></i>'.repeat(lit)}${'<i></i>'.repeat(24 - lit)}</div><span>≈${bytesText(t.out)}</span></div>
      <div class="readout"><span class="led-readout">−${Math.round(t.saving * 100)}%</span><small>saves ${bytesText(BATCH.bytesIn - t.out)}</small></div>
      <div class="stats"><span><b>${BATCH.count.toLocaleString()}</b> images</span>${t.kept ? `<span><b>${t.kept}</b> kept</span>` : ''}${t.over ? `<span><b style="color:var(--status-alarm)">${t.over}</b> over</span>` : ''}<span><b>${t.clash}</b> clash</span></div>
    </div>
    <div class="dest"><span class="legend legend--quiet">Save to</span>${radioGroup({ label: 'Save to', bind: 'dest', options: [{ id: 'beside', label: 'Beside' }, { id: 'folder', label: 'Folder…' }, { id: 'replace', label: 'Replace' }], value: state.dest, neutral: true, fid: 'dest' })}</div>
    <div class="go">
      ${keyBtn({ label: 'Undo', iconId: 'undo', cls: 'key--tall', act: 'undo', fid: 'undo', disabled: true, aria: 'Undo last run: nothing to undo yet' })}
      <button type="button" class="run" data-act="run" data-fid="run" aria-describedby="run-note">Run</button>
      <span class="note" id="run-note">${dest}</span>
    </div>`;
}

function renderQuick() {
  $('#quick').innerHTML = `
    <div style="display:flex;justify-content:space-between;align-items:center"><span class="legend">wrangler · quick</span><span class="led led--on" style="--led:var(--accent-format)"></span></div>
    <div class="screen screen--fx">
      <div class="row"><span>SHOPIFY PRODUCT</span><b>48/48</b></div>
      <div class="ladder ladder--out">${'<i class="on"></i>'.repeat(24)}</div>
      <span class="led-readout">−86%</span>
      <div class="row"><span>41.2 MB → 5.8 MB</span><b>3 s</b></div>
      <div class="row"><span>replaced in place</span><span>2 kept</span></div>
    </div>
    <div class="keys" style="--n:2">${keyBtn({ iconId: 'undo', label: 'Undo' })}${keyBtn({ label: 'Open', cls: 'key--dark' })}</div>`;
}

// ---------- state updates ----------
function setPath(path, raw) {
  const keys = path.split('.');
  let o = state; keys.slice(0, -1).forEach(k => o = o[k]);
  const last = keys.at(-1), cur = o[last];
  let v = raw;
  if (path === 'size.preset') v = raw === 'orig' ? 'orig' : Number(raw);
  else if (typeof cur === 'number') v = Number(raw);
  else if (typeof cur === 'boolean') v = raw === true || raw === 'true';
  o[last] = v;
  if (path === 'format.id') state.format.browsing = false;
  if (path === 'view' || path === 'selected') {} else state.recipe.edited = true;
}

function render(light = false) {
  const fid = document.activeElement?.dataset?.fid;
  if (!light) { renderHead(); renderExtras(); }
  renderMain(); renderChannels(light); renderOutput();
  document.querySelectorAll('.key, .run').forEach(k => { if (!k.style.getPropertyValue('--gx')) { k.style.setProperty('--gx', `${(k.textContent.length * 37) % 256}px`); k.style.setProperty('--gy', `${(k.textContent.length * 91) % 256}px`); } });
  if (fid && !light) document.querySelector(`[data-fid="${fid}"]`)?.focus();
}

// clicks: any [data-bind] writes its value; [data-act] runs an action
document.addEventListener('click', e => {
  const el = e.target.closest('[data-bind],[data-act]');
  if (!el || el.closest('.tune')) return;
  if (el.dataset.bind) { setPath(el.dataset.bind, el.dataset.value); render(); return; }
  const a = el.dataset.act, r = state.recipe;
  if (a === 'recipe-prev' || a === 'recipe-next') { r.index = (r.index + (a === 'recipe-next' ? 1 : -1) + r.names.length) % r.names.length; r.edited = false; }
  if (a === 'save') r.edited = false;
  if (a === 'format-more') { state.format.browsing = !state.format.browsing; }
  if (a === 'matte') state.format.matte = state.format.matte === '#ffffff' ? '#000000' : '#ffffff';
  if (a === 'tok') { state.name.pattern += el.dataset.tok; r.edited = true; }
  if (a === 'edit-size') {
    const btn = el, input = document.createElement('input');
    Object.assign(input, { type: 'number', min: 100, max: 20000, step: 10, value: btn.textContent.trim(), className: 'scr-value value-input' });
    input.setAttribute('aria-label', 'Size in pixels, press Enter to apply');
    btn.replaceWith(input); input.focus(); input.select();
    const done = ok => { if (ok && Number(input.value) >= 100) { state.size.preset = Number(input.value); r.edited = true; } render(); $('[data-fid="size-type"]')?.focus(); };
    input.addEventListener('keydown', ev => { if (ev.key === 'Enter') done(true); if (ev.key === 'Escape') done(false); });
    input.addEventListener('blur', () => done(true), { once: true });
    return;
  }
  render();
});

document.addEventListener('input', e => {
  const el = e.target.closest('[data-input]');
  if (!el) return;
  setPath(el.dataset.input, el.type === 'number' ? Number(el.value) : el.value);
  render(true);
});

// keyboard: roving focus in radio groups, arrows on sliders and the sheet, global shortcuts
document.addEventListener('keydown', e => {
  const t = e.target;
  if (t.closest('.tune')) return;
  const group = t.closest('[role=radiogroup]');
  const arrows = { ArrowRight: 1, ArrowDown: 1, ArrowLeft: -1, ArrowUp: -1 };
  if (group && arrows[e.key] && t.matches('[role=radio]')) {
    const items = [...group.querySelectorAll('[role=radio]:not([disabled])')];
    const next = items[(items.indexOf(t) + arrows[e.key] + items.length) % items.length];
    e.preventDefault(); next.focus(); next.click(); return;
  }
  if (t.matches('[data-slider]') && (arrows[e.key] || e.key === 'Home' || e.key === 'End')) {
    e.preventDefault(); nudgeSlider(t, e.key === 'Home' ? -1e9 : e.key === 'End' ? 1e9 : arrows[e.key] * (e.shiftKey ? 5 : 1)); return;
  }
  if (t.closest('.sheet') && arrows[e.key]) {
    const step = { ArrowRight: 1, ArrowLeft: -1, ArrowDown: 4, ArrowUp: -4 }[e.key];
    state.selected = Math.max(0, Math.min(FILES.length - 1, state.selected + step)); e.preventDefault(); render();
    document.querySelector(`[data-fid="tile-${state.selected}"]`)?.focus(); return;
  }
  if (t.matches('#view.split') && arrows[e.key]) { state.split = Math.max(0, Math.min(100, (state.split ?? 50) + arrows[e.key] * 5)); e.preventDefault(); render(); return; }
  if (t.matches('input')) return;
  if (e.key === 'Escape' && state.format.browsing) { state.format.browsing = false; render(); }
  if (e.key.toLowerCase() === 'b') { state.view = state.view === 'original' ? 'auto' : 'original'; render(); }
  if (['1', '2', '3', '4'].includes(e.key)) { const c = CHANNELS[Number(e.key) - 1]; $(`#chan-${c.id} [tabindex="0"], #chan-${c.id} .slider`)?.focus(); }
});

function nudgeSlider(el, delta) {
  const path = el.dataset.slider;
  if (el.dataset.steps) { const n = Number(el.dataset.steps); setPath(path, Math.max(0, Math.min(n, Number(el.getAttribute('aria-valuenow')) + Math.sign(delta) * (Math.abs(delta) > 1e8 ? n : 1)))); }
  else { const min = Number(el.dataset.min), max = Number(el.dataset.max); setPath(path, Math.max(min, Math.min(max, Number(el.getAttribute('aria-valuenow')) + delta))); }
  render();
}

// pointer: sliders, the before/after split, and the draggable parts of the screens
const sizeFromFrac = (frac, f) => {
  const maxSrc = Math.max(f.w, f.h);
  let px = Math.round(Math.max(0.03, Math.min(1, frac)) * maxSrc / 10) * 10;
  const snap = SIZE_STOPS.find(p => Math.abs(p.px - px) < maxSrc * 0.035);
  return Math.max(100, snap ? snap.px : px);
};
function qualityMax(f) {
  const e = estimate(f), mp = (e.dims.w * e.dims.h) / (2048 * 2048);
  return Math.max(...QUALITY_STOPS.map(s => s.kb * e.fmt.factor * mp * f.detail), state.quality.limitKb) * 1.15;
}
const DRAGS = {
  size(ev) {
    const d = $('[data-scr="size"] .diagram').getBoundingClientRect(), f = FILES[state.selected];
    const frac = Math.max((ev.clientX - d.left) / d.width, (d.bottom - ev.clientY) / d.height);
    const px = sizeFromFrac(frac, f);
    if (state.extras.canvas.on || state.size.preset !== px) { state.size.preset = px; state.recipe.edited = true; }
  },
  limit(ev) {
    const c = $('[data-scr="quality"] .chart').getBoundingClientRect(), f = FILES[state.selected];
    const kb = (c.bottom - 16 - ev.clientY) / 62 * qualityMax(f);
    state.quality.limitKb = Math.max(20, Math.min(5000, Math.round(kb / 10) * 10)); state.recipe.edited = true;
  },
  fill(ev) {
    const el = $('.mini-canvas'), r = el.getBoundingClientRect();
    const dx = Math.abs(ev.clientX - (r.left + r.width / 2)), dy = Math.abs(ev.clientY - (r.top + r.height / 2));
    const fill = Math.round(Math.max(50, Math.min(100, Math.max(dx, dy) * 2 / r.width * 100)));
    state.extras.canvas.fill = fill; el.style.setProperty('--fill', fill / 100);
    const v = $('#fillv'); if (v) v.textContent = fill + '%'; state.recipe.edited = true;
  },
};

document.addEventListener('pointerdown', e => {
  const s = e.target.closest('[data-slider]');
  const v = e.target.closest('#view.split');
  const drag = e.target.closest('[data-drag]')?.dataset.drag || e.target.closest('[data-dragzone="size"],[data-dragzone="fill"]')?.dataset.dragzone;
  if (!s && !v && !drag) return;
  e.preventDefault();
  (e.target.closest('[tabindex]') || s)?.focus({ preventScroll: true });
  const move = ev => {
    if (drag) { DRAGS[drag](ev); render(true); return; }
    const el = s || $('#view.split'); if (!el) return;
    const r = el.getBoundingClientRect();
    if (v) { state.split = Math.round(Math.max(0, Math.min(100, (ev.clientX - r.left) / r.width * 100))); el.style.setProperty('--split', state.split + '%'); return; }
    const inset = 10, x = Math.max(0, Math.min(1, (ev.clientX - r.left - inset) / (r.width - 2 * inset)));
    if (el.dataset.steps) { const n = Number(el.dataset.steps), i = Math.round(x * n); el.style.setProperty('--v', i / n); el.setAttribute('aria-valuenow', i); setPath(el.dataset.slider, i); }
    else { const min = Number(el.dataset.min), max = Number(el.dataset.max), val = Math.round(min + x * (max - min)); el.style.setProperty('--v', x); el.setAttribute('aria-valuenow', val); setPath(el.dataset.slider, val); }
    render(true);
  };
  move(e);
  const up = () => { removeEventListener('pointermove', move); removeEventListener('pointerup', up); removeEventListener('pointercancel', up); render(); };
  addEventListener('pointermove', move); addEventListener('pointerup', up); addEventListener('pointercancel', up);
});

// keyboard for the screen handles
document.addEventListener('keydown', e => {
  const h = e.target.closest('[data-drag]'); if (!h) return;
  const dir = { ArrowUp: 1, ArrowRight: 1, ArrowDown: -1, ArrowLeft: -1, PageUp: 5, PageDown: -5 }[e.key];
  if (h.dataset.drag === 'size' && e.key === 'Enter') { e.preventDefault(); $('[data-act="edit-size"]').click(); return; }
  if (!dir) return;
  e.preventDefault(); e.stopPropagation();
  if (h.dataset.drag === 'size') {
    const f = FILES[state.selected], cur = Math.max(estimate(f).dims.w, estimate(f).dims.h);
    const list = SIZE_STOPS.map(p => p.px).filter(px => px <= Math.max(f.w, f.h)).concat(Math.max(f.w, f.h));
    const next = dir > 0 ? list.find(px => px > cur) : [...list].reverse().find(px => px < cur);
    if (next) state.size.preset = next;
  }
  if (h.dataset.drag === 'limit') state.quality.limitKb = Math.max(20, state.quality.limitKb + dir * (e.shiftKey ? 50 : 10));
  if (h.dataset.drag === 'fill') state.extras.canvas.fill = Math.max(50, Math.min(100, state.extras.canvas.fill + dir));
  state.recipe.edited = true; render();
}, true);

const fit = () => document.querySelectorAll('.stage').forEach(s => s.querySelector('.fit').style.transform = `scale(${s.clientWidth / 1440})`);
addEventListener('resize', fit); fit();
render(); renderQuick();
