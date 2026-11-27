// SPDX-License-Identifier: AGPL-3.0-or-later
// What a recipe does, in a few words per step, read from its settings. A recipe list shows
// this, so what it says is always what the recipe will do.

import type { Recipe } from '$lib/types';
import { addonOf, MODULES } from './registry';
import type { Accent } from './types';

export interface Bit { id: string; label: string; accent: Accent; text: string }

/** A summary with room to breathe, and a bare size given its unit. */
const say = (id: string, text: string) => {
  const spaced = text.replace(/\s*·\s*/g, ' · ');
  return id === 'size' ? spaced.replace(/(^| · )(\d+)(?=( short side| wide| high)? · |( short side| wide| high)?$)/, '$1$2 px') : spaced;
};

/** The steps worth saying, in the order they run. Size, format, quality and where it saves always
 *  show. The rest show when they do something out of the ordinary. */
export function describe(r: Recipe): Bit[] {
  const always = ['size', 'format', 'quality', 'save'];
  const worth = (id: string) => (id === 'metadata' ? r.metadata.keep !== 'none' : id === 'files' ? r.select.include.length > 0 || r.select.exclude.length > 0 || r.select.min_kb != null : true);
  return MODULES
    .filter(m => (m.kind === 'addon' ? !!addonOf(r, m)?.enabled : always.includes(m.id) || (m.active(r) && worth(m.id))))
    .map(m => ({ id: m.id, label: m.label, accent: m.accent, text: say(m.id, m.summary(r)) }));
}
