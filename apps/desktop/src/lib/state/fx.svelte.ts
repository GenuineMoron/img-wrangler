// SPDX-License-Identifier: AGPL-3.0-or-later
// Full or light effects (see "light effects" in design/system/components.css). Automatic checks
// once how fast this computer redraws the window, and again after an update, and goes light when
// drawing is slow: a webview without a graphics card redraws every soft shadow and glow itself.

export type FxMode = 'auto' | 'full' | 'light';

const MODE = 'wrangler.fx';
const PROBE = 'wrangler.fx-probe';
/** Redrawing everything slower than this, in ms, reads as drawing without a graphics card. */
const SLOW = 50;

const frame = () => new Promise<number>(r => requestAnimationFrame(r));

class FxState {
  mode = $state<FxMode>((localStorage.getItem(MODE) as FxMode) || 'auto');
  /** What the check found: true when drawing is slow. Null until known. */
  slow = $state<boolean | null>(null);
  light = $derived(this.mode === 'light' || (this.mode === 'auto' && this.slow === true));

  constructor() {
    const kept = JSON.parse(localStorage.getItem(PROBE) ?? 'null') as { version: string; slow: boolean } | null;
    if (kept?.version === __APP_VERSION__) this.slow = kept.slow;
    $effect.root(() => {
      $effect(() => {
        document.documentElement.classList.toggle('lite', this.light);
      });
    });
  }

  set(mode: FxMode) {
    this.mode = mode;
    localStorage.setItem(MODE, mode);
  }

  /** Redraw the whole window a few times and time it. Done once per version. */
  async check() {
    if (this.slow !== null) return;
    await frame();
    await frame();
    const root = document.documentElement;
    const css = getComputedStyle(root);
    // Every key's shadow, every glow and the case's light, nudged by an amount too small to see,
    // so everything costly redraws, as when a dialog opens or the recipe changes.
    const nudge = ['--key-depth', '--key-soft', '--bloom', '--led-glow'].map(name => [name, `${parseFloat(css.getPropertyValue(name)) + 0.01}px`]);
    const times: number[] = [];
    for (let i = 0; i < 4; i++) {
      for (const [name, value] of nudge) {
        if (i % 2) root.style.removeProperty(name);
        else root.style.setProperty(name, value);
      }
      const t0 = await frame();
      times.push((await frame()) - t0);
    }
    for (const [name] of nudge) root.style.removeProperty(name);
    times.sort((a, b) => a - b);
    this.slow = times[1] > SLOW;
    localStorage.setItem(PROBE, JSON.stringify({ version: __APP_VERSION__, slow: this.slow, ms: Math.round(times[1]) }));
  }
}

export const fx = new FxState();
