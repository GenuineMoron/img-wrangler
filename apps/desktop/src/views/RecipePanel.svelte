<!-- SPDX-License-Identifier: AGPL-3.0-or-later
  The recipe, top to bottom in the order it runs. Every step shows what it will do; open one
  for its settings. Add-ons switch on and off with their key, and Add puts new ones in. -->
<script lang="ts">
  import Icon from '$lib/components/Icon.svelte';
  import Key from '$lib/components/Key.svelte';
  import { addable, steps } from '$lib/modules/registry';
  import type { UiModule } from '$lib/modules/types';
  import { recipe } from '$lib/state/recipe.svelte';
  import { ui } from '$lib/state/ui.svelte';
  import type { AddOn } from '$lib/types';

  const r = $derived(recipe.value);
  const list = $derived(r ? steps(r) : []);
  const more = $derived(r ? addable(r) : []);
  const on = $derived(r ? list.filter(s => (s.addon ? s.addon.enabled : true) && s.module.active(r)).length : 0);
  let adding = $state(false);

  const glyph: Record<string, string> = { size: '■', format: '●', quality: '▲', name: '◆' };

  function add(m: UiModule) {
    if (m.kind !== 'addon' || !r) return;
    r.addons.push(m.create());
    adding = false;
    ui.toggle(m.id, true);
  }

  function remove(a: AddOn) {
    if (!r) return;
    r.addons.splice(r.addons.indexOf(a), 1);
  }

  function open(m: UiModule) {
    const hasDetails = m.kind === 'addon' || !!m.Details;
    if (hasDetails) { ui.toggle(m.id); return; }
    // No settings beyond its channel: go to the channel instead.
    document.querySelector<HTMLElement>(`#chan-${m.id} [tabindex="0"]`)?.focus();
  }
</script>

<aside class="panel recipe-panel" aria-label="Recipe steps">
  <div class="top">
    <span class="legend">Recipe</span>
    <span class="legend legend--quiet">{list.length} steps · {on} on</span>
  </div>

  <ol class="steps">
    {#each list as s (s.module.id + (s.addon ? r?.addons.indexOf(s.addon) : ''))}
      {@const m = s.module}
      {@const isOpen = ui.step === m.id}
      {@const idle = r ? !m.active(r) : true}
      <li class="step" class:open={isOpen} class:idle style:--accent="var(--accent-{m.accent})" style:--on-accent="var(--on-{m.accent})">
        <div class="line">
          {#if s.addon}
            <Key icon={m.icon} pressed={s.addon.enabled} aria-label="{m.label} on" onclick={() => (s.addon!.enabled = !s.addon!.enabled)} />
          {:else}
            <span class="badge" aria-hidden="true">{#if glyph[m.id]}<b>{glyph[m.id]}</b>{:else}<Icon name={m.icon} />{/if}</span>
          {/if}
          <button type="button" class="row" id="step-{m.id}" aria-expanded={m.kind === 'addon' || m.Details ? isOpen : undefined} onclick={() => open(m)}>
            <span class="name">{m.label}</span>
            <span class="sum">{r ? m.summary(r) : ''}</span>
            {#if m.kind === 'addon' || m.Details}<span class="chev"><Icon name={isOpen ? 'up' : 'down'} size={14} /></span>{/if}
          </button>
        </div>
        {#if isOpen}
          <div class="body">
            {#if m.kind === 'addon' && s.addon}
              <m.Details value={s.addon as never} />
              <div class="remove"><Key icon="trash" label="Remove {m.label.toLowerCase()}" onclick={() => remove(s.addon!)} /></div>
            {:else if m.kind === 'core' && m.Details}
              <m.Details />
            {/if}
          </div>
        {/if}
      </li>
    {/each}
  </ol>

  <div class="add">
    {#if adding}
      <div class="menu" role="menu" aria-label="Add a step">
        {#each more as m (m.id)}
          <button type="button" role="menuitem" onclick={() => add(m)}><Icon name={m.icon} /><span><b>{m.label}</b>{m.blurb}</span></button>
        {/each}
      </div>
    {/if}
    <Key insert label={adding ? 'Close' : 'Add step'} disabled={!more.length} aria-expanded={adding} onclick={() => (adding = !adding)}
      title={more.length ? more.map(m => m.label).join(', ') : 'Every add-on is in this recipe'} />
  </div>
</aside>

<style>
  .recipe-panel { padding: var(--sp-6) var(--sp-6) var(--sp-6) var(--sp-7); display: grid; grid-template-rows: auto minmax(0, 1fr) auto; gap: var(--sp-4); }
  .top { display: flex; justify-content: space-between; align-items: baseline; padding-right: var(--sp-2); }
  .steps { list-style: none; margin: 0; padding: 0 var(--sp-2) 0 0; overflow-y: auto; scrollbar-width: thin; }
  .step { border-top: 1px solid var(--seam-dark); box-shadow: inset 0 1px 0 var(--seam-light); }
  .step:first-child { border-top: 0; box-shadow: none; }
  .line { display: grid; grid-template-columns: var(--key-h) minmax(0, 1fr); align-items: center; gap: var(--sp-5); padding: 5px 0; }
  .badge { width: var(--key-h); height: var(--key-h); display: grid; place-items: center; color: var(--accent); }
  .badge b { font-size: 12px; }
  .idle .badge { color: var(--ink-2); opacity: .7; }
  .row { all: unset; box-sizing: border-box; cursor: pointer; display: grid; grid-template-columns: auto minmax(0, 1fr) auto; align-items: center; gap: var(--sp-5); min-height: var(--key-h); border-radius: var(--r-chip); padding: 0 2px; }
  .row:focus-visible { outline: var(--focus); outline-offset: 2px; }
  .name { font: 600 var(--fs-body)/1.2 var(--font-ui); color: var(--ink-1); }
  .idle .name { color: var(--ink-2); font-weight: 500; }
  .sum { font: var(--fs-small)/1.2 var(--font-data); color: var(--ink-2); text-align: right; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .chev { color: var(--ink-2); display: grid; }
  .body { padding: 0 0 var(--sp-4) calc(var(--key-h) + var(--sp-5)); }
  .remove { display: flex; justify-content: flex-end; }
  .add { position: relative; display: grid; }
  .menu { position: absolute; bottom: calc(100% + 6px); left: 0; right: 0; z-index: 10; display: grid; gap: 2px; padding: 5px; border-radius: 10px;
    background: var(--surface-dark); box-shadow: 0 12px 30px rgba(0, 0, 0, .35); }
  .menu button { all: unset; cursor: pointer; display: grid; grid-template-columns: 16px 1fr; gap: 10px; align-items: start; padding: 8px 10px; border-radius: 6px; color: var(--ink-on-dark); font: 12px/1.35 var(--font-ui); }
  .menu button b { display: block; font-weight: 600; font-size: 13px; }
  .menu button span { color: #b9b9b5; }
  .menu button:hover, .menu button:focus-visible { background: var(--surface-dark-hi); }
</style>
