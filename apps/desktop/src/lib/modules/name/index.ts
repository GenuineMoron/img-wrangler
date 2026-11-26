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
  blurb: 'Rename with patterns and numbers, find and replace, or a list from a spreadsheet.',
  icon: 'name',
  accent: 'name',
  order: 130,
  active: r => r.rename.mode !== 'keep' || r.rename.case !== 'keep',
  summary: r => {
    const n = r.rename;
    if (n.mode === 'keep') return n.case === 'keep' ? 'unchanged' : n.case;
    if (n.mode === 'pattern') return n.pattern;
    if (n.mode === 'list') return 'from a list';
    const swaps = n.replace.filter(x => x.find).length;
    return swaps ? `${swaps} ${swaps === 1 ? 'replacement' : 'replacements'}` : 'nothing to find yet';
  },
  Details,
  channel: { slot: 4, glyph: '◆', Screen, Keys },
};

export default name;
