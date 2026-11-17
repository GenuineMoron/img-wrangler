// SPDX-License-Identifier: AGPL-3.0-or-later
// Registries: the single place that lists formats, sizes, quality stops, extras and pipeline order.
// Layout code reads these; adding a format or an extra is a data change, not a layout change.

/** Output formats. `key: true` puts it on the main row; the rest live under More. */
const FORMATS = [
  { id: 'same', label: 'Same', key: true, name: 'same', note: 'keeps each file’s own format' },
  { id: 'jpg', label: 'JPG', key: true, name: 'jpg', ext: 'jpg', alpha: false, motion: false, lossless: false, browsers: 100, factor: 1.0 },
  { id: 'png', label: 'PNG', key: true, name: 'png', ext: 'png', alpha: true, motion: false, lossless: true, browsers: 100, factor: 2.6 },
  { id: 'webp', label: 'WebP', key: true, name: 'webp', ext: 'webp', alpha: true, motion: true, lossless: true, browsers: 97, factor: 0.7 },
  { id: 'avif', label: 'AVIF', key: true, name: 'avif', ext: 'avif', alpha: true, motion: true, lossless: true, browsers: 96, factor: 0.52, slow: true },
  { id: 'jxl', label: 'JPEG XL', name: 'jxl', ext: 'jxl', alpha: true, motion: true, lossless: true, browsers: 69, factor: 0.46, hint: 'newest; Safari, Chrome 155+' },
  { id: 'gif', label: 'GIF', name: 'gif', ext: 'gif', alpha: true, motion: true, lossless: false, browsers: 100, factor: 1.7, hint: '256 colours' },
  { id: 'tiff', label: 'TIFF', name: 'tiff', ext: 'tif', alpha: true, motion: false, lossless: true, browsers: 0, factor: 3.6, hint: 'print and archive' },
  { id: 'bmp', label: 'BMP', name: 'bmp', ext: 'bmp', alpha: false, motion: false, lossless: true, browsers: 100, factor: 6.0, hint: 'uncompressed' },
  { id: 'ico', label: 'ICO', name: 'ico', ext: 'ico', alpha: true, motion: false, lossless: true, browsers: 100, factor: 1.0, hint: 'favicons' },
  { id: 'qoi', label: 'QOI', name: 'qoi', ext: 'qoi', alpha: true, motion: false, lossless: true, browsers: 0, factor: 3.0, hint: 'fast lossless' },
];

const SIZE_PRESETS = [
  { id: 'orig', label: 'Orig', px: null, aria: 'Original size' },
  { id: 1200, label: '1200', px: 1200 }, { id: 1600, label: '1600', px: 1600 }, { id: 2048, label: '2048', px: 2048 },
  { id: 2560, label: '2560', px: 2560 }, { id: 3840, label: '4K', px: 3840, aria: '3840 pixels' },
];
const SIZE_MODES = [
  { id: 'long', label: 'Long side', aria: 'Resize the longest side' },
  { id: 'box', label: 'Fit in box', aria: 'Fit inside a square box' },
  { id: 'exact', label: 'Exact', aria: 'Exact size, crop to fill' },
];

/** Quality stops: the score each aims for, and the typical size of a 2048 px JPEG at that stop. */
const QUALITY_STOPS = [
  { id: 'small', label: 'Small', score: 65, kb: 243 },
  { id: 'balanced', label: 'Balanced', score: 75, kb: 411 },
  { id: 'high', label: 'High', score: 83, kb: 650 },
  { id: 'pristine', label: 'Pristine', score: 90, kb: 871 },
];

const NAME_MODES = [
  { id: 'keep', label: 'Keep' },
  { id: 'pattern', label: 'Pattern' },
  { id: 'replace', label: 'Replace' },
];

/** Drawn icons: one 16 px grid, 1.5 px stroke, round caps (matches Lucide geometry). */
const ICONS = {
  trim: '<path d="M4.5 1.5v10h10M1.5 4.5h10v10"/>',
  canvas: '<rect x="1.5" y="1.5" width="13" height="13" rx="1.5"/><rect x="5" y="5" width="6" height="6" rx=".5"/>',
  sharpen: '<path d="M8 1.5 9.6 6.4 14.5 8 9.6 9.6 8 14.5 6.4 9.6 1.5 8 6.4 6.4Z"/>',
  rotate: '<path d="M13.5 8.5A5.5 5.5 0 1 1 11.8 4.3"/><path d="M12.5 1.5v3.4H9.1"/>',
  location: '<path d="M8 14.5s4.8-4.2 4.8-8A4.8 4.8 0 0 0 3.2 6.5c0 3.8 4.8 8 4.8 8Z"/><circle cx="8" cy="6.4" r="1.6"/><path d="M2 2l12 12"/>',
  prev: '<path d="M10 3.5 5.5 8 10 12.5"/>',
  next: '<path d="M6 3.5 10.5 8 6 12.5"/>',
  plus: '<path d="M8 3v10M3 8h10"/>',
  more: '<path d="M4 6.5 8 10.5 12 6.5"/>',
  undo: '<path d="M5 3.5 2 6.5l3 3"/><path d="M2 6.5h7.5a4 4 0 0 1 0 8H7"/>',
  command: '<path d="M6 6V4.5a1.5 1.5 0 1 0-1.5 1.5H6Zm0 0h4m-4 0v4m4-4V4.5A1.5 1.5 0 1 1 11.5 6H10Zm0 0v4m0 0h1.5a1.5 1.5 0 1 1-1.5 1.5V10Zm0 0H6m0 0v1.5A1.5 1.5 0 1 1 4.5 10H6Z"/>',
};
const icon = (id, label) => `<svg viewBox="0 0 16 16" ${label ? `role="img" aria-label="${label}"` : 'aria-hidden="true"'}>${ICONS[id]}</svg>`;

