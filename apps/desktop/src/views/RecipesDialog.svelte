<!-- SPDX-License-Identifier: AGPL-3.0-or-later
  The recipes: pick one to use, and look after them. Each line says what the recipe does, read
  from its own settings. A recipe can be renamed, copied, deleted and switched off, and the
  list dragged into any order. The ones that are on are what the window, the right-click menus
  and the menu bar offer, in this order. -->
<script lang="ts">
  import Dialog from '$lib/components/Dialog.svelte';
  import Icon from '$lib/components/Icon.svelte';
  import Key from '$lib/components/Key.svelte';
  import Switch from '$lib/components/Switch.svelte';
  import { backend } from '$lib/ipc';
  import { describe } from '$lib/modules/describe';
  import { recipe } from '$lib/state/recipe.svelte';
  import { ui } from '$lib/state/ui.svelte';

  const bar = backend.platform === 'mac' ? 'menu bar' : 'tray';
  let naming = $state<string | null>(null);
  let name = $state('');
  let deleting = $state<string | null>(null);
  let list: HTMLElement | undefined = $state();

  $effect(() => { if (ui.dialog !== 'recipes') { naming = null; deleting = null; } });

  async function use(id: string) {
    await recipe.load(id);
    ui.dialog = null;
  }

  function startNaming(id: string, current: string) {
    naming = id;
    name = current;
    deleting = null;
  }

  async function rename() {
    const id = naming;
    naming = null;
    const was = recipe.entries.find(e => e.id === id);
    if (!id || !was || !name.trim() || name.trim() === was.name) return;
    try {
      await recipe.rename(id, name.trim());
    } catch (e) {
      ui.say(String(e), true);
    }
  }

  async function add() {
    const id = await recipe.create();
    startNaming(id, 'New recipe');
  }

  async function copy(id: string) {
    const made = await recipe.duplicate(id);
    startNaming(made, recipe.entries.find(e => e.id === made)?.name ?? '');
  }

  async function restore() {
    const n = await recipe.restore();
    ui.say(n ? `Brought back ${n} starter ${n === 1 ? 'recipe' : 'recipes'}.` : 'Every starter recipe is already here.');
  }

  // ---------- dragging a line to a new place ----------
  let drag = $state<{ id: string; at: number } | null>(null);

  function grab(e: PointerEvent, id: string) {
    if (e.button !== 0 || !list) return;
    e.preventDefault();
    const rows = [...list.querySelectorAll<HTMLElement>('[data-row]')];
    const slot = (y: number) => {
      const i = rows.findIndex(r => { const b = r.getBoundingClientRect(); return y < b.top + b.height / 2; });
      return i < 0 ? rows.length : i;
    };
    drag = { id, at: slot(e.clientY) };
    const moved = (ev: PointerEvent) => { drag = { id, at: slot(ev.clientY) }; };
    const up = () => {
      removeEventListener('pointermove', moved);
      removeEventListener('pointerup', up);
      removeEventListener('pointercancel', up);
      const d = drag;
      drag = null;
      if (d) place(d.id, d.at);
    };
    addEventListener('pointermove', moved);
    addEventListener('pointerup', up);
    addEventListener('pointercancel', up);
  }

  /** Move a recipe to before whatever is at `at` now. */
  function place(id: string, at: number) {
    const ids = recipe.entries.map(e => e.id);
    const from = ids.indexOf(id);
    if (from < 0 || at === from || at === from + 1) return;
    ids.splice(from, 1);
    ids.splice(at > from ? at - 1 : at, 0, id);
    recipe.reorder(ids);
  }

  function nudge(e: KeyboardEvent, id: string) {
    const by = { ArrowUp: -1, ArrowDown: 2 }[e.key];
    if (!by) return;
    e.preventDefault();
    const from = recipe.entries.findIndex(x => x.id === id);
    if (from + by < 0 || from + by > recipe.entries.length) return;
    place(id, from + by);
    queueMicrotask(() => list?.querySelector<HTMLElement>(`[data-row="${id}"] .grip`)?.focus());
  }

  const focus = (el: HTMLInputElement) => { el.focus(); el.select(); };
</script>

