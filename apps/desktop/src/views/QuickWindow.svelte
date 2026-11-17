<!-- SPDX-License-Identifier: AGPL-3.0-or-later
  The small unit a file manager's right-click opens: runs a pinned recipe on the selection at
  once, shows where the files went and what was saved, with Undo one press away. -->
<script lang="ts">
  import { onMount } from 'svelte';
  import Key from '$lib/components/Key.svelte';
  import Ladder from '$lib/components/Ladder.svelte';
  import Led from '$lib/components/Led.svelte';
  import Screen from '$lib/components/Screen.svelte';
  import { backend } from '$lib/ipc';
  import { bytes, change, count } from '$lib/text';
  import type { QuickResult, Summary } from '$lib/types';

  let paths = $state<string[]>([]);
  let name = $state('');
  let total = $state(0);
  let done = $state(0);
  let bytesIn = $state(0);
  let bytesOut = $state(0);
  let result = $state<QuickResult | null>(null);
  let error = $state<string | null>(null);
  let undone = $state(false);
  let running = $state(false);

  const s = $derived<Summary | null>(result?.summary ?? null);
  const where = $derived.by(() => {
    const d = result?.recipe.destination;
    if (!d) return '';
    return { beside: 'saved beside the originals', subfolder: `saved into ${d.folder || 'Wrangled'}/`, folder: 'saved into your folder', replace: 'replaced in place' }[d.mode];
  });

  async function go(recipe: string, p: string[]) {
    paths = p;
    total = done = bytesIn = bytesOut = 0;
    result = null;
    error = null;
    undone = false;
    running = true;
    name = recipe.replace(/-/g, ' ');
    try {
      result = await backend.quickRun(recipe, p, e => {
        if (e.type === 'started') total = e.total;
        else if (e.type === 'item') { done++; bytesIn += e.bytes_in; bytesOut += e.bytes_out; }
      });
      name = result.recipe.name;
    } catch (e) {
      error = String(e);
    } finally {
      running = false;
    }
  }

  onMount(() => {
    let off: (() => void) | undefined;
    // Another right-click while this one runs waits its turn, then runs here too.
    const take = async () => {
      if (running) return;
      const l = await backend.takeLaunch('quick');
      if (l?.mode === 'quick') { await go(l.recipe, l.paths); take(); }
    };
    // Explorer starts one process per selected file; give them a moment to fold into one batch.
    setTimeout(take, 250);
    backend.onLaunch(take).then(f => (off = f));
    backend.ready();
    const esc = (e: KeyboardEvent) => e.key === 'Escape' && !running && backend.close();
    addEventListener('keydown', esc);
    return () => { off?.(); removeEventListener('keydown', esc); };
  });

  async function undo() {
    if (!s) return;
    const r = await backend.undo(s.run_id);
    undone = true;
    error = r.skipped.length ? `${r.skipped.length} changed since and left alone` : null;
  }
</script>

<main class="qa mat-case" data-tauri-drag-region>
  <div class="top" data-tauri-drag-region>
    <span class="legend">wrangler · quick</span>
    <Led on color={error ? 'var(--status-alarm)' : running ? 'var(--output)' : 'var(--accent-format)'} />
  </div>
  <Screen class="qs">
    <div class="row"><span>{name.toUpperCase()}</span><b>{count(done)}/{count(total || done)}</b></div>
    <Ladder lit={total ? (24 * done) / total : 0} kind="out" />
    <span class="led-readout" role="status">{bytesIn ? change(bytesIn, bytesOut) : running ? '…' : '−−%'}</span>
    <div class="row"><span>{bytes(bytesIn)} → {bytes(bytesOut)}</span><b>{s ? `${s.seconds.toFixed(1)} s` : ''}</b></div>
    <div class="row">
      <span>{error ?? (undone ? 'undone, originals back' : running ? 'working…' : where)}</span>
      <span>{s?.kept ? `${s.kept} kept` : ''}{s?.failed ? ` · ${s.failed} failed` : ''}</span>
    </div>
  </Screen>
  <div class="keys" style:--n="3">
    <Key icon="undo" label="Undo" disabled={!s || undone || running} onclick={undo} />
    <Key icon="reveal" label="Show" disabled={!result?.first_output || undone} onclick={() => result?.first_output && backend.reveal(result.first_output)} />
    <Key dark label="Open" onclick={async () => { await backend.openMain(paths); backend.close(); }} title="Open these in the main window" />
  </div>
</main>

<style>
  :global(html[data-window='quick']), :global(html[data-window='quick'] body) { background: var(--surface-case); }
  .qa { height: 100vh; padding: 16px; display: grid; grid-template-rows: auto 1fr auto; gap: 12px; color: var(--ink-1); }
  .top { display: flex; justify-content: space-between; align-items: center; }
  .qa :global(.qs) { padding: 12px 14px; display: grid; gap: 8px; align-content: center; }
  .row { display: flex; justify-content: space-between; gap: 8px; color: var(--screen-2); }
  .row b { color: var(--screen-1); font-weight: 400; }
  .row span:first-child { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .led-readout { font-size: var(--fs-led-lg); }
</style>
