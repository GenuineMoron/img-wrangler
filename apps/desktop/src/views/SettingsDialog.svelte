<!-- SPDX-License-Identifier: AGPL-3.0-or-later
  Settings: Wrangler in the file manager's right-click menu, the menu bar icon and what a drop
  on it does, starting at login, updates and effects. Each one takes effect as it's switched.
  Which recipes the menus offer is set with the recipes themselves (the Recipes list). -->
<script lang="ts">
  import Dialog from '$lib/components/Dialog.svelte';
  import Key from '$lib/components/Key.svelte';
  import Led from '$lib/components/Led.svelte';
  import Segmented from '$lib/components/Segmented.svelte';
  import Switch from '$lib/components/Switch.svelte';
  import { backend } from '$lib/ipc';
  import { recipe } from '$lib/state/recipe.svelte';
  import { ui } from '$lib/state/ui.svelte';
  import { updates } from '$lib/state/updates.svelte';
  import { fx, type FxMode } from '$lib/state/fx.svelte';
  import type { AppSettings, Integrations } from '$lib/types';

  let info = $state<Integrations | null>(null);
  let app = $state<AppSettings | null>(null);
  let busy = $state(false);
  let error = $state<string | null>(null);
  const bar = backend.platform === 'mac' ? 'menu bar' : 'system tray';
  const installed = $derived(info?.targets.some(t => t.installed) ?? false);

  $effect(() => {
    if (ui.dialog !== 'settings') return;
    backend.integrations().then(i => (info = i));
    backend.appSettings().then(a => (app = a));
  });

  /** Every setting here takes effect as it's pressed. */
  async function apply(change: () => Promise<Integrations>) {
    busy = true;
    error = null;
    try {
      info = await change();
    } catch (e) {
      error = String(e);
    } finally {
      busy = false;
    }
  }

  /** Dropped on the hat: open in the window (null), or run a recipe there and then. */
  async function setDrop(id: string | null) {
    await backend.setDropRecipe(id);
    app = { ...app!, drop_runs: id };
  }
  const dropRecipe = $derived(recipe.shown.find(e => e.id === app?.drop_runs));

  function chooseDropRecipe() {
    backend.contextMenu(recipe.shown.map(e => ({ text: e.name, checked: e.id === app?.drop_runs, action: () => setDrop(e.id) })));
  }

  async function setApp(change: Partial<Pick<AppSettings, 'menu_bar' | 'auto_update' | 'login'>>) {
    if (!app) return;
    error = null;
    try {
      app = await backend.setAppSettings({ menu_bar: app.menu_bar, auto_update: app.auto_update, login: app.login, ...change });
    } catch (e) {
      error = String(e);
    }
  }
</script>

