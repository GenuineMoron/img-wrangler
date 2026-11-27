// SPDX-License-Identifier: AGPL-3.0-or-later
// Name: keep names, build them from pieces, find and replace in them, or take them from a list.

import type { CoreModule } from '../types';
import { label, parse } from './pattern';
import Screen from './NameScreen.svelte';
import Keys from './NameKeys.svelte';
import Details from './NameDetails.svelte';

const name: CoreModule = {
  kind: 'core',
  id: 'name',
  label: 'Name',
  blurb: 'Build names from pieces and numbers, find and replace, or a list from a spreadsheet.',
  icon: 'name',
  accent: 'name',
  order: 130,
  active: r => r.rename.mode !== 'keep' || r.rename.case !== 'keep',
  summary: r => {
    const n = r.rename;
    if (n.mode === 'keep') return n.case === 'keep' ? 'unchanged' : n.case;
    if (n.mode === 'pattern') return parse(n.pattern).map(p => (p.kind === 'text' ? p.text : `[${label(p)}]`)).join('');
    if (n.mode === 'list') return 'from a list';
    const swaps = n.replace.filter(x => x.find).length;
    return swaps ? `${swaps} ${swaps === 1 ? 'replacement' : 'replacements'}` : 'nothing to find yet';
  },
  Details,
  channel: { slot: 4, glyph: '◆', Screen, Keys },
};

export default name;
