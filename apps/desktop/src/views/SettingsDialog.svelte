<!-- SPDX-License-Identifier: AGPL-3.0-or-later
  Settings: right-click menus in the file manager (which recipes to offer, install, update,
  remove), the menu bar icon, starting at login, and updates. -->
<script lang="ts">
  import Dialog from '$lib/components/Dialog.svelte';
  import Key from '$lib/components/Key.svelte';
  import Led from '$lib/components/Led.svelte';
  import { backend } from '$lib/ipc';
  import { recipe } from '$lib/state/recipe.svelte';
  import { ui } from '$lib/state/ui.svelte';
  import { updates } from '$lib/state/updates.svelte';
  import { fx, type FxMode } from '$lib/state/fx.svelte';
  import type { AppSettings, Integrations } from '$lib/types';

  const MAX_PINS = 6;
  let info = $state<Integrations | null>(null);
  let pins = $state<string[]>([]);
  let busy = $state(false);
  let error = $state<string | null>(null);
  let app = $state<AppSettings | null>(null);
  const bar = backend.platform === 'mac' ? 'menu bar' : 'system tray';

  $effect(() => {
    if (ui.dialog !== 'settings') return;
    backend.integrations().then(i => { info = i; pins = [...i.pins]; });
    backend.appSettings().then(a => (app = a));
  });

  async function setApp(change: Partial<Pick<AppSettings, 'menu_bar' | 'auto_update' | 'login'>>) {
    if (!app) return;
    error = null;
    try {
      app = await backend.setAppSettings({ menu_bar: app.menu_bar, auto_update: app.auto_update, login: app.login, ...change });
    } catch (e) {
      error = String(e);
    }
  }

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
    <p class="note">Select images or folders in your file manager, right-click, and choose Wrangler. Pinned recipes run straight away in a small window, with Undo. “Open in Wrangler…” loads them here.</p>
    {#if info}
      {#if info.targets.length}
        <ul class="targets">
          {#each info.targets as t (t.target)}
            <li><Led on={t.installed} color={t.installed ? 'var(--accent-format)' : undefined} /><span>{t.label}</span><span class="note">{t.installed ? 'installed' : 'not installed'}</span></li>
          {/each}
        </ul>
      {:else}
        <p class="note">No supported file manager found. Wrangler works with Finder, Explorer, Dolphin, Files (Nautilus), Nemo, Thunar and Luciole.</p>
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

  {#if app}
    <section class="sec">
      <h3 class="legend">{bar === 'menu bar' ? 'Menu bar' : 'System tray'}</h3>
      {#if app.menu_bar_available}
        <p class="note">The hat sits in the {bar}. Its menu runs your pinned recipes on images you choose, and takes back the last run.{backend.platform === 'mac' ? ' Drop images straight on the hat to run a recipe on them.' : ''} Closing the window leaves Wrangler there, ready.</p>
        <div class="pins">
          <Key label="Keep in the {bar}" pressed={app.menu_bar} onclick={() => setApp({ menu_bar: !app!.menu_bar })} />
          <Key label="Open at login" pressed={app.login} disabled={!app.menu_bar} onclick={() => setApp({ login: !app!.login })} title={app.menu_bar ? undefined : `Needs the ${bar} icon`} />
        </div>
        {#if backend.platform === 'mac' && app.menu_bar}
          <h3 class="legend legend--quiet">Images dropped on the hat</h3>
          <div class="pins" role="radiogroup" aria-label="Recipe for images dropped on the hat">
            {#each recipe.entries.filter(e => pins.includes(e.id)) as e (e.id)}
              <Key label={e.name} checked={app.drop_recipe === e.id} onclick={async () => { await backend.setDropRecipe(e.id); app = { ...app!, drop_recipe: e.id }; }} />
            {/each}
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
        <Key label="Check automatically" pressed={app.auto_update} onclick={() => setApp({ auto_update: !app!.auto_update })} />
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
    <div class="pins" role="radiogroup" aria-label="Effects">
      {#each [['auto', 'Automatic'], ['full', 'Full'], ['light', 'Light']] as [id, label] (id)}
        <Key {label} checked={fx.mode === id} onclick={() => fx.set(id as FxMode)} />
      {/each}
    </div>
  </section>

  {#if error}<p class="note alarm" role="alert">{error}</p>{/if}

  <section class="sec">
    <h3 class="legend">About</h3>
    <p class="note">Wrangler {__APP_VERSION__} · free software under the GNU AGPL 3.0 or later · undo history is kept for 30 days.</p>
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
