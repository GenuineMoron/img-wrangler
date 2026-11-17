<!-- SPDX-License-Identifier: AGPL-3.0-or-later
  Right-click menus in the file manager: which recipes to offer, install, update, remove. -->
<script lang="ts">
  import Dialog from '$lib/components/Dialog.svelte';
  import Key from '$lib/components/Key.svelte';
  import Led from '$lib/components/Led.svelte';
  import { backend } from '$lib/ipc';
  import { recipe } from '$lib/state/recipe.svelte';
  import { ui } from '$lib/state/ui.svelte';
  import type { Integrations } from '$lib/types';

  const MAX_PINS = 6;
  let info = $state<Integrations | null>(null);
  let pins = $state<string[]>([]);
  let busy = $state(false);
  let error = $state<string | null>(null);

  $effect(() => {
    if (ui.dialog !== 'settings') return;
    backend.integrations().then(i => { info = i; pins = [...i.pins]; });
  });

  const installed = $derived(info?.targets.some(t => t.installed) ?? false);

  function toggle(id: string) {
    pins = pins.includes(id) ? pins.filter(p => p !== id) : pins.length < MAX_PINS ? [...pins, id] : pins;
  }

  async function apply(install: boolean) {
    busy = true;
    error = null;
    try {
      info = await backend.setIntegrations(install, pins);
      ui.say(install ? 'Right-click menus are set up.' : 'Right-click menus removed.');
    } catch (e) {
      error = String(e);
    } finally {
      busy = false;
    }
  }
</script>

<Dialog open={ui.dialog === 'settings'} onclose={() => (ui.dialog = null)} title="Settings" wide>
  <section class="sec">
    <h3 class="legend">Right-click menus</h3>
    <p class="note">Select images or folders in your file manager, right-click, and choose Wrangler. Pinned recipes run straight away in a small window, with Undo; “Open in Wrangler…” loads them here.</p>
    {#if info}
      {#if info.targets.length}
        <ul class="targets">
          {#each info.targets as t (t.target)}
            <li><Led on={t.installed} color={t.installed ? 'var(--accent-format)' : undefined} /><span>{t.label}</span><span class="note">{t.installed ? 'installed' : 'not installed'}</span></li>
          {/each}
        </ul>
      {:else}
        <p class="note">No supported file manager found. Wrangler works with Finder, Explorer, Dolphin, Files (Nautilus), Nemo and Thunar.</p>
      {/if}
    {/if}
  </section>

  <section class="sec">
    <h3 class="legend">Pinned recipes <span class="legend--quiet">{pins.length} of {MAX_PINS}</span></h3>
    <div class="pins">
      {#each recipe.entries as e (e.id)}
        <Key label={e.name} pressed={pins.includes(e.id)} disabled={!pins.includes(e.id) && pins.length >= MAX_PINS} onclick={() => toggle(e.id)} />
      {/each}
    </div>
  </section>

  {#if error}<p class="note alarm" role="alert">{error}</p>{/if}

  <section class="sec">
    <h3 class="legend">About</h3>
    <p class="note">Wrangler 0.1 · free software under the GNU AGPL 3.0 or later · undo history is kept for 30 days.</p>
  </section>

  {#snippet actions()}
    {#if installed}<Key label="Remove menus" disabled={busy} onclick={() => apply(false)} />{/if}
    <Key label={installed ? 'Update menus' : 'Add to right-click menus'} disabled={busy || !info?.targets.length} onclick={() => apply(true)} />
  {/snippet}
</Dialog>

<style>
  .sec { display: grid; gap: var(--sp-5); }
  .sec + .sec { padding-top: var(--sp-6); border-top: 1px solid var(--seam-dark); box-shadow: inset 0 1px 0 var(--seam-light); }
  h3 { display: flex; gap: 8px; }
  .targets { list-style: none; margin: 0; padding: 0; display: grid; gap: 6px; }
  .targets li { display: grid; grid-template-columns: 10px 140px 1fr; gap: 10px; align-items: center; font: 500 13px/1.3 var(--font-ui); }
  .pins { display: flex; flex-wrap: wrap; gap: var(--sp-4); }
  .alarm { color: #c22a20; }
</style>
