<!-- SPDX-License-Identifier: AGPL-3.0-or-later
  Choose a recipe, save the current one under a name, revert or delete. -->
<script lang="ts">
  import Dialog from '$lib/components/Dialog.svelte';
  import Field from '$lib/components/Field.svelte';
  import Key from '$lib/components/Key.svelte';
  import { recipe } from '$lib/state/recipe.svelte';
  import { ui } from '$lib/state/ui.svelte';

  let name = $state('');
  $effect(() => {
    if (ui.dialog === 'recipes') name = recipe.entry?.built_in ? `${recipe.value?.name ?? 'Recipe'} (mine)` : (recipe.value?.name ?? '');
  });
  const taken = $derived(recipe.entries.find(e => e.name.toLowerCase() === name.trim().toLowerCase()));

  async function save() {
    if (!name.trim()) return;
    await recipe.save(name.trim());
    ui.say(`Saved “${name.trim()}”.`);
    ui.dialog = null;
  }
</script>

<Dialog open={ui.dialog === 'recipes'} onclose={() => (ui.dialog = null)} title="Recipes">
  <ul class="list" aria-label="Recipes">
    {#each recipe.entries as e, i (e.id)}
      <li class:on={e.id === recipe.id}>
        <button type="button" class="pick" aria-current={e.id === recipe.id} onclick={() => recipe.load(e.id)}>
          <span class="num">{String(i + 1).padStart(2, '0')}</span>
          <span class="txt"><b>{e.name}</b><span>{e.description || (e.built_in ? 'Built in' : 'Your recipe')}</span></span>
        </button>
        {#if !e.built_in}<Key icon="trash" aria-label="Delete {e.name}" onclick={() => recipe.remove(e.id)} />{/if}
      </li>
    {/each}
  </ul>
  <div class="save">
    <span class="legend legend--quiet">Save current as</span>
    <Field label="Recipe name" bind:value={name} onkeydown={e => e.key === 'Enter' && save()} />
    <Key label={taken && !taken.built_in ? 'Replace' : 'Save'} disabled={!name.trim() || !!taken?.built_in} onclick={save} />
  </div>
  {#if taken?.built_in}<p class="note">Built-in recipes can’t be replaced; choose another name.</p>{/if}
  {#snippet actions()}
    <Key label="Revert changes" disabled={!recipe.edited} onclick={() => recipe.revert()} />
    <Key label="Done" onclick={() => (ui.dialog = null)} />
  {/snippet}
</Dialog>

<style>
  .list { list-style: none; margin: 0; padding: 0; display: grid; gap: 2px; }
  li { display: flex; align-items: center; gap: 6px; }
  .pick { all: unset; flex: 1; cursor: pointer; display: grid; grid-template-columns: 28px 1fr; gap: 10px; align-items: start; padding: 8px 10px; border-radius: 8px; }
  .pick:hover { background: rgba(0, 0, 0, .05); }
  .pick:focus-visible { outline: var(--focus); }
  .on .pick { background: var(--surface-screen); color: var(--screen-1); }
  .num { font: var(--fs-small)/1.5 var(--font-data); color: var(--output); }
  .txt { display: grid; gap: 2px; }
  .txt b { font: 600 13px/1.3 var(--font-ui); }
  .txt span { font: 12px/1.35 var(--font-ui); color: var(--ink-2); }
  .on .txt span { color: var(--screen-2); }
  .save { display: grid; grid-template-columns: auto 1fr auto; gap: var(--sp-5); align-items: center; padding-top: var(--sp-6); border-top: 1px solid var(--seam-dark); box-shadow: inset 0 1px 0 var(--seam-light); }
</style>
