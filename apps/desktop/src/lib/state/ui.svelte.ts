// SPDX-License-Identifier: AGPL-3.0-or-later
// Window state that isn't part of a recipe or a batch.

export type View = 'auto' | 'result' | 'original' | 'split';
export type DialogId = 'settings' | 'history' | 'recipes' | 'update' | null;

class UiState {
  view = $state<View>('auto');
  split = $state(50);
  /** The module whose further settings are open, above its control. One at a time. */
  more = $state<string | null>(null);
  dialog = $state<DialogId>(null);
  dropping = $state(false);
  /** A short message on the status line, read out by screen readers. */
  message = $state<{ text: string; alarm: boolean } | null>(null);
  #timer: ReturnType<typeof setTimeout> | undefined;

  say(text: string, alarm = false) {
    this.message = { text, alarm };
    clearTimeout(this.#timer);
    this.#timer = setTimeout(() => (this.message = null), alarm ? 9000 : 5000);
  }

  /** Open a module's further settings, or close them if they're open. */
  toggleMore(id: string) {
    this.more = this.more === id ? null : id;
  }
}

export const ui = new UiState();
