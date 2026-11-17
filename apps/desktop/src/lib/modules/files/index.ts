// SPDX-License-Identifier: AGPL-3.0-or-later
// Files: which images in the dropped files and folders the recipe takes.

import type { CoreModule } from '../types';
import Details from './FilesDetails.svelte';

const files: CoreModule = {
  kind: 'core',
  id: 'files',
  label: 'Files',
  blurb: 'Which images to take: subfolders, name patterns, a minimum size.',
  icon: 'folder',
  accent: 'tool',
  order: 0,
  active: r => r.select.include.length > 0 || r.select.exclude.length > 0 || !!r.select.min_kb,
  summary: r => [r.select.recursive ? 'with subfolders' : 'top level only', r.select.include.length ? r.select.include.join(' ') : '', r.select.min_kb ? `≥${r.select.min_kb} KB` : ''].filter(Boolean).join(' · '),
  Details,
};

export default files;
