// SPDX-License-Identifier: AGPL-3.0-or-later
// The real backend: commands on the Rust host (apps/desktop/src-tauri/src/commands.rs).

import { Channel, convertFileSrc, invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { getCurrentWebview } from '@tauri-apps/api/webview';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { open } from '@tauri-apps/plugin-dialog';
import { revealItemInDir } from '@tauri-apps/plugin-opener';
import type { Preview, RunEvent } from '$lib/types';
import { IMAGE_EXTENSIONS, type Backend } from './backend';

type RawPreview = Omit<Preview, 'before' | 'after'> & { before: string; after: string };

function platform(): Backend['platform'] {
  const ua = navigator.userAgent;
  return /Mac/.test(ua) ? 'mac' : /Windows/.test(ua) ? 'windows' : 'linux';
}

export const tauri: Backend = {
  kind: 'tauri',
  platform: platform(),

  formats: () => invoke('formats'),
  recipes: () => invoke('recipes'),
  loadRecipe: id => invoke('load_recipe', { id }),
  saveRecipe: recipe => invoke('save_recipe', { recipe }),
  deleteRecipe: id => invoke('delete_recipe', { id }),

  scan: (paths, select, append) => invoke('scan', { paths, select, append }),
  forget: paths => invoke('forget', { paths }),
  plan: recipe => invoke('plan', { recipe }),
  estimate: (recipe, samples) => invoke('estimate', { recipe, samples: samples ?? null }),
  async preview(path, recipe) {
    const p = await invoke<RawPreview>('preview', { path, recipe });
    return { ...p, before: convertFileSrc(p.before, 'preview'), after: convertFileSrc(p.after, 'preview') };
  },

  run(recipe, onEvent: (e: RunEvent) => void) {
    const channel = new Channel<RunEvent>();
    channel.onmessage = onEvent;
    return invoke('run', { recipe, onEvent: channel });
  },
  quickRun(recipe, paths, onEvent: (e: RunEvent) => void) {
    const channel = new Channel<RunEvent>();
    channel.onmessage = onEvent;
    return invoke('quick_run', { recipe, paths, onEvent: channel });
  },
  openMain: paths => invoke('open_main', { paths }),
  cancel: () => invoke('cancel'),
  undo: id => invoke('undo', { id }),
  history: () => invoke('history'),

  takeLaunch: kind => invoke('take_launch', { kind }),
  onLaunch: cb => listen('launch://pending', () => cb()),
  onDrop: (drop, hover) =>
    getCurrentWebview().onDragDropEvent(e => {
      const p = e.payload;
      if (p.type === 'enter' || p.type === 'over') hover(true);
      else if (p.type === 'leave') hover(false);
      else if (p.type === 'drop') { hover(false); drop(p.paths); }
    }),
  async pick(kind) {
    const r = await open(kind === 'folder'
      ? { directory: true, multiple: true, title: 'Add folders' }
      : { multiple: true, title: 'Add images', filters: [{ name: 'Images', extensions: IMAGE_EXTENSIONS }] });
    return r == null ? [] : Array.isArray(r) ? r : [r];
  },
  async pickFolder() {
    const r = await open({ directory: true, multiple: false, title: 'Save into' });
    return typeof r === 'string' ? r : null;
  },
  reveal: path => revealItemInDir(path),
  thumbUrl: path => convertFileSrc(path, 'thumb'),

  integrations: () => invoke('integrations'),
  setIntegrations: (install, pins) => invoke('set_integrations', { install, pins }),

  ready: () => getCurrentWindow().show(),
  close: () => getCurrentWindow().close(),
};
