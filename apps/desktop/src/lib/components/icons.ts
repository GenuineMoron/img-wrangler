// SPDX-License-Identifier: AGPL-3.0-or-later
// Drawn icons: one 16 px grid, 1.5 px stroke, round caps and joins (Lucide geometry).
// Add an icon by adding a path here; Icon.svelte draws any of them.

export const ICONS = {
  plus: '<path d="M8 3v10M3 8h10"/>',
  minus: '<path d="M3 8h10"/>',
  x: '<path d="M4 4l8 8M12 4l-8 8"/>',
  check: '<path d="M3 8.5 6.5 12 13 4.5"/>',
  prev: '<path d="M10 3.5 5.5 8 10 12.5"/>',
  next: '<path d="M6 3.5 10.5 8 6 12.5"/>',
  down: '<path d="M4 6.5 8 10.5 12 6.5"/>',
  up: '<path d="M4 9.5 8 5.5 12 9.5"/>',
  undo: '<path d="M5 3.5 2 6.5l3 3"/><path d="M2 6.5h7.5a4 4 0 0 1 0 8H7"/>',
  command: '<path d="M6 6V4.5a1.5 1.5 0 1 0-1.5 1.5H6Zm0 0h4m-4 0v4m4-4V4.5A1.5 1.5 0 1 1 11.5 6H10Zm0 0v4m0 0h1.5a1.5 1.5 0 1 1-1.5 1.5V10Zm0 0H6m0 0v1.5A1.5 1.5 0 1 1 4.5 10H6Z"/>',
  // Lucide's gear, drawn on its 24 px grid and scaled to this one.
  settings: '<g transform="scale(.6667)" stroke-width="2.25"><path d="M9.671 4.136a2.34 2.34 0 0 1 4.659 0 2.34 2.34 0 0 0 3.319 1.915 2.34 2.34 0 0 1 2.33 4.033 2.34 2.34 0 0 0 0 3.831 2.34 2.34 0 0 1-2.33 4.033 2.34 2.34 0 0 0-3.319 1.915 2.34 2.34 0 0 1-4.659 0 2.34 2.34 0 0 0-3.32-1.915 2.34 2.34 0 0 1-2.33-4.033 2.34 2.34 0 0 0 0-3.831A2.34 2.34 0 0 1 6.35 6.051a2.34 2.34 0 0 0 3.319-1.915"/><circle cx="12" cy="12" r="3"/></g>',
  history: '<path d="M2.5 8a5.5 5.5 0 1 0 1.6-3.9"/><path d="M2 2.5v2.8h2.8"/><path d="M8 5v3l2 1.5"/>',
  folder: '<path d="M1.5 4.5a1 1 0 0 1 1-1h3.2l1.5 1.5h6.3a1 1 0 0 1 1 1v6.5a1 1 0 0 1-1 1h-11a1 1 0 0 1-1-1Z"/>',
  image: '<rect x="1.5" y="2.5" width="13" height="11" rx="1.5"/><circle cx="5.5" cy="6" r="1.2"/><path d="m14.5 10.5-3.5-3.5-7 6.5"/>',
  reveal: '<path d="M9.5 2.5h4v4M13.5 2.5 8 8"/><path d="M12 9.5v3a1 1 0 0 1-1 1H3.5a1 1 0 0 1-1-1V5a1 1 0 0 1 1-1h3"/>',
  save: '<path d="M2.5 3.5a1 1 0 0 1 1-1h7.6l2.4 2.4v7.6a1 1 0 0 1-1 1h-9a1 1 0 0 1-1-1Z"/><path d="M5 2.5v3h5v-3M4.5 13.5V9.5h7v4"/>',
  trash: '<path d="M2.5 4h11M6 4V2.5h4V4M4 4l.7 9.5h6.6L12 4"/>',
  stop: '<rect x="4" y="4" width="8" height="8" rx="1"/>',
  search: '<circle cx="7" cy="7" r="4.5"/><path d="m10.5 10.5 3.5 3.5"/>',
  rotate: '<path d="M13.5 8.5A5.5 5.5 0 1 1 11.8 4.3"/><path d="M12.5 1.5v3.4H9.1"/>',
  sharpen: '<path d="M8 1.5 9.6 6.4 14.5 8 9.6 9.6 8 14.5 6.4 9.6 1.5 8 6.4 6.4Z"/>',
  trim: '<path d="M4.5 1.5v10h10M1.5 4.5h10v10"/>',
  size: '<rect x="1.5" y="1.5" width="13" height="13" rx="1.5"/><rect x="4.5" y="4.5" width="7" height="7" rx=".5"/>',
  format: '<path d="M9.5 1.5H4a1 1 0 0 0-1 1v11a1 1 0 0 0 1 1h8a1 1 0 0 0 1-1V5Z"/><path d="M9.5 1.5V5H13"/>',
  quality: '<path d="M2 13.5h12"/><path d="M3.5 13.5v-3M6.5 13.5v-5M9.5 13.5v-7.5M12.5 13.5V3"/>',
  name: '<path d="M8.5 2.5h-5a1 1 0 0 0-1 1v5l6.3 6.3a1 1 0 0 0 1.4 0l4.1-4.1a1 1 0 0 0 0-1.4Z"/><circle cx="5.5" cy="5.5" r="1"/>',
  metadata: '<circle cx="8" cy="8" r="6.5"/><path d="M8 7.5v4M8 4.6v.1"/>',
  flipH: '<path d="M8 1.5v13"/><path d="M5.5 4 2 11.5h3.5ZM10.5 4l3.5 7.5h-3.5Z"/>',
  flipV: '<path d="M1.5 8h13"/><path d="M4 5.5 11.5 2v3.5ZM4 10.5l7.5 3.5v-3.5Z"/>',
  drop: '<path d="M8 1.5v8M4.5 6 8 9.5 11.5 6"/><path d="M2 10.5v2a1 1 0 0 0 1 1h10a1 1 0 0 0 1-1v-2"/>',
  stamp: '<path d="M5.5 9.5V8a2.5 2.5 0 1 1 5 0v1.5"/><path d="M6.5 6.5 6 3.5a2 2 0 0 1 4 0l-.5 3"/><rect x="2.5" y="9.5" width="11" height="3" rx="1"/><path d="M3.5 14.5h9"/>',
  list: '<path d="M6 4h8M6 8h8M6 12h8"/><path d="M2.5 4h.01M2.5 8h.01M2.5 12h.01"/>',
  split: '<rect x="1.5" y="2.5" width="13" height="11" rx="1.5"/><path d="M8 1v14"/>',
} as const;

export type IconName = keyof typeof ICONS;
