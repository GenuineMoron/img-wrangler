// SPDX-License-Identifier: AGPL-3.0-or-later
// Format: keep each file's own, convert everything to one, or pick per image (auto).

import { catalog } from '$lib/state/catalog.svelte';
import type { CoreModule } from '../types';
import Screen from './FormatScreen.svelte';
import Keys from './FormatKeys.svelte';

const format: CoreModule = {
  kind: 'core',
  id: 'format',
  label: 'Format',
  blurb: 'Keep each file’s format, convert to one, or choose per image.',
  icon: 'format',
  accent: 'format',
  order: 60,
  active: r => r.format.id !== 'same' || r.format.auto,
  summary: r => (r.format.auto ? 'auto per image' : r.format.id === 'same' ? 'unchanged' : (catalog.info(r.format.id)?.label ?? r.format.id)),
  channel: { slot: 2, glyph: '●', Screen, Keys },
};

export default format;
