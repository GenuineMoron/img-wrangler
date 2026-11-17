// SPDX-License-Identifier: AGPL-3.0-or-later
// What the engine can do: formats. Loaded once.

import { backend } from '$lib/ipc';
import type { Format, FormatInfo } from '$lib/types';

class Catalog {
  formats = $state<FormatInfo[]>([]);

  async load() {
    this.formats = await backend.formats();
  }

  info(id: Format | string | null | undefined): FormatInfo | undefined {
    return this.formats.find(f => f.id === id || f.ext === id);
  }

  /** Formats Wrangler can write, primary ones first. */
  writable = $derived(this.formats.filter(f => f.encode));
}

export const catalog = new Catalog();
