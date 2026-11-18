// SPDX-License-Identifier: AGPL-3.0-or-later
// The real backend: commands on the Rust host (apps/desktop/src-tauri/src/commands.rs).

import { Channel, convertFileSrc, invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { getCurrentWebview } from '@tauri-apps/api/webview';
import { getCurrentWindow, LogicalSize } from '@tauri-apps/api/window';
import { open } from '@tauri-apps/plugin-dialog';
import { openUrl, revealItemInDir } from '@tauri-apps/plugin-opener';
import { CheckMenuItem, Menu, MenuItem, PredefinedMenuItem } from '@tauri-apps/api/menu';
import type { Preview, RunEvent, UpdateInfo } from '$lib/types';
import { IMAGE_EXTENSIONS, type Backend } from './backend';

const shown = (token: string) => convertFileSrc(token, 'preview');

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
  async preview(path, recipe, exact) {
    const p = await invoke<Preview>('preview', { path, recipe, exact });
    return { ...p, before: shown(p.before), before_full: shown(p.before_full), after: shown(p.after) };
  },
  previewFocus: (paths, recipe) => invoke('preview_focus', { paths, recipe }),

  run(recipe, onEvent: (e: RunEvent) => void, only) {
    const channel = new Channel<RunEvent>();
    channel.onmessage = onEvent;
    return invoke('run', { recipe, only: only ?? null, onEvent: channel });
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
      const at = 'position' in p ? { x: p.position.x / devicePixelRatio, y: p.position.y / devicePixelRatio } : null;
      if (p.type === 'enter' || p.type === 'over') hover(true, at);
      else if (p.type === 'leave') hover(false, null);
      else if (p.type === 'drop') { hover(false, null); drop(p.paths, at); }
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
  async pickImage() {
    const r = await open({ multiple: false, title: 'Choose a logo', filters: [{ name: 'Images', extensions: IMAGE_EXTENSIONS }] });
    return typeof r === 'string' ? r : null;
  },
  reveal: path => revealItemInDir(path),
  openImage: path => invoke('open_image', { path }),
  // Thumbnails as big as the sheet shows them on this display: less to scale on every repaint.
  thumbUrl: path => `${convertFileSrc(path, 'thumb')}?size=${devicePixelRatio > 1.25 ? 384 : 256}`,

  integrations: () => invoke('integrations'),
  setIntegrations: (install, pins) => invoke('set_integrations', { install, pins }),

  appSettings: () => invoke('app_settings'),
  setAppSettings: s => invoke('set_app_settings', s),
  setDropRecipe: id => invoke('set_drop_recipe', { id }),
  async onMenu(cb) {
    const offs = await Promise.all(['settings', 'updates'].map(what => listen(`menu://${what}`, () => cb(what))));
    return () => offs.forEach(off => off());
  },

  updateCheck: () => invoke('update_check'),
  updateFound: () => invoke('update_found'),
  onUpdate: cb => listen<UpdateInfo>('update://available', e => cb(e.payload)),
  updateInstall(progress) {
    const channel = new Channel<{ done: number; total: number | null }>();
    channel.onmessage = p => progress(p.done, p.total);
    return invoke('update_install', { onProgress: channel });
  },
  openUrl: url => openUrl(url),

  async contextMenu(lines) {
    const items = await Promise.all(lines.map(l =>
      !l ? PredefinedMenuItem.new({ item: 'Separator' })
        : l.checked !== undefined ? CheckMenuItem.new({ text: l.text, checked: l.checked, enabled: l.enabled ?? true, action: l.action })
        : MenuItem.new({ text: l.text, enabled: l.enabled ?? true, action: l.action })));
    await (await Menu.new({ items })).popup();
  },

  ready: () => invoke('window_ready'),
  close: () => getCurrentWindow().close(),

  selftest: () => invoke('selftest_config'),
  selftestDone: report => invoke('selftest_done', { report }),
  async setWindowSize(w, h) {
    await getCurrentWindow().setSize(new LogicalSize(w, h));
    return true;
  },
};
