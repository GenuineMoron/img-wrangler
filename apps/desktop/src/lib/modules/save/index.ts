// SPDX-License-Identifier: AGPL-3.0-or-later
// Save: where results go and what happens when a name is taken. Quick keys live on the
// Output panel; this holds the rest.

import { fileName } from '$lib/text';
import type { CoreModule } from '../types';
import Details from './SaveDetails.svelte';

const save: CoreModule = {
  kind: 'core',
  id: 'save',
  label: 'Save',
  blurb: 'Where the results go: beside, a subfolder, a folder, or replacing originals.',
  icon: 'save',
  accent: 'tool',
  order: 140,
  active: () => true,
  summary: r => {
    const d = r.destination;
    return { beside: `beside, ${d.suffix || 'same name'}`, subfolder: `into ${d.folder || 'Wrangled'}/`, folder: `into ${fileName(d.folder || 'Wrangled')}`, replace: 'replaces originals' }[d.mode];
  },
  Details,
};

export default save;
