<!-- SPDX-License-Identifier: AGPL-3.0-or-later
  Every action by name. Type to filter, arrows to move, Enter to do it. -->
<script lang="ts">
  import Dialog from '$lib/components/Dialog.svelte';
  import { backend } from '$lib/ipc';
  import { ADDONS } from '$lib/modules/registry';
  import { batch } from '$lib/state/batch.svelte';
  import { recipe } from '$lib/state/recipe.svelte';
  import { ui } from '$lib/state/ui.svelte';

  interface Command { label: string; hint?: string; run: () => unknown; when?: () => boolean }
  const mod = backend.platform === 'mac' ? '⌘' : 'Ctrl+';

  const all = $derived<Command[]>([
    { label: 'Run', hint: `${mod}↵`, run: () => batch.run(), when: () => !batch.running && batch.items.length > 0 },
    { label: 'Stop the run', hint: 'Esc', run: () => batch.cancel(), when: () => batch.running },
    { label: 'Add images…', hint: `${mod}O`, run: async () => batch.add(await backend.pick('files')) },
    { label: 'Add a folder…', hint: `${mod}⇧O`, run: async () => batch.add(await backend.pick('folder')) },
    { label: 'Clear the batch', run: () => batch.clear(), when: () => !batch.empty && !batch.running },
    { label: 'Undo the last run', run: () => batch.undo(), when: () => !!batch.undoable },
    { label: 'Save recipe', hint: `${mod}S`, run: () => recipe.save(), when: () => recipe.edited && !recipe.entry?.built_in },
    { label: 'Save recipe as…', run: () => (ui.dialog = 'recipes') },
    { label: 'Revert recipe changes', run: () => recipe.revert(), when: () => recipe.edited },
    ...recipe.entries.map(e => ({ label: `Recipe: ${e.name}`, hint: e.built_in ? 'built in' : 'yours', run: () => recipe.load(e.id) })),
    ...ADDONS.map(m => ({ label: `Add step: ${m.label}`, hint: m.blurb, run: () => { recipe.value?.addons.push(m.create()); ui.toggle(m.id, true); }, when: () => !recipe.addon(m.addon) })),
    { label: 'Show result', run: () => (ui.view = 'result') },
    { label: 'Show original', hint: 'space', run: () => (ui.view = 'original') },
    { label: 'Show before and after', run: () => (ui.view = 'split') },
    { label: 'Past runs', run: () => (ui.dialog = 'history') },
    { label: 'Settings and right-click menus', hint: `${mod},`, run: () => (ui.dialog = 'settings') },
  ]);

  let query = $state('');
  let at = $state(0);
  const shown = $derived(all.filter(c => (c.when?.() ?? true) && query.toLowerCase().split(/\s+/).every(w => c.label.toLowerCase().includes(w))));
  $effect(() => { query; at = 0; });
  $effect(() => { if (ui.dialog === 'palette') query = ''; });

  function go(c: Command | undefined) {
    if (!c) return;
    ui.dialog = null;
    queueMicrotask(() => c.run());
  }

  function keydown(e: KeyboardEvent) {
    if (e.key === 'ArrowDown') { e.preventDefault(); at = Math.min(shown.length - 1, at + 1); }
    else if (e.key === 'ArrowUp') { e.preventDefault(); at = Math.max(0, at - 1); }
    else if (e.key === 'Enter') { e.preventDefault(); go(shown[at]); }
  }
</script>

<Dialog open={ui.dialog === 'palette'} onclose={() => (ui.dialog = null)} title="Commands">
  <label class="field search">
    <span class="sr-only">Command</span>
    <!-- svelte-ignore a11y_autofocus -->
    <input bind:value={query} autofocus placeholder="Type a command or recipe" onkeydown={keydown} role="combobox" aria-expanded="true" aria-controls="commands" aria-activedescendant="cmd-{at}" spellcheck="false" />
  </label>
  <ul id="commands" class="list" role="listbox" aria-label="Commands">
    {#each shown as c, i (c.label)}
      <li id="cmd-{i}" role="option" aria-selected={i === at} onclick={() => go(c)} onpointermove={() => (at = i)} onkeydown={() => {}}>
        <span>{c.label}</span>{#if c.hint}<kbd>{c.hint}</kbd>{/if}
      </li>
    {:else}
      <li class="none">Nothing matches.</li>
    {/each}
  </ul>
</Dialog>

<style>
  .search { height: 34px; }
  .list { list-style: none; margin: 0; padding: 0; display: grid; gap: 1px; max-height: 50vh; overflow: auto; }
  li { display: flex; justify-content: space-between; gap: 12px; padding: 7px 10px; border-radius: 6px; cursor: pointer; font: 500 13px/1.3 var(--font-ui); }
  li[aria-selected='true'] { background: var(--surface-dark); color: var(--ink-on-dark); }
  kbd { font: var(--fs-small)/1.3 var(--font-data); color: var(--ink-2); white-space: nowrap; overflow: hidden; text-overflow: ellipsis; max-width: 50%; }
  li[aria-selected='true'] kbd { color: #b9b9b5; }
  .none { color: var(--ink-2); cursor: default; }
</style>
