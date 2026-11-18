<!-- SPDX-License-Identifier: AGPL-3.0-or-later
  The main unit and its keyboard. -->
<script lang="ts">
  import { onMount } from 'svelte';
  import { backend } from '$lib/ipc';
  import { CHANNELS } from '$lib/modules/registry';
  import { batch } from '$lib/state/batch.svelte';
  import { start } from '$lib/state/controller.svelte';
  import { recipe } from '$lib/state/recipe.svelte';
  import { ui } from '$lib/state/ui.svelte';
  import { fx } from '$lib/state/fx.svelte';
  import ChannelStrip from './ChannelStrip.svelte';
  import Display from './Display.svelte';
  import Header from './Header.svelte';
  import HistoryDialog from './HistoryDialog.svelte';
  import OutputPanel from './OutputPanel.svelte';
  import Palette from './Palette.svelte';
  import RecipePanel from './RecipePanel.svelte';
  import RecipesDialog from './RecipesDialog.svelte';
  import SettingsDialog from './SettingsDialog.svelte';
  import UpdateDialog from './UpdateDialog.svelte';

  let ready = $state(false);
  let failed = $state<string | null>(null);

  onMount(() => {
    let stop: (() => void) | undefined;
    start()
      .then(s => {
        stop = s;
        ready = true;
        requestAnimationFrame(() => backend.ready());
        const booted = performance.now();
        // Once the window is up: is drawing fast enough for the full effects?
        setTimeout(() => fx.check(), 600);
        // Started for a self-test: load it only now, so it costs nothing otherwise.
        backend.selftest().then(async cfg => {
          if (!cfg) return;
          const t = await import('$lib/selftest');
          (window as unknown as { __wrangler: unknown }).__wrangler = { layoutHere: t.layoutHere, ui, batch };
          await t.run(cfg, booted);
        });
      })
      .catch(e => { failed = String(e); backend.ready(); });
    return () => stop?.();
  });

  const typing = (t: EventTarget | null) => t instanceof HTMLElement && (t.isContentEditable || /^(INPUT|TEXTAREA|SELECT)$/.test(t.tagName));

  async function keydown(e: KeyboardEvent) {
    const mod = backend.platform === 'mac' ? e.metaKey : e.ctrlKey;
    if (mod && e.key === 'Enter') { e.preventDefault(); batch.run(); return; }
    if (mod && e.key.toLowerCase() === 'k') { e.preventDefault(); ui.dialog = ui.dialog === 'palette' ? null : 'palette'; return; }
    if (mod && e.key.toLowerCase() === 'o') { e.preventDefault(); batch.add(await backend.pick(e.shiftKey ? 'folder' : 'files')); return; }
    if (mod && e.key.toLowerCase() === 's') {
      e.preventDefault();
      if (recipe.entry?.built_in || e.shiftKey) ui.dialog = 'recipes';
      else if (recipe.edited) { await recipe.save(); ui.say(`Saved “${recipe.value?.name}”.`); }
      return;
    }
    if (mod && e.key === ',') { e.preventDefault(); ui.dialog = 'settings'; return; }
    if (e.key === 'Escape' && batch.running && !ui.dialog) { batch.cancel(); return; }
    if (ui.dialog || typing(e.target) || mod || e.altKey) return;
    const slot = Number(e.key);
    if (slot >= 1 && slot <= CHANNELS.length) {
      e.preventDefault();
      document.querySelector<HTMLElement>(`#chan-${CHANNELS[slot - 1].id} .screen [tabindex="0"], #chan-${CHANNELS[slot - 1].id} .screen input, #chan-${CHANNELS[slot - 1].id} [tabindex="0"]`)?.focus();
    }
    if (e.key === 'b' || e.key === 'B') ui.view = ui.view === 'original' ? 'auto' : 'original';
  }
</script>

<svelte:window onkeydown={keydown} />

{#if ready && recipe.value}
  <div class="device mat-case" class:dropping={ui.dropping}>
    <Header />
    <Display />
    <RecipePanel />
    <ChannelStrip />
    <OutputPanel />
  </div>
  <Palette />
  <RecipesDialog />
  <SettingsDialog />
  <HistoryDialog />
  <UpdateDialog />
{:else if failed}
  <div class="device mat-case failed"><p role="alert">Wrangler couldn’t start: {failed}</p></div>
{:else}
  <div class="device mat-case" aria-busy="true"></div>
{/if}

<style>
  .dropping { box-shadow: inset 0 0 0 3px var(--output); }
  .failed { display: grid; grid-template: 1fr / 1fr; place-items: center; }
  .failed p { max-width: 60ch; font: 500 14px/1.5 var(--font-ui); }
</style>