<Dialog open={ui.dialog === 'settings'} onclose={() => (ui.dialog = null)} title="Settings" wide>
  <section class="sec">
    <h3 class="legend">Right-click menus</h3>
    <p class="note">Select images or folders in your file manager, right-click, and choose Wrangler. A recipe runs straight away in a small window, with Undo. “Open in Wrangler…” loads them here. The menu offers the recipes that are switched on.</p>
    {#if info}
      {#if info.targets.length}
        <ul class="targets">
          {#each info.targets as t (t.target)}
            <li><Led on={t.installed} color={t.installed ? 'var(--accent-format)' : undefined} /><span>{t.label}</span><span class="note">{t.place ?? (t.installed ? 'in the right-click menu' : 'not added')}</span></li>
          {/each}
        </ul>
      {:else}
        <p class="note">No supported file manager found. Wrangler works with Finder, Explorer, Dolphin, Files (Nautilus), Nemo, Thunar, Caja, PCManFM and Luciole.</p>
      {/if}
      {#if info.targets.length}
        <div class="pins">
          <Switch label="Show Wrangler in right-click menus" on={installed} disabled={busy} onchange={on => apply(() => backend.setMenus(on))} />
          <Key label="Choose the recipes…" onclick={() => (ui.dialog = 'recipes')} />
        </div>
      {/if}
    {/if}
  </section>

  {#if app}
    <section class="sec">
      <h3 class="legend">{bar === 'menu bar' ? 'Menu bar' : 'System tray'}</h3>
      {#if app.menu_bar_available}
        <p class="note">The hat sits in the {bar}. Its menu runs a recipe on images you choose, and takes back the last run. Closing the window leaves Wrangler there, ready.</p>
        <div class="pins">
          <Switch label="Keep the hat in the {bar}" on={app.menu_bar} onchange={on => setApp({ menu_bar: on })} />
          <Switch label="Open at login" on={app.login} disabled={!app.menu_bar} onchange={on => setApp({ login: on })} title={app.menu_bar ? undefined : `Needs the hat in the ${bar}`} />
        </div>
        {#if backend.platform === 'mac' && app.menu_bar}
          <div class="line">
            <span class="legend legend--quiet">Images dropped on the hat</span>
            <Segmented label="Images dropped on the hat" neutral value={dropRecipe ? 'run' : 'open'}
              options={[{ id: 'open', label: 'Open in Wrangler' }, { id: 'run', label: 'Run a recipe' }]}
              onchange={v => setDrop(v === 'open' ? null : (dropRecipe ?? recipe.shown[0])?.id ?? null)} />
            {#if dropRecipe}<Key label="{dropRecipe.name} ▾" aria-label="Recipe to run: {dropRecipe.name}. Press to choose another" onclick={chooseDropRecipe} />{/if}
          </div>
        {/if}
      {:else}
        <p class="note">This system has no tray library, so the hat can't sit in the bar. Install libayatana-appindicator with your package manager and reopen Wrangler.</p>
      {/if}
    </section>

    <section class="sec">
      <h3 class="legend">Updates</h3>
      <p class="note">
        {#if updates.checking}Looking for a new version…
        {:else if updates.found}Wrangler {updates.found.version} is out.
        {:else if updates.upToDate}You have the newest version, {__APP_VERSION__}.
        {:else}Wrangler looks for a new version when it starts and every few hours, and tells you what changed before anything is installed.{/if}
      </p>
      {#if updates.error}<p class="note alarm">{updates.error}</p>{/if}
      <div class="pins">
        <Switch label="Check automatically" on={app.auto_update} onchange={on => setApp({ auto_update: on })} />
        {#if updates.found}
          <Key dark label="See what's new" onclick={() => (ui.dialog = 'update')} />
        {:else}
          <Key label="Check now" disabled={updates.checking} onclick={() => updates.check()} />
        {/if}
      </div>
    </section>
  {/if}

  <section class="sec">
    <h3 class="legend">Effects</h3>
    <p class="note">Light effects drop the soft shadows and glows, for computers that draw without a graphics card. Automatic chooses for you{fx.mode === 'auto' && fx.slow !== null ? `, and chose ${fx.light ? 'light' : 'full'} on this one` : ''}.</p>
    <div class="pins">
      <Segmented label="Effects" class="fx" neutral value={fx.mode} onchange={v => fx.set(v as FxMode)}
        options={[{ id: 'auto', label: 'Automatic' }, { id: 'full', label: 'Full' }, { id: 'light', label: 'Light' }]} />
    </div>
  </section>

  {#if error}<p class="note alarm" role="alert">{error}</p>{/if}

  <section class="sec">
    <h3 class="legend">About</h3>
    <p class="note">Wrangler {__APP_VERSION__} · free software under the GNU AGPL 3.0 or later · undo history is kept for 30 days.</p>
  </section>
</Dialog>

<style>
  .sec { display: grid; gap: var(--sp-5); }
  .sec + .sec { padding-top: var(--sp-6); border-top: 1px solid var(--seam-dark); box-shadow: inset 0 1px 0 var(--seam-light); }
  h3 { display: flex; gap: 8px; }
  .targets { list-style: none; margin: 0; padding: 0; display: grid; gap: 6px; }
  .targets li { display: grid; grid-template-columns: 10px 140px 1fr; gap: 10px; align-items: center; font: 500 13px/1.3 var(--font-ui); }
  .pins { display: flex; flex-wrap: wrap; align-items: center; gap: var(--sp-4) var(--sp-8); }
  .line { display: flex; flex-wrap: wrap; align-items: center; gap: var(--sp-4) var(--sp-6); }
  .line :global(.seg) { width: 300px; }
  .pins :global(.seg.fx) { width: 300px; }
  /* A switch here reads as a sentence: its label first, in ordinary case. */
  .sec :global(.switch) { flex-direction: row-reverse; text-transform: none; letter-spacing: 0; font: 500 13px/1.3 var(--font-ui); }
  .alarm { color: #c22a20; }
</style>
