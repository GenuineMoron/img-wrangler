// SPDX-License-Identifier: AGPL-3.0-or-later
// Picks the backend: the Tauri host when running in the app, the mock in a dev browser.

import type { Backend } from './backend';

const inTauri = typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;

export const backend: Backend = inTauri
  ? (await import('./tauri')).tauri
  : import.meta.env.DEV
    ? (await import('./mock')).mock
    : (() => { throw new Error('Wrangler needs its desktop host to run.'); })();

export type { Backend } from './backend';
