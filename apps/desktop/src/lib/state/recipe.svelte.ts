// SPDX-License-Identifier: AGPL-3.0-or-later
// The recipe being edited. Every control reads and writes `recipe.value`; `edited` compares it
// with the last loaded or saved copy.

import { backend } from '$lib/ipc';
import type { AddOn, Recipe, RecipeEntry } from '$lib/types';

const LAST = 'wrangler.recipe';

class RecipeState {
  entries = $state<RecipeEntry[]>([]);
  /** Id of the saved or built-in recipe this started from. */
  id = $state('web-ready');
  value = $state<Recipe | null>(null);
  #saved = $state('');

  edited = $derived(this.value != null && JSON.stringify($state.snapshot(this.value)) !== this.#saved);
  entry = $derived(this.entries.find(e => e.id === this.id));
  index = $derived(this.entries.findIndex(e => e.id === this.id));

  async init() {
    this.entries = await backend.recipes();
    const last = localStorage.getItem(LAST);
    await this.load(last && this.entries.some(e => e.id === last) ? last : 'web-ready');
  }

  async load(id: string) {
    const r = await backend.loadRecipe(id);
    this.value = r;
    this.#saved = JSON.stringify(r);
    this.id = id;
    localStorage.setItem(LAST, id);
  }

  step(by: 1 | -1) {
    if (!this.entries.length) return;
    const i = (this.index + by + this.entries.length) % this.entries.length;
    return this.load(this.entries[i].id);
  }

  /** Save under the current name, or a new one. Built-ins are saved as a copy. */
  async save(name?: string) {
    if (!this.value) return;
    if (name) this.value.name = name;
    const id = await backend.saveRecipe(this.snapshot());
    this.entries = await backend.recipes();
    this.id = id;
    this.#saved = JSON.stringify(this.snapshot());
    localStorage.setItem(LAST, id);
    return id;
  }

  async remove(id: string) {
    await backend.deleteRecipe(id);
    this.entries = await backend.recipes();
    if (id === this.id) await this.load(this.entries.find(e => e.id === id)?.id ?? 'web-ready');
  }

  revert() {
    return this.load(this.id);
  }

  snapshot(): Recipe {
    return $state.snapshot(this.value) as Recipe;
  }

  /** The add-on of a kind in this recipe, if present. */
  addon<K extends AddOn['kind']>(kind: K): Extract<AddOn, { kind: K }> | undefined {
    return this.value?.addons.find(a => a.kind === kind) as Extract<AddOn, { kind: K }> | undefined;
  }
}

export const recipe = new RecipeState();
