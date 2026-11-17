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
  order: 70,
  active: r => r.quality.look !== 'untouched',
  summary: r => `${lookText(r)}${r.quality.max_kb ? ` · ≤${r.quality.max_kb} KB` : ''}`,
  Details,
  channel: { slot: 3, glyph: '▲', Screen, Keys },
};

export default quality;
