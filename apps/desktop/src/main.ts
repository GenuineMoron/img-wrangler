// SPDX-License-Identifier: AGPL-3.0-or-later
import '@design/fonts.css';
import '@design/tokens.css';
import '@design/components.css';
import './app.css';
import { mount } from 'svelte';
import { backend } from '$lib/ipc';
import App from './App.svelte';

// In the app, the webview's own right-click menu (Reload, Inspect) has no place: Wrangler shows
// its own where there's something to offer. Text fields keep theirs, for copy and paste.
if (backend.kind === 'tauri') {
  addEventListener('contextmenu', e => {
    if (!(e.target instanceof Element && e.target.closest('input, textarea, [contenteditable]'))) e.preventDefault();
  });
}

mount(App, { target: document.getElementById('app')! });
