// SPDX-License-Identifier: AGPL-3.0-or-later
// Size: one place for how big, what shape, and how the picture meets that shape (crop, pad on a
// backdrop, or fit inside), with trimming empty edges first. Engine: modules/size.rs.

import { parseShape } from '$lib/geometry';
import type { Edge, Recipe } from '$lib/types';
import type { CoreModule } from '../types';
import Screen from './SizeScreen.svelte';
import Keys from './SizeKeys.svelte';
import Details from './SizeDetails.svelte';

export const SHAPES = [
  { id: 'original', label: 'Same', aria: 'Same shape as each image' },
  { id: '1:1', label: '1:1', aria: 'Square, 1 by 1' },
  { id: '4:5', label: '4:5', aria: 'Portrait, 4 by 5' },
  { id: '3:2', label: '3:2', aria: 'Landscape, 3 by 2' },
  { id: '16:9', label: '16:9', aria: 'Wide, 16 by 9' },
];

/** The side the size measures when each image keeps its shape, as the screen says it. */
export const EDGES: { id: Edge; label: string; menu: string; short: string }[] = [
  { id: 'long', label: 'px long side', menu: 'Long side', short: '' },
  { id: 'short', label: 'px short side', menu: 'Short side', short: ' short side' },
  { id: 'width', label: 'px wide', menu: 'Width', short: ' wide' },
  { id: 'height', label: 'px high', menu: 'Height', short: ' high' },
];

/** Size stops on the ruler; null keeps the full size. */
export const STOPS: { px: number | null; label: string; aria?: string }[] = [
  { px: null, label: 'full', aria: 'Full size' }, { px: 1200, label: '1200' }, { px: 1600, label: '1600' },
  { px: 2048, label: '2048' }, { px: 2560, label: '2560' }, { px: 3840, label: '4k', aria: '3840 pixels' },
];

export const FITS = [
  { id: 'crop', label: 'crop', aria: 'Crop: fill the frame, cutting the edges' },
  { id: 'pad', label: 'pad', aria: 'Pad: whole image on a backdrop' },
  { id: 'inside', label: 'inside', aria: 'Inside: whole image, frame is the most it can be' },
] as const;

function summary(r: Recipe): string {
  const s = r.size;
  const shape = parseShape(s.shape) ?? { kind: 'original' as const };
  const parts: string[] = [];
  if (s.trim) parts.push('trim');
  const edge = shape.kind === 'original' ? (EDGES.find(e => e.id === s.edge)?.short ?? '') : '';
  parts.push(shape.kind === 'exact' ? `${shape.w}×${shape.h}` : s.long_side ? `${s.long_side}${edge}` : 'full size');
  if (shape.kind === 'ratio') parts.push(`${shape.a}:${shape.b}`);
  if (shape.kind !== 'original') parts.push(s.fit === 'pad' && s.pad.fill < 100 ? `pad ${Math.round(s.pad.fill)}%` : s.fit);
  return parts.join('·');
}

const size: CoreModule = {
  kind: 'core',
  id: 'size',
  label: 'Size',
  blurb: 'Resize, change shape, crop or pad onto a backdrop, trim empty edges.',
  icon: 'size',
  accent: 'size',
  order: 30,
  active: r => r.size.long_side != null || r.size.shape !== 'original' || r.size.trim != null,
  summary,
  Details,
  channel: { slot: 1, glyph: '■', Screen, Keys },
};

export default size;
