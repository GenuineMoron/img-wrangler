// SPDX-License-Identifier: AGPL-3.0-or-later
// Every module the app knows. Adding a feature: a folder under modules/ and one line here
// (plus its engine side, crates/engine/src/modules). Layout code only ever reads these lists.

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

/** In the order the steps run. */
export const MODULES: UiModule[] = [files, rotate, size, sharpen, watermark, format, quality, metadata, name, save].sort((a, b) => a.order - b.order);

/** The screens under the display. */
export const CHANNELS = MODULES.filter((m): m is CoreModule & { channel: NonNullable<CoreModule['channel']> } => m.kind === 'core' && !!m.channel)
  .sort((a, b) => a.channel.slot - b.channel.slot);

/** The row of step keys: every module that isn't a screen or docked on the Output panel. */
export const KEYS = MODULES.filter(m => m.kind === 'addon' || (!m.channel && !m.dock));

export const ADDONS = MODULES.filter((m): m is AddOnModule => m.kind === 'addon');

export const moduleById = (id: string) => MODULES.find(m => m.id === id);

/** An add-on's settings in a recipe, when it has been switched on at some point. */
export const addonOf = (r: Recipe | null | undefined, m: UiModule): AddOn | undefined =>
  m.kind === 'addon' ? r?.addons.find(a => a.kind === m.addon) : undefined;

/** Whether a module can be opened for more settings. */
export const hasDetails = (m: UiModule) => m.kind === 'addon' || !!m.Details;
