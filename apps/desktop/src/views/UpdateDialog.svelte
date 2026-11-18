<!-- SPDX-License-Identifier: AGPL-3.0-or-later
  A new version: what changed, and installing it (or, for a copy a package manager looks after,
  the command that does). -->
<script lang="ts">
  import Dialog from '$lib/components/Dialog.svelte';
  import Key from '$lib/components/Key.svelte';
  import Ladder from '$lib/components/Ladder.svelte';
  import Screen from '$lib/components/Screen.svelte';
  import { backend } from '$lib/ipc';
  import { notes } from '$lib/changelog';
  import { updates } from '$lib/state/updates.svelte';
  import { ui } from '$lib/state/ui.svelte';

  const u = $derived(updates.found);
  const lines = $derived(u ? notes(u.notes) : []);
  let copied = $state(false);

  async function copy() {
    if (!u?.manual) return;
    await navigator.clipboard.writeText(u.manual);
    copied = true;
    setTimeout(() => (copied = false), 1600);
  }
</script>

<Dialog open={ui.dialog === 'update'} onclose={() => !updates.installing && (ui.dialog = null)} title="Update">
  {#if u}
    <Screen class="ver">
      <span class="scr-label">NEW VERSION</span>
      <span class="num glow">{u.version}</span>
      <span class="scr-label">you have {u.current}{u.date ? ` · released ${u.date}` : ''}</span>
      {#if updates.installing}
        <Ladder lit={updates.progress < 0 ? 12 : 24 * updates.progress} kind="out" />
        <span class="scr-label">{updates.progress >= 1 ? 'installing, Wrangler will restart' : 'downloading'}</span>
      {/if}
    </Screen>
    <section class="notes" aria-label="What's new">
      <h3 class="legend">What's new</h3>
      {#each lines as l, i (i)}
        <p class:li={l.list}>{#each l.parts as p, j (j)}{#if p.code}<code>{p.text}</code>{:else}{p.text}{/if}{/each}</p>
      {:else}
        <p>No notes for this version.</p>
      {/each}
    </section>
    {#if u.manual}
      <p class="note">This copy was installed by your package manager, so it updates the same way. Run this in a terminal:</p>
      <code class="cmd">{u.manual}</code>
    {/if}
    {#if updates.error}<p class="note alarm" role="alert">{updates.error}</p>{/if}
  {/if}

  {#snippet actions()}
    <Key label="Later" disabled={updates.installing} onclick={() => (ui.dialog = null)} />
    {#if u?.manual}
      <Key label="Download page" onclick={() => backend.openUrl('https://genuinemoron.github.io/img-wrangler/')} />
      <Key dark label={copied ? 'Copied' : 'Copy command'} onclick={copy} />
    {:else}
      <Key dark label={updates.installing ? 'Installing…' : 'Install and restart'} disabled={updates.installing} onclick={() => updates.install()} />
    {/if}
  {/snippet}
</Dialog>

<style>
  :global(.ver) { padding: 14px 16px; display: grid; gap: 8px; }
  .num { font: var(--value-weight) 34px/1 var(--font-value); color: var(--screen-1); }
  .notes { display: grid; gap: 6px; }
  .notes p { margin: 0; font: 400 14px/1.5 var(--font-ui); color: var(--ink-1); }
  .notes p.li { padding-left: 16px; position: relative; }
  .notes p.li::before { content: ""; position: absolute; left: 3px; top: .62em; width: 5px; height: 5px; border-radius: 50%; background: var(--output); }
  code { font: 12.5px/1.4 var(--font-data); background: rgba(0, 0, 0, .06); padding: 1px 4px; border-radius: 3px; }
  .cmd { display: block; padding: 10px 12px; background: var(--surface-screen); color: var(--screen-1); border-radius: var(--r-key); overflow-wrap: anywhere; user-select: all; }
  .alarm { color: #c22a20; }
</style>