<Dialog open={ui.dialog === 'recipes'} onclose={() => (ui.dialog = null)} title="Recipes" wide>
  <ul bind:this={list} class="list" aria-label="Recipes">
    {#each recipe.entries as e, i (e.id)}
      {@const current = e.id === recipe.id}
      {#if drag?.at === i}<li class="mark" aria-hidden="true"></li>{/if}
      <li class="row" class:current class:off={!e.on} class:lifted={drag?.id === e.id} data-row={e.id}>
        <button type="button" class="grip" aria-label="Move {e.name}. Drag, or use the up and down arrow keys" title="Drag to reorder"
          onpointerdown={ev => grab(ev, e.id)} onkeydown={ev => nudge(ev, e.id)}><Icon name="grip" /></button>

        {#if naming === e.id}
          <div class="what">
            <input class="field name-field" aria-label="Recipe name" spellcheck="false" autocomplete="off" bind:value={name} use:focus
              onkeydown={ev => { if (ev.key === 'Enter') rename(); if (ev.key === 'Escape') { ev.stopPropagation(); naming = null; } }} onblur={rename} />
          </div>
        {:else}
          <button type="button" class="what pick" aria-current={current} onclick={() => use(e.id)} title="Use {e.name}">
            <span class="title"><b>{e.name}</b>{#if current}<span class="tag">in use</span>{/if}{#if recipe.unsaved(e.id)}<span class="tag tag--edited">● changes not saved</span>{/if}</span>
            <span class="does">
              {#each describe(current && recipe.value ? recipe.value : e.recipe) as bit (bit.id)}
                <span class="bit" style:--dot="var(--accent-{bit.accent})"><i aria-hidden="true"></i><span class="sr-only">{bit.label}: </span>{bit.text}</span>
              {/each}
            </span>
          </button>
        {/if}

        {#if deleting === e.id}
          <span class="sure">
            <span class="note">{e.built_in ? 'Delete this starter?' : 'Delete for good?'}</span>
            <Key dark label="Delete" onclick={() => { deleting = null; recipe.remove(e.id); }} />
            <Key label="Keep" onclick={() => (deleting = null)} />
          </span>
        {:else}
          <span class="acts">
            <Key icon="pencil" aria-label="Rename {e.name}" title="Rename" onclick={() => startNaming(e.id, e.name)} />
            <Key icon="copy" aria-label="Make a copy of {e.name}" title="Make a copy" onclick={() => copy(e.id)} />
            {#if e.changed}<Key icon="restore" aria-label="Put {e.name} back as it came" title="Put back as it came" onclick={() => recipe.reset(e.id)} />{/if}
            <Key icon="trash" aria-label="Delete {e.name}" title="Delete" onclick={() => { deleting = e.id; naming = null; }} />
          </span>
          <Switch label={e.on ? 'On' : 'Off'} on={e.on} onchange={on => recipe.setOn(e.id, on)} aria-label="{e.name}: offered in the window and the menus"
            title={e.on ? 'On: offered in the window, the right-click menu and the menu bar' : 'Off: kept here, offered nowhere'} />
        {/if}
      </li>
    {/each}
    {#if drag?.at === recipe.entries.length}<li class="mark" aria-hidden="true"></li>{/if}
  </ul>

  {#snippet actions()}
    <p class="note foot-note">Recipes that are on are offered in the window, the right-click menu and the {bar}, in this order. To change one, use it, change the controls, then press Save.</p>
    <Key label="Bring back starters" title="Bring back the starter recipes you deleted" onclick={restore} />
    <Key dark insert label="New recipe" onclick={add} />
  {/snippet}
</Dialog>

<style>
  .list { list-style: none; margin: 0; padding: 0; display: grid; gap: 2px; }
  .row { display: grid; grid-template-columns: auto minmax(0, 1fr) auto auto; align-items: center; gap: var(--sp-4); padding: 3px var(--sp-4) 3px 2px; border-radius: 10px; }
  .row:hover { background: rgba(0, 0, 0, .04); }
  .row.current { background: rgba(0, 0, 0, .07); box-shadow: inset 0 0 0 1px var(--seam-dark); }
  .row.lifted { opacity: .35; }
  .mark { height: 2px; margin: -2px 8px; border-radius: 1px; background: var(--ink-1); }
  .grip { all: unset; cursor: grab; touch-action: none; display: grid; place-items: center; width: 22px; height: 34px; border-radius: 6px; color: var(--ink-2); }
  .grip:hover { color: var(--ink-1); }
  .grip:focus-visible { outline: var(--focus); }
  .what { min-width: 0; display: grid; gap: 3px; padding: 6px 4px; border-radius: 8px; }
  .pick { all: unset; cursor: pointer; min-width: 0; display: grid; gap: 4px; padding: 6px 4px; border-radius: 8px; }
  .pick:focus-visible { outline: var(--focus); }
  .title { display: flex; align-items: baseline; gap: var(--sp-4); min-width: 0; }
  .title b { font: 600 13.5px/1.25 var(--font-ui); white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .tag { font: 600 var(--fs-legend)/1 var(--font-ui); letter-spacing: var(--tracking-legend); text-transform: uppercase; color: var(--ink-2); white-space: nowrap; }
  .tag--edited { color: var(--output-lo); }
  .does { display: flex; flex-wrap: wrap; gap: 2px var(--sp-6); font: var(--fs-small)/1.35 var(--font-data); color: var(--ink-2); }
  .bit { display: inline-flex; align-items: center; gap: 5px; white-space: nowrap; }
  .bit i { width: 7px; height: 7px; border-radius: 50%; background: var(--dot); box-shadow: inset 0 -1px 1px rgba(0, 0, 0, .2); flex: none; }
  .off .pick { opacity: .55; }
  .name-field { width: 100%; font: 13px/1 var(--font-data); padding: 0 var(--sp-5); border: 0; color: var(--screen-1); -webkit-user-select: text; user-select: text; }
  .name-field:focus-visible { outline: var(--focus); outline-offset: var(--focus-offset); }
  .acts, .sure { display: flex; align-items: center; gap: var(--sp-3); }
  .acts { opacity: .55; transition: opacity var(--dur-latch); }
  .row:hover .acts, .row:focus-within .acts, .current .acts { opacity: 1; }
  .row :global(.switch) { width: 62px; justify-content: flex-end; }
  .foot-note { margin-right: auto; max-width: 56ch; align-self: center; }
</style>
