// SPDX-License-Identifier: AGPL-3.0-or-later
// Name: keep names, build them from a pattern, or find and replace in them.

import type { CoreModule } from '../types';
import Screen from './NameScreen.svelte';
import Keys from './NameKeys.svelte';
import Details from './NameDetails.svelte';

export const TOKENS = [
  { token: '{name}', means: 'original name' },
  { token: '{n:000}', means: 'number, padded' },
  { token: '{w}×{h}', means: 'output size' },
  { token: '{date}', means: 'file date' },
  { token: '{taken}', means: 'photo taken' },
  { token: '{folder}', means: 'folder name' },
  { token: '{name|slug}', means: 'web-safe name' },
];

const name: CoreModule = {
  kind: 'core',
  id: 'name',
  label: 'Name',
  blurb: 'Rename with patterns and numbers, find and replace, change case.',
  icon: 'name',
  accent: 'name',
  order: 90,
  active: r => r.rename.mode !== 'keep' || r.rename.case !== 'keep',
  summary: r => (r.rename.mode === 'keep' ? (r.rename.case === 'keep' ? 'unchanged' : r.rename.case) : r.rename.mode === 'pattern' ? r.rename.pattern : `${r.rename.replace.filter(x => x.find).length} replace`),
  Details,
  channel: { slot: 4, glyph: '◆', Screen, Keys },
};

export default name;
