// SPDX-License-Identifier: AGPL-3.0-or-later
// New versions: what the app found, and installing it.

import { backend } from '$lib/ipc';
import type { UpdateInfo } from '$lib/types';
import { ui } from './ui.svelte';

class UpdatesState {
  found = $state.raw<UpdateInfo | null>(null);
  checking = $state(false);
  /** A look someone asked for came back with nothing newer. */
  upToDate = $state(false);
  installing = $state(false);
  /** 0 to 1, or -1 when the size isn't known. */
  progress = $state(0);
  error = $state<string | null>(null);

  async init() {
    this.found = await backend.updateFound();
    return backend.onUpdate(u => {
      const fresh = u.version !== this.found?.version;
      this.found = u;
      if (fresh) ui.say(`Wrangler ${u.version} is out. Press Update to see what's new.`);
    });
  }

  async check() {
    this.checking = true;
    this.error = null;
    this.upToDate = false;
    try {
      this.found = await backend.updateCheck();
      this.upToDate = !this.found;
    } catch (e) {
      this.error = `Couldn't check for updates: ${e}`;
    } finally {
      this.checking = false;
    }
  }

  async install() {
    if (!this.found || this.found.manual) return;
    this.installing = true;
    this.error = null;
    this.progress = 0;
    try {
      // The app restarts into the new version when this finishes.
      await backend.updateInstall((done, total) => (this.progress = total ? done / total : -1));
    } catch (e) {
      this.error = `The update didn't install: ${e}`;
      this.installing = false;
    }
  }
}

export const updates = new UpdatesState();
