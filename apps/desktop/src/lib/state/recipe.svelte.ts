// SPDX-License-Identifier: AGPL-3.0-or-later
// The recipes, and the one being worked on. Every control reads and writes `recipe.value`.
// `edited` compares it with the copy last loaded or saved. Changes that aren't saved yet stay
// with their recipe while you look at another, so stepping through recipes loses nothing.

import { backend } from '$lib/ipc';
import type { AddOn, Recipe, RecipeEntry } from '$lib/types';

const LAST = 'wrangler.recipe';

class RecipeState {
  /** Every recipe, on or off, in the order they're kept. */
  entries = $state.raw<RecipeEntry[]>([]);
  /** Id of the recipe being worked on. */
  id = $state('');
  value = $state<Recipe | null>(null);
  #saved = $state('');
  /** Recipes left with changes that aren't saved: the changed recipe and the saved copy. */
  #drafts = $state.raw(new Map<string, { value: Recipe; saved: string }>());

  edited = $derived(this.value != null && JSON.stringify($state.snapshot(this.value)) !== this.#saved);
  entry = $derived(this.entries.find(e => e.id === this.id));
  /** The recipes that are switched on: what the header steps through and the menus offer. */
  shown = $derived(this.entries.filter(e => e.on));
  index = $derived(this.shown.findIndex(e => e.id === this.id));

  /** Whether a recipe has changes waiting to be saved. */
  unsaved(id: string) {
    return id === this.id ? this.edited : this.#drafts.has(id);
  }

  async refresh() {
    this.entries = await backend.recipes();
  }

  async init() {
    await this.refresh();
    const last = localStorage.getItem(LAST);
    const start = this.entries.find(e => e.id === last) ?? this.shown[0] ?? this.entries[0];
    await this.load(start?.id ?? 'new');
  }

  #drafted(change: (drafts: Map<string, { value: Recipe; saved: string }>) => void) {
    const next = new Map(this.#drafts);
    change(next);
    this.#drafts = next;
  }

  async load(id: string) {
    if (id === this.id && this.value) return;
    if (this.value && this.edited) this.#drafted(d => d.set(this.id, { value: this.snapshot(), saved: this.#saved }));
    const draft = this.#drafts.get(id);
    if (draft) {
      this.#drafted(d => d.delete(id));
      this.value = draft.value;
      this.#saved = draft.saved;
    } else {
      const r = await backend.loadRecipe(id);
      this.value = r;
      this.#saved = JSON.stringify(r);
    }
    this.id = id;
    localStorage.setItem(LAST, id);
  }

  step(by: 1 | -1) {
    const list = this.shown.length ? this.shown : this.entries;
    if (!list.length) return;
    const at = list.findIndex(e => e.id === this.id);
    return this.load(list[at < 0 ? 0 : (at + by + list.length) % list.length].id);
  }

  #settle(id: string) {
    this.id = id;
    this.#saved = JSON.stringify(this.snapshot());
    localStorage.setItem(LAST, id);
  }

  /** Save the changes to this recipe. */
  async save() {
    if (!this.value) return;
    const id = await backend.saveRecipe(this.snapshot());
    await this.refresh();
    this.#settle(id);
  }

  /** Save what's on the controls as a new recipe, and put the one it came from back as it was. */
  async saveAs(name: string) {
    if (!this.value) return;
    const id = await backend.saveRecipeAs({ ...this.snapshot(), name }, this.id);
    await this.refresh();
    // The name may have been given a number to keep it apart from another recipe.
    this.value.name = this.entries.find(e => e.id === id)?.name ?? name;
    this.#settle(id);
    return id;
  }

  /** A new recipe with nothing set, ready to name. */
  async create() {
    const blank = await backend.loadRecipe('new');
    const id = await backend.saveRecipeAs({ ...blank, name: 'New recipe' }, this.id);
    await this.refresh();
    await this.load(id);
    return id;
  }

  async rename(id: string, name: string) {
    const to = await backend.renameRecipe(id, name);
    await this.refresh();
    const called = this.entries.find(e => e.id === to)?.name ?? name;
    if (id === this.id && this.value) {
      const saved = { ...JSON.parse(this.#saved), name: called };
      this.value.name = called;
      this.id = to;
      this.#saved = JSON.stringify(saved);
      localStorage.setItem(LAST, to);
    } else if (this.#drafts.has(id)) {
      this.#drafted(d => { const draft = d.get(id)!; d.delete(id); d.set(to, { value: { ...draft.value, name: called }, saved: JSON.stringify({ ...JSON.parse(draft.saved), name: called }) }); });
    }
    return to;
  }

  async duplicate(id: string) {
    const copy = await backend.duplicateRecipe(id);
    await this.refresh();
    return copy;
  }

  async remove(id: string) {
    await backend.deleteRecipe(id);
    this.#drafted(d => d.delete(id));
    await this.refresh();
    if (id === this.id) {
      this.value = null;
      await this.load((this.shown[0] ?? this.entries[0])?.id ?? 'new');
    }
  }

  /** Throw away the changes that aren't saved. */
  async revert() {
    const id = this.id;
    this.value = null;
    await this.load(id);
  }

  /** Put a changed starter recipe back as it came. */
  async reset(id: string) {
    await backend.resetRecipe(id);
    this.#drafted(d => d.delete(id));
    await this.refresh();
    if (id === this.id) await this.revert();
  }

  /** Bring back deleted starters. Says how many came back. */
  async restore() {
    const before = this.entries.length;
    await backend.restoreRecipes();
    await this.refresh();
    return this.entries.length - before;
  }

  async setOn(id: string, on: boolean) {
    await backend.setRecipeOn(id, on);
    await this.refresh();
  }

  async reorder(ids: string[]) {
    await backend.setRecipeOrder(ids);
    await this.refresh();
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
