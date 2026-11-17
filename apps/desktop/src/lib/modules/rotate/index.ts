// SPDX-License-Identifier: AGPL-3.0-or-later
// Rotate: turn and flip, before sizing. Engine: modules/rotate.rs.

import type { AddOnModule } from '../types';
import Details from './RotateDetails.svelte';

const rotate: AddOnModule<'rotate'> = {
  kind: 'addon',
  addon: 'rotate',
  id: 'rotate',
  label: 'Rotate',
  blurb: 'Turn by 90° steps and flip.',
  icon: 'rotate',
  accent: 'tool',
  order: 20,
  create: () => ({ kind: 'rotate', enabled: true, turn: 90, flip_horizontal: false, flip_vertical: false }),
  active: r => r.addons.some(a => a.kind === 'rotate' && a.enabled && (a.turn % 360 !== 0 || a.flip_horizontal || a.flip_vertical)),
  summary: r => {
    const a = r.addons.find(x => x.kind === 'rotate');
    if (!a || a.kind !== 'rotate') return '';
    return [a.turn % 360 ? `${a.turn}°` : '', a.flip_horizontal ? 'flip ↔' : '', a.flip_vertical ? 'flip ↕' : ''].filter(Boolean).join(' · ') || 'nothing';
  },
  Details,
};

export default rotate;
