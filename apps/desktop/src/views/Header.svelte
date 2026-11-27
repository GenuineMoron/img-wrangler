<!-- SPDX-License-Identifier: AGPL-3.0-or-later
  The top of the unit: the mark, the recipe selector, and the keys for adding images, past runs
  and settings. The empty parts drag the window. -->
<script lang="ts">
  import { tick } from 'svelte';
  import Key from '$lib/components/Key.svelte';
  import HatScreen from '$lib/components/HatScreen.svelte';
  import Screen from '$lib/components/Screen.svelte';
  import { backend } from '$lib/ipc';
  import { batch } from '$lib/state/batch.svelte';
  import { recipe } from '$lib/state/recipe.svelte';
  import { ui } from '$lib/state/ui.svelte';
  import { updates } from '$lib/state/updates.svelte';
  import Led from '$lib/components/Led.svelte';

  const mod = backend.platform === 'mac' ? '⌘' : 'Ctrl+';

  async function add(kind: 'files' | 'folder') {
    const paths = await backend.pick(kind);
    if (paths.length) await batch.add(paths);
  }

  /** Images or a whole folder: the system's picker takes one kind at a time, so ask which. */
  function addMenu() {
    backend.contextMenu([
      { text: 'Add images…', action: () => add('files') },
      { text: 'Add a folder…', action: () => add('folder') },
    ]);
  }

  async function save() {
    await recipe.save();
    ui.say(`Saved “${recipe.value?.name}”.`);
  }

  // Saving as a new recipe: its name is typed on the recipe screen, where the name lives.
  let newName = $state('');
  $effect(() => { if (ui.naming) newName = `${recipe.value?.name ?? 'Recipe'} copy`; });

  async function saveNew() {
    const name = newName.trim();
    if (!name) return;
    ui.naming = false;
    await recipe.saveAs(name);
    ui.say(`Saved “${recipe.value?.name}” as a new recipe.`);
  }

  // Selected once the suggested name is in, so typing replaces it.
  const focus = (el: HTMLInputElement) => { el.focus(); tick().then(() => el.select()); };
</script>

<header class="head" data-tauri-drag-region>
  <HatScreen />
  <div class="word" data-tauri-drag-region>wrangler <span class="legend legend--quiet">batch image unit</span></div>

  <div class="recipe">
    <Key icon="prev" aria-label="Previous recipe" onclick={() => recipe.step(-1)} />
    <Screen class="recipe__screen">
      {#if ui.naming}
        <input class="recipe__new" aria-label="Name for the new recipe" spellcheck="false" autocomplete="off" bind:value={newName} use:focus
          onkeydown={e => { if (e.key === 'Enter') saveNew(); if (e.key === 'Escape') ui.naming = false; }} />
      {:else}
        <button type="button" class="recipe__name" onclick={() => (ui.dialog = 'recipes')} title="Choose a recipe, or look after them"
          aria-label="Recipe: {recipe.value?.name}{recipe.edited ? ', with changes not saved' : ''}. Press to choose a recipe or look after them">
          <span class="num">{recipe.index < 0 ? '--' : String(recipe.index + 1).padStart(2, '0')}</span>
          <span class="label" data-long>{recipe.value?.name.toUpperCase() ?? '--'}</span>
          {#if recipe.edited}<span class="edited">● edited</span>{/if}
        </button>
      {/if}
    </Screen>
    <Key icon="next" aria-label="Next recipe" onclick={() => recipe.step(1)} />
    {#if ui.naming}
      <Key dark label="Save" disabled={!newName.trim()} onclick={saveNew} />
      <Key label="Cancel" onclick={() => (ui.naming = false)} />
    {:else}
      <Key label="Save" disabled={!recipe.edited} onclick={save} title="Save your changes to this recipe ({mod}S)" />
      <Key insert label="Save as new" onclick={() => (ui.naming = true)} title="Leave this recipe as it was and save what’s on the controls as a new one ({mod}⇧S)" />
      {#if recipe.edited}<Key icon="undo" aria-label="Undo your changes to this recipe" title="Undo your changes to this recipe" onclick={() => recipe.revert()} />{/if}
    {/if}
  </div>

  <div class="right">
    {#if updates.found}
      <button type="button" class="update" onclick={() => (ui.dialog = 'update')} title="Wrangler {updates.found.version} is out">
        <Led on /> <span>Update</span>
      </button>
    {/if}
    <Key dark icon="plus" label="Add" onclick={addMenu} title="Add images ({mod}O) or a folder ({mod}⇧O). Or drop them anywhere" />
    <Key dark icon="history" aria-label="Past runs" title="Past runs and undo" onclick={() => (ui.dialog = 'history')} />
    <Key dark icon="settings" aria-label="Settings" title="Settings ({mod},)" onclick={() => (ui.dialog = 'settings')} />
  </div>
</header>

<style>
  .head { grid-column: 1 / -1; display: flex; align-items: center; gap: var(--sp-8); }
  .word { font: 700 22px/1 var(--font-ui); letter-spacing: -0.04em; color: #18191b; display: flex; align-items: baseline; gap: var(--sp-4); white-space: nowrap; }
  .recipe { display: flex; align-items: center; gap: var(--sp-3); margin-left: var(--sp-8); }
  .recipe :global(.recipe__screen) { width: 290px; height: var(--key-h); }
  .recipe__name { all: unset; box-sizing: border-box; cursor: pointer; width: 100%; height: 100%; display: flex; align-items: center; gap: var(--sp-6); padding: 0 var(--sp-6);
    font: 12.5px/1 var(--font-data); white-space: nowrap; overflow: hidden; }
  .recipe__name:focus-visible { outline: var(--focus-on-screen); outline-offset: -3px; }
  .recipe__new { all: unset; box-sizing: border-box; width: 100%; height: 100%; padding: 0 var(--sp-6); font: 12.5px/1 var(--font-data); color: var(--screen-1); box-shadow: inset 0 0 0 1px var(--output); border-radius: var(--r-screen); }
  .num { color: var(--output); }
  .label { overflow: hidden; text-overflow: ellipsis; }
  .edited { margin-left: auto; color: var(--output); font-size: var(--fs-small); }
  .right { margin-left: auto; display: flex; align-items: center; gap: var(--sp-4); }
  .update { all: unset; cursor: pointer; display: flex; align-items: center; gap: 6px; padding: 0 var(--sp-4); height: var(--key-h); border-radius: var(--r-key);
    font: 600 var(--fs-key)/1 var(--font-ui); letter-spacing: .05em; text-transform: uppercase; color: var(--ink-1); }
  .update:hover { color: var(--output); }
  .update:focus-visible { outline: var(--focus); outline-offset: var(--focus-offset); }
  @media (max-width: 1320px) { .word .legend { display: none; } .recipe :global(.recipe__screen) { width: 230px; } }
</style>