/** Extras: optional steps. `stage` sets where each runs in the pipeline. */
const EXTRAS = [
  { id: 'rotate', label: 'Rotate', icon: 'rotate', accent: 'tool', stage: 1, summary: s => s.on ? `${s.turn}°` : 'off' },
  { id: 'trim', label: 'Trim', icon: 'trim', accent: 'trim', stage: 2, summary: s => s.on ? `${s.tolerance}% tolerance` : 'off' },
  { id: 'canvas', label: 'Canvas', icon: 'canvas', accent: 'canvas', stage: 3, summary: s => s.on ? `${s.shape} · fills ${s.fill}%` : 'off' },
  { id: 'sharpen', label: 'Sharpen', icon: 'sharpen', accent: 'tool', stage: 5, summary: s => s.on ? `${s.amount}%` : 'off' },
  { id: 'location', label: 'Location data', icon: 'location', accent: 'tool', stage: 9, summary: s => s.on ? 'removed' : 'kept' },
];

/** Channels: always present. `glyph` gives each a shape so colour is never the only cue. */
const CHANNELS = [
  { id: 'size', label: 'Size', glyph: '■', accent: 'size', stage: 4, key: '1' },
  { id: 'format', label: 'Format', glyph: '●', accent: 'format', stage: 6, key: '2' },
  { id: 'quality', label: 'Quality', glyph: '▲', accent: 'quality', stage: 7, key: '3' },
  { id: 'name', label: 'Name', glyph: '◆', accent: 'name', stage: 8, key: '4' },
];

/** A sample of the batch (the whole batch is 1,284 files, 1.82 GB). `detail` scales file size. */
const BATCH = { count: 1284, bytesIn: 1.82e9, folder: '~/Clients/Acme/', name: 'spring-catalogue', kept: 40, clashes: 2 };
const FILES = [
  { src: 'DSC_4471.JPG', photo: 1, w: 6000, h: 4000, kb: 4300, detail: 0.8 },
  { src: 'DSC_4472.JPG', photo: 2, w: 6000, h: 4000, kb: 3990, detail: 1.0 },
  { src: 'DSC_4473.JPG', photo: 3, w: 6000, h: 4000, kb: 6960, detail: 0.7 },
  { src: 'board_oak_v2.jpg', photo: 4, w: 4032, h: 3024, kb: 2150, detail: 1.1 },
  { src: 'cushion-green.tif', photo: 5, w: 5472, h: 3648, kb: 38900, detail: 1.5 },
  { src: 'IMG_0921.HEIC', photo: 6, w: 4032, h: 3024, kb: 2100, detail: 0.6 },
  { src: 'wallet-tan.jpg', photo: 7, w: 1600, h: 1600, kb: 180, detail: 1.0, kept: true },
  { src: 'succulent_01.jpg', photo: 8, w: 6000, h: 4000, kb: 2760, detail: 1.3 },
  { src: 'DSC_4480.JPG', photo: 1, w: 6000, h: 4000, kb: 4410, detail: 0.8 },
  { src: 'DSC_4481.JPG', photo: 2, w: 6000, h: 4000, kb: 4050, detail: 1.0, clash: true },
  { src: 'DSC_4482.JPG', photo: 3, w: 6000, h: 4000, kb: 6890, detail: 0.7 },
  { src: 'board_walnut.jpg', photo: 4, w: 4032, h: 3024, kb: 2330, detail: 1.2 },
  { src: 'cushion-ochre.tif', photo: 5, w: 5472, h: 3648, kb: 37200, detail: 1.6 },
  { src: 'IMG_0925.HEIC', photo: 6, w: 4032, h: 3024, kb: 2080, detail: 0.6 },
  { src: 'wallet-black.jpg', photo: 7, w: 3000, h: 3000, kb: 1900, detail: 1.0 },
  { src: 'succulent_02.jpg', photo: 8, w: 6000, h: 4000, kb: 2810, detail: 1.3 },
];

/** The recipe being edited. Every control reads and writes this. */
const DEFAULT_STATE = {
  recipe: { index: 2, names: ['Web ready', 'Just compress', 'Shopify product', 'Amazon main image', 'Email safe', 'Social square'], edited: true },
  size: { preset: 2048, mode: 'long' },
  format: { id: 'webp', browsing: false, cursor: 'jxl', auto: false, matte: '#ffffff' },
  quality: { stop: 1, limitOn: true, limitKb: 400, lossless: false, neverBigger: true },
  name: { mode: 'pattern', pattern: 'spring-24-{n:000}' },
  extras: {
    rotate: { on: false, turn: 90 },
    trim: { on: true, tolerance: 4, open: false },
    canvas: { on: true, shape: '1:1', fill: 85, backdrop: 'auto', open: true },
    sharpen: { on: false, amount: 40 },
    location: { on: true },
  },
  dest: 'replace',
  tab: 'all',
  selected: 2,
  view: 'auto',
};
