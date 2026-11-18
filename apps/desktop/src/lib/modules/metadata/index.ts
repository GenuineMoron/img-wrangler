// SPDX-License-Identifier: AGPL-3.0-or-later
// Metadata: what camera data travels with the image. Location is never kept by "safe".

import type { CoreModule } from '../types';
import Details from './MetadataDetails.svelte';

const metadata: CoreModule = {
  kind: 'core',
  id: 'metadata',
  label: 'Metadata',
  blurb: 'Keep or strip camera data. Location goes unless you keep everything.',
  icon: 'metadata',
  accent: 'tool',
  order: 120,
  active: () => true,
  summary: r => ({ none: 'all removed', safe: 'kept, no location', all: 'all kept' })[r.metadata.keep],
  Details,
};

export default metadata;
