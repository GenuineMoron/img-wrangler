// SPDX-License-Identifier: AGPL-3.0-or-later
// What a module is, to the UI. A module is one step of a recipe (Size, Format, Rotate…).
// Core modules are always in a recipe; add-ons are added and removed. See docs/MODULES.md.

import type { Component } from 'svelte';
import type { IconName } from '$lib/components/icons';
import type { AddOn, Recipe } from '$lib/types';

/** Colour family from tokens.css (--accent-<name>). Graphite "tool" for utility steps. */
export type Accent = 'size' | 'format' | 'quality' | 'name' | 'trim' | 'canvas' | 'tool';

interface Base {
  id: string;
  label: string;
  /** One line on what it does: the Add menu and tooltips. */
  blurb: string;
  icon: IconName;
  accent: Accent;
  /** Place in the run, so the panel reads top to bottom. Pixel steps use the engine's stage
   *  order (20 to 99); encoding and saving steps come after, from 100. */
  order: number;
  /** Whether these settings change anything. */
  active(r: Recipe): boolean;
  /** A few words for the panel row and the channel header. */
  summary(r: Recipe): string;
}

export interface CoreModule extends Base {
  kind: 'core';
  /** Settings beyond the channel, shown when its row in the Recipe panel opens. */
  Details?: Component;
  /** A channel in the strip under the display. `slot` 1-4 is also its number key. */
  channel?: { slot: number; glyph: string; Screen: Component; Keys: Component };
}

export interface AddOnModule<K extends AddOn['kind'] = AddOn['kind']> extends Base {
  kind: 'addon';
  addon: K;
  /** The add-on as first added: enabled, with defaults. */
  create(): Extract<AddOn, { kind: K }>;
  /** Gets the add-on's settings object (`value`) to edit in place. */
  // eslint-disable-next-line @typescript-eslint/no-explicit-any -- each add-on's own props; checked in its folder
  Details: Component<{ value: any }>;
}

export type UiModule = CoreModule | AddOnModule;
