// SPDX-License-Identifier: AGPL-3.0-or-later
// Watermark: your logo in the same place on every image. Engine: modules/watermark.rs.

import { fileName } from '$lib/text';
import type { AddOnModule } from '../types';
import Details from './WatermarkDetails.svelte';

const watermark: AddOnModule<'watermark'> = {
  kind: 'addon',
  addon: 'watermark',
  id: 'watermark',
  label: 'Watermark',
  blurb: 'Your logo in the same corner of every image.',
  icon: 'stamp',
  accent: 'tool',
  order: 60,
  create: () => ({ kind: 'watermark', enabled: true, image: null, anchor: 'bottom_right', size: 20, opacity: 70, margin: 3 }),
  active: r => r.addons.some(a => a.kind === 'watermark' && a.enabled && !!a.image),
  summary: r => {
    const a = r.addons.find(x => x.kind === 'watermark');
    if (!a || a.kind !== 'watermark') return '';
    return a.image ? `${fileName(a.image)} · ${Math.round(a.size)}%` : 'choose a logo';
  },
  Details,
};

export default watermark;
