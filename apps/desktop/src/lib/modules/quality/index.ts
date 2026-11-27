// SPDX-License-Identifier: AGPL-3.0-or-later
// Quality: a look that aims at a measured visual score, an optional size limit, lossless.

import type { Recipe } from '$lib/types';
import type { CoreModule } from '../types';
import Screen from './QualityScreen.svelte';
import Keys from './QualityKeys.svelte';
import Details from './QualityDetails.svelte';

/** The looks, the SSIMULACRA 2 score each aims for, and a typical 2048 px JPG at that look. */
export const LOOKS = [
  { id: 'small', label: 'Small', score: 65, kb: 243 },
  { id: 'balanced', label: 'Balanced', score: 75, kb: 411 },
  { id: 'high', label: 'High', score: 83, kb: 650 },
  { id: 'pristine', label: 'Pristine', score: 90, kb: 871 },
] as const;

/** A size limit as people say it: 400 KB, 1.5 MB, 20 MB. */
export function kbText(kb: number): string {
  if (kb < 1000) return `${kb} KB`;
  const mb = kb / 1000;
  return `${mb < 10 ? Number(mb.toFixed(2)) : Number(mb.toFixed(1))} MB`;
}

/** A typed size limit in KB: "400", "400 kb", "1.5 MB", "2m". A bare number is KB. Null when
 *  it isn't a size, or is under 10 KB. */
export function parseKb(text: string): number | null {
  const m = text.trim().toLowerCase().replace(',', '.').match(/^(\d+(?:\.\d+)?)\s*(k|kb|m|mb|g|gb)?$/);
  if (!m) return null;
  const n = Number(m[1]) * (m[2]?.startsWith('g') ? 1e6 : m[2]?.startsWith('m') ? 1000 : 1);
  return n >= 10 && n <= 1e6 ? Math.round(n) : null;
}

/** A limit rounded to a step that suits its size, for dragging. */
export function roundKb(kb: number): number {
  const step = kb < 1000 ? 10 : kb < 10000 ? 100 : 1000;
  return Math.max(10, Math.round(kb / step) * step);
}

export function lookText(r: Recipe): string {
  const l = r.quality.look;
  return typeof l === 'number' ? `quality ${l}` : l;
}

const quality: CoreModule = {
  kind: 'core',
  id: 'quality',
  label: 'Quality',
  blurb: 'How hard to compress, a size limit, and never making files bigger.',
  icon: 'quality',
  accent: 'quality',
  order: 110,
  active: r => r.quality.look !== 'untouched',
  summary: r => `${lookText(r)}${r.quality.max_kb ? ` · ≤${kbText(r.quality.max_kb)}` : ''}`,
  Details,
  channel: { slot: 3, glyph: '▲', Screen, Keys },
};

export default quality;
