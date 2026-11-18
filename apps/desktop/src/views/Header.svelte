<!-- SPDX-License-Identifier: AGPL-3.0-or-later
  The top of the unit: the hat on its little screen is the Add button, the recipe
  selector, and the utility keys. The empty parts drag the window. -->
<script lang="ts">
  import Key from '$lib/components/Key.svelte';
  import HatScreen from '$lib/components/HatScreen.svelte';
  import Screen from '$lib/components/Screen.svelte';
  import { backend } from '$lib/ipc';
  import { batch } from '$lib/state/batch.svelte';
  import { recipe } from '$lib/state/recipe.svelte';
  import { ui } from '$lib/state/ui.svelte';

  const mod = backend.platform === 'mac' ? '⌘' : 'Ctrl+';

  async function add(kind: 'files' | 'folder') {
    const paths = await backend.pick(kind);
    if (paths.length) await batch.add(paths);
  }

  async function save() {
    if (recipe.entry?.built_in) ui.dialog = 'recipes';
    else { await recipe.save(); ui.say(`Saved “${recipe.value?.name}”.`); }
  }
</script>

<header class="head" data-tauri-drag-region>
  <button type="button" class="pal" aria-label="Add images. Drop them anywhere, or press to choose" title="Add images ({mod}O)" onclick={() => add('files')}>
    <HatScreen />
  </button>
  <div class="word" data-tauri-drag-region>wrangler <span class="legend legend--quiet">batch image unit</span></div>

  <div class="recipe">
    <Key icon="prev" aria-label="Previous recipe" onclick={() => recipe.step(-1)} />
    <Screen class="recipe__screen">
      <button type="button" class="recipe__name" onclick={() => (ui.dialog = 'recipes')} aria-label="Recipe: {recipe.value?.name}{recipe.edited ? ', edited' : ''}. Press to choose or manage recipes">
        <span class="num">{String(recipe.index + 1).padStart(2, '0')}</span>
        <span class="label">{recipe.value?.name.toUpperCase() ?? '--'}</span>
        {#if recipe.edited}<span class="edited">● edited</span>{/if}
      </button>
    </Screen>
    <Key icon="next" aria-label="Next recipe" onclick={() => recipe.step(1)} />
    <Key label="Save" disabled={!recipe.edited} onclick={save} title="Save recipe ({mod}S)" />
  </div>

  <div class="right">
    <Key dark icon="plus" label="Add" onclick={() => add('files')} title="Add images ({mod}O), or folders ({mod}⇧O)" />
    <Key dark icon="history" aria-label="Past runs" title="Past runs and undo" onclick={() => (ui.dialog = 'history')} />
    <Key dark icon="settings" aria-label="Settings" title="Settings and right-click menus" onclick={() => (ui.dialog = 'settings')} />
    <Key dark icon="command" aria-label="Commands" title="Commands ({mod}K)" onclick={() => (ui.dialog = 'palette')} />
  </div>
</header>

<style>
  .head { grid-column: 1 / -1; display: flex; align-items: center; gap: var(--sp-8); }
  .pal { all: unset; cursor: copy; border-radius: var(--r-key); display: grid; }
  .pal:focus-visible { outline: var(--focus); outline-offset: var(--focus-offset); }
  .word { font: 700 22px/1 var(--font-ui); letter-spacing: -0.04em; color: #18191b; display: flex; align-items: baseline; gap: var(--sp-4); white-space: nowrap; }
  .recipe { display: flex; align-items: center; gap: var(--sp-3); margin-left: var(--sp-8); }
  .recipe :global(.recipe__screen) { width: 290px; height: var(--key-h); }
  .recipe__name { all: unset; box-sizing: border-box; cursor: pointer; width: 100%; height: 100%; display: flex; align-items: center; gap: var(--sp-6); padding: 0 var(--sp-6);
    font: 12.5px/1 var(--font-data); white-space: nowrap; overflow: hidden; }
  .recipe__name:focus-visible { outline: var(--focus-on-screen); outline-offset: -3px; }
  .num { color: var(--output); }
  .label { overflow: hidden; text-overflow: ellipsis; }
  .edited { margin-left: auto; color: var(--output); font-size: var(--fs-small); }
  .right { margin-left: auto; display: flex; align-items: center; gap: var(--sp-4); }
  @media (max-width: 1320px) { .word .legend { display: none; } .recipe :global(.recipe__screen) { width: 230px; } }
</style>
