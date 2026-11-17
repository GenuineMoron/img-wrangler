// SPDX-License-Identifier: AGPL-3.0-or-later
// Window state that isn't part of a recipe or a batch.

export type View = 'auto' | 'result' | 'original' | 'split';
export type DialogId = 'palette' | 'settings' | 'history' | 'recipes' | null;

class UiState {
  view = $state<View>('auto');
  split = $state(50);
  /** The step open in the Recipe panel; one at a time keeps the list readable. */
  step = $state<string | null>(null);
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

  toggle(id: string, open?: boolean) {
    this.step = (open ?? this.step !== id) ? id : null;
  }

  /** Open a step's settings in the Recipe panel and move focus there. */
  reveal(id: string) {
    this.step = id;
    requestAnimationFrame(() => {
      const row = document.getElementById(`step-${id}`);
      row?.scrollIntoView({ block: 'nearest', behavior: 'smooth' });
      row?.focus({ preventScroll: true });
    });
  }
}

export const ui = new UiState();
