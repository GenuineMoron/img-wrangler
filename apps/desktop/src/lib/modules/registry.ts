// SPDX-License-Identifier: AGPL-3.0-or-later
// Every module the app knows. Adding a feature: a folder under modules/ and one line here
// (plus its engine side, crates/engine/src/modules). Layout code only ever reads this list.

import type { AddOn, Recipe } from '$lib/types';
import type { AddOnModule, CoreModule, UiModule } from './types';
import files from './files';
import rotate from './rotate';
import size from './size';
import sharpen from './sharpen';
import watermark from './watermark';
import format from './format';
import quality from './quality';
import metadata from './metadata';
import name from './name';
import save from './save';

export const MODULES: UiModule[] = [files, rotate, size, sharpen, watermark, format, quality, metadata, name, save].sort((a, b) => a.order - b.order);

export const CHANNELS = MODULES.filter((m): m is CoreModule & { channel: NonNullable<CoreModule['channel']> } => m.kind === 'core' && !!m.channel)
  .sort((a, b) => a.channel.slot - b.channel.slot);

export const ADDONS = MODULES.filter((m): m is AddOnModule => m.kind === 'addon');

export const moduleById = (id: string) => MODULES.find(m => m.id === id);

export interface Step {
  module: UiModule;
  /** For add-ons: the settings in the recipe. */
  addon?: AddOn;
}

/** The steps of a recipe in run order: every core module, plus the add-ons it holds. */
export function steps(r: Recipe): Step[] {
  return MODULES.flatMap((m): Step[] => {
    if (m.kind === 'core') return [{ module: m }];
    return r.addons.filter(a => a.kind === m.addon).map(addon => ({ module: m, addon }));
  });
}

/** Add-ons not yet in the recipe. */
export const addable = (r: Recipe) => ADDONS.filter(m => !r.addons.some(a => a.kind === m.addon));
