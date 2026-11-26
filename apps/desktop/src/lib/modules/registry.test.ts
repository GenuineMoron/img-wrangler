// SPDX-License-Identifier: AGPL-3.0-or-later
// The registry is the only list of modules, so it must stay consistent as features are added.

import { describe, expect, it } from 'vitest';
import RECIPES from '../../../dev/recipes.json';
import type { Recipe } from '$lib/types';
import { ADDONS, CHANNELS, KEYS, MODULES, addonOf, hasDetails } from './registry';

const shopify = RECIPES['shopify-product'] as unknown as Recipe;

describe('module registry', () => {
  it('has unique ids in run order', () => {
    const ids = MODULES.map(m => m.id);
    expect(new Set(ids).size).toBe(ids.length);
    const orders = MODULES.map(m => m.order);
    expect([...orders].sort((a, b) => a - b)).toEqual(orders);
  });

  it('puts the four everyday channels in slots 1-4', () => {
    expect(CHANNELS.map(c => [c.id, c.channel.slot])).toEqual([['size', 1], ['format', 2], ['quality', 3], ['name', 4]]);
  });

  it('creates add-ons of their own kind, switched on', () => {
    for (const m of ADDONS) {
      const a = m.create();
      expect(a.kind).toBe(m.addon);
      expect(a.enabled).toBe(true);
    }
  });

  it('gives every module one home: a screen, a step key, or the Output panel', () => {
    expect(KEYS.map(m => m.id)).toEqual(['files', 'rotate', 'sharpen', 'watermark', 'metadata']);
    const docked = MODULES.filter(m => m.kind === 'core' && m.dock).map(m => m.id);
    expect(docked).toEqual(['save']);
    expect([...CHANNELS.map(m => m.id), ...KEYS.map(m => m.id), ...docked].sort()).toEqual(MODULES.map(m => m.id).sort());
  });

  it('finds an add-on’s settings in a recipe, and knows which modules open', () => {
    const sharpen = ADDONS.find(m => m.id === 'sharpen')!;
    const r: Recipe = { ...shopify, addons: [sharpen.create()] };
    expect(addonOf(r, sharpen)?.kind).toBe('sharpen');
    expect(addonOf(r, ADDONS.find(m => m.id === 'rotate')!)).toBeUndefined();
    expect(MODULES.filter(m => !hasDetails(m)).map(m => m.id)).toEqual(['format']);
  });

  it('summarises every built-in recipe without throwing', () => {
    for (const r of Object.values(RECIPES) as unknown as Recipe[]) for (const m of MODULES) expect(typeof m.summary(r)).toBe('string');
  });
});
