// SPDX-License-Identifier: AGPL-3.0-or-later
// Sharpen: an unsharp mask after resizing, to restore crispness. Engine: modules/sharpen.rs.

import type { AddOnModule } from '../types';
import Details from './SharpenDetails.svelte';

const sharpen: AddOnModule<'sharpen'> = {
  kind: 'addon',
  addon: 'sharpen',
  id: 'sharpen',
  label: 'Sharpen',
  blurb: 'Restore crispness after making images smaller.',
  icon: 'sharpen',
  accent: 'tool',
  order: 50,
  create: () => ({ kind: 'sharpen', enabled: true, amount: 40, radius: 0.6, threshold: 2 }),
  active: r => r.addons.some(a => a.kind === 'sharpen' && a.enabled && a.amount > 0),
  summary: r => {
    const a = r.addons.find(x => x.kind === 'sharpen');
    return a && a.kind === 'sharpen' ? `${a.amount}% · ${a.radius} px` : '';
  },
  Details,
};

export default sharpen;
