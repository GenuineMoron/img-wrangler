// SPDX-License-Identifier: AGPL-3.0-or-later
// Keeps the plan, the estimate and the preview in step with the recipe and the batch.

import { untrack } from 'svelte';
import { backend } from '$lib/ipc';
import { batch } from './batch.svelte';
import { catalog } from './catalog.svelte';
import { recipe } from './recipe.svelte';
import { ui } from './ui.svelte';
import { updates } from './updates.svelte';

export async function start(): Promise<() => void> {
  await Promise.all([catalog.load(), recipe.init()]);

  // Only what changes pixels or bytes; renaming or moving the output doesn't need a new
  // estimate or preview.
  const pixels = () => {
    const r = recipe.value;
    return r ? JSON.stringify($state.snapshot([r.size, r.format, r.quality, r.metadata, r.addons])) : '';
  };

  const stop = $effect.root(() => {
    $effect(() => {
      const json = recipe.value ? JSON.stringify($state.snapshot(recipe.value)) : '';
      const v = batch.version;
      if (json || v) untrack(() => batch.schedulePlan());
    });
    $effect(() => {
      const key = pixels();
      const v = batch.version;
      if (key || v) untrack(() => batch.scheduleEstimate());
    });
    let lastSelect = '';
    $effect(() => {
      const select = recipe.value ? JSON.stringify($state.snapshot(recipe.value.select)) : '';
      if (lastSelect && select !== lastSelect) untrack(() => batch.rescan());
      lastSelect = select;
    });
    let lastPath: string | null = null;
    $effect(() => {
      const path = batch.currentPath;
      const key = pixels();
      if (!path || !key) return;
      const moved = path !== lastPath;
      lastPath = path;
      untrack(() => batch.schedulePreview(moved));
    });
  });

  const takeLaunch = async () => {
    const l = await backend.takeLaunch('open');
    if (l?.mode === 'open') await batch.add(l.paths);
  };
  const offs = await Promise.all([
    backend.onLaunch(takeLaunch),
    backend.onDrop(paths => batch.add(paths), over => (ui.dropping = over)),
    updates.init(),
    // From the menu bar icon's menu.
    backend.onMenu(what => {
      if (what === 'settings') ui.dialog = 'settings';
      if (what === 'updates') { ui.dialog = 'update'; if (!updates.found) updates.check(); }
    }),
  ]);
  await takeLaunch();
  return () => { stop(); offs.forEach(off => off()); };
}
