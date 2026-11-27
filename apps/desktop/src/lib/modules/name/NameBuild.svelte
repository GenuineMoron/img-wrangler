<!-- SPDX-License-Identifier: AGPL-3.0-or-later
  A name built from pieces, on the Name screen. Each piece is a block: the original name, a
  number that counts up, a date, the size, the folder, or text of your own. Press a piece in the
  tray to add it, drag blocks to reorder them, press one for its options, drag one away to
  remove it. The blocks are the recipe's pattern (see pattern.ts), so the name updates as you go. -->
<script lang="ts">
  import { backend } from '$lib/ipc';
  import { recipe } from '$lib/state/recipe.svelte';
  import type { MenuLine } from '$lib/types';
  import { describe, label, move, parse, print, text, token, type Piece } from './pattern';

  const r = $derived(recipe.value!.rename);
  // The working copy. It can hold a text block that's still empty, which the pattern can't.
  let pieces = $state<Piece[]>([]);
  $effect(() => { if (print(pieces) !== r.pattern) pieces = parse(r.pattern); });
  const commit = () => { r.pattern = print(pieces); };

  /** The text block being typed in, and the block new pieces go after. */
  let editing = $state<number | null>(null);
  let chosen = $state<number | null>(null);
  let row: HTMLElement;

  /** What the tray offers. Size is three blocks: width, an x, height. */
  const TRAY: { add: string; make: () => Piece[]; means: string }[] = [
    { add: 'name', make: () => [token('name')], means: 'The original name' },
    { add: '001', make: () => [token('n')], means: 'A number that counts up' },
    { add: 'date', make: () => [token('date')], means: 'The file’s date, or the date the photo was taken' },
    { add: 'size', make: () => [token('w'), text('x'), token('h')], means: 'The new size, like 1200x800' },
    { add: 'folder', make: () => [token('folder')], means: 'The name of the folder the image is in' },
    { add: 'text', make: () => [text('')], means: 'Text of your own' },
  ];

  function insert(made: Piece[], at = chosen != null && chosen < pieces.length ? chosen + 1 : pieces.length) {
    // Two blocks side by side run together, so a new one gets a dash before it.
    const dash = made[0].kind === 'token' && pieces[at - 1]?.kind === 'token' ? [text('-')] : [];
    pieces.splice(at, 0, ...dash, ...made);
    chosen = at + dash.length + made.length - 1;
    if (made[0].kind === 'text' && !made[0].text) editing = chosen;
    commit();
  }

  function remove(i: number) {
    pieces.splice(i, 1);
    chosen = null;
    commit();
  }

  function options(i: number) {
    const p = pieces[i];
    if (p.kind === 'text') { editing = i; return; }
    const set = (change: Partial<Extract<Piece, { kind: 'token' }>>) => () => { Object.assign(p, change); commit(); };
    const lines: MenuLine[] =
      p.key === 'n' ? ['1, 2, 3', '01, 02, 03', '001, 002, 003', '0001, 0002', '00001, 00002'].map((t, d) => ({ text: t, checked: Math.max(1, p.pad) === d + 1, action: set({ pad: d + 1 }) }))
      : p.key === 'name' ? [
          { text: 'As it is', checked: !p.filter, action: set({ filter: null }) },
          { text: 'Lower case with dashes', checked: p.filter === 'slug', action: set({ filter: 'slug' }) },
          { text: 'lower case', checked: p.filter === 'lower', action: set({ filter: 'lower' }) },
          { text: 'UPPER CASE', checked: p.filter === 'upper', action: set({ filter: 'upper' }) },
        ]
      : p.key === 'date' || p.key === 'taken' ? [
          { text: 'The file’s date', checked: p.key === 'date', action: set({ key: 'date' }) },
          { text: 'The date the photo was taken', checked: p.key === 'taken', action: set({ key: 'taken' }) },
        ]
      : [];
    backend.contextMenu([...lines, ...(lines.length ? [null] : []), { text: 'Remove', action: () => remove(i) }]);
  }

  // ---------- dragging: a block already in the name, or a new one from the tray ----------
  let drag = $state<{ piece: Piece; from: number | null; x: number; y: number; at: number | null } | null>(null);
  let dragged = false;

  /** Where a block dropped here would go: before the first block the pointer is ahead of. */
  function slotAt(x: number, y: number): number {
    const blocks = [...row.querySelectorAll<HTMLElement>('[data-i]')];
    for (const b of blocks) {
      const box = b.getBoundingClientRect();
      if (y < box.top || (y <= box.bottom && x < box.left + box.width / 2)) return Number(b.dataset.i);
    }
    return pieces.length;
  }

  function down(e: PointerEvent, from: number | null, made?: () => Piece[]) {
    if (e.button !== 0) return;
    const start = { x: e.clientX, y: e.clientY };
    dragged = false;
    const moved = (ev: PointerEvent) => {
      if (!dragged && Math.hypot(ev.clientX - start.x, ev.clientY - start.y) < 5) return;
      dragged = true;
      const box = row.getBoundingClientRect();
      const near = ev.clientX > box.left - 20 && ev.clientX < box.right + 20 && ev.clientY > box.top - 20 && ev.clientY < box.bottom + 20;
      drag = { piece: from != null ? pieces[from] : made!()[0], from, x: ev.clientX, y: ev.clientY, at: near ? slotAt(ev.clientX, ev.clientY) : null };
    };
    const up = () => {
      removeEventListener('pointermove', moved);
      removeEventListener('pointerup', up);
      removeEventListener('pointercancel', up);
      const d = drag;
      drag = null;
      if (!d) return;
      if (d.from != null && d.at == null) remove(d.from); // dragged away
      else if (d.from != null) { pieces = move(pieces, d.from, d.at!); commit(); }
      else if (d.at != null) { chosen = null; insert(made!(), d.at); }
    };
    addEventListener('pointermove', moved);
    addEventListener('pointerup', up);
    addEventListener('pointercancel', up);
  }

  /** A press that wasn't a drag. */
  const press = (act: () => void) => () => { if (!dragged) act(); };

  function keys(e: KeyboardEvent, i: number) {
    if (e.key === 'Delete' || e.key === 'Backspace') { e.preventDefault(); remove(i); }
    const step = e.altKey ? { ArrowLeft: -1, ArrowRight: 2 }[e.key] : undefined;
    if (step && i + step >= 0 && i + step <= pieces.length) {
      e.preventDefault();
      pieces = move(pieces, i, i + step);
      commit();
      const to = step < 0 ? i - 1 : i + 1;
      queueMicrotask(() => row.querySelector<HTMLElement>(`[data-i="${to}"]`)?.focus());
    }
  }

  function typed(e: Event & { currentTarget: HTMLInputElement }, i: number) {
    (pieces[i] as Extract<Piece, { kind: 'text' }>).text = e.currentTarget.value;
    commit();
  }

  function doneTyping(i: number) {
    editing = null;
    if (pieces[i]?.kind === 'text' && !pieces[i].text) remove(i);
  }

  const focus = (el: HTMLInputElement) => { el.focus(); el.select(); };
</script>

<div bind:this={row} class="build" class:dropping={drag?.at != null} role="group" aria-label="The new name, piece by piece">
  {#each pieces as p, i (i)}
    {#if drag?.at === i}<i class="mark" aria-hidden="true"></i>{/if}
    {#if p.kind === 'text' && editing === i}
      <input class="typing" aria-label="Text in the name" spellcheck="false" autocomplete="off" value={p.text} style:width="{Math.max(2, p.text.length + 1)}ch"
        use:focus oninput={e => typed(e, i)} onblur={() => doneTyping(i)} onkeydown={e => (e.key === 'Enter' || e.key === 'Escape') && e.currentTarget.blur()} />
    {:else}
      <button type="button" class="block" class:block--text={p.kind === 'text'} class:lifted={drag?.from === i} data-i={i}
        aria-label="{describe(p)}. Press {p.kind === 'text' ? 'to change it' : 'for options'}, Delete removes it, Alt and the arrow keys move it"
        title={describe(p)}
        onpointerdown={e => down(e, i)} onclick={press(() => options(i))} onfocus={() => (chosen = i)} onkeydown={e => keys(e, i)}>{label(p)}</button>
    {/if}
  {/each}
  {#if drag?.at === pieces.length}<i class="mark" aria-hidden="true"></i>{/if}
  {#if !pieces.length && !drag}<span class="empty">add pieces from below</span>{/if}
</div>

<div class="tray" role="group" aria-label="Add a piece to the name">
  {#each TRAY as t (t.add)}
    <button type="button" title="{t.means}. Press to add it, or drag it into place" aria-label="Add: {t.means}"
      onpointerdown={e => down(e, null, t.make)} onclick={press(() => insert(t.make()))}>{t.add}</button>
  {/each}
</div>

{#if drag}
  <span class="ghost" class:ghost--away={drag.at == null && drag.from != null} style:left="{drag.x}px" style:top="{drag.y}px" aria-hidden="true">{label(drag.piece) || 'text'}</span>
{/if}

<style>
  /* Two lines of blocks, then it scrolls: the new name and the tray below always stay in view. */
  .build { display: flex; flex-wrap: wrap; align-items: center; align-content: flex-start; gap: 3px; min-height: 26px; max-height: 47px; overflow-y: auto; scrollbar-width: thin; scrollbar-color: var(--screen-dim) transparent;
    padding-bottom: 5px; border-bottom: 1px solid var(--screen-line); }
  .build.dropping { border-bottom-color: var(--accent-name); }
  .block, .typing, .tray button, .ghost { font: 11.5px/18px var(--font-data); border-radius: 3px; white-space: pre; }
  .block { all: unset; cursor: grab; touch-action: none; padding: 0 5px; max-width: 100%; overflow: hidden; text-overflow: ellipsis; box-sizing: border-box;
    font: 11.5px/18px var(--font-data); border-radius: 3px; white-space: pre; color: var(--on-name); background: var(--accent-name); }
  /* Your own text: an outline, not a filled block, so it reads as what you typed. */
  .block--text { cursor: text; color: var(--screen-1); background: none; box-shadow: inset 0 0 0 1px var(--screen-dim); }
  .block:hover { filter: brightness(1.12); }
  .block--text:hover { box-shadow: inset 0 0 0 1px var(--screen-3); filter: none; }
  .block:focus-visible { outline: var(--focus-on-screen); outline-offset: 1px; }
  .block.lifted { opacity: .3; }
  .typing { all: unset; box-sizing: content-box; min-width: 2ch; max-width: calc(100% - 10px); padding: 0 5px; font: 11.5px/18px var(--font-data); border-radius: 3px; color: var(--screen-1);
    box-shadow: inset 0 0 0 1px var(--accent-name); }
  .mark { width: 2px; height: 18px; margin: 0 -2.5px; border-radius: 1px; background: var(--screen-1); box-shadow: 0 0 6px var(--screen-1); flex: none; }
  .empty { color: var(--screen-3); font-size: var(--fs-small); }

  .tray { order: 3; display: flex; flex-wrap: wrap; gap: 3px; }
  .tray button { all: unset; cursor: grab; touch-action: none; padding: 0 4px; font: 10.5px/16px var(--font-data); border-radius: 3px; color: var(--accent-name);
    box-shadow: inset 0 0 0 1px color-mix(in srgb, var(--accent-name) 50%, transparent); }
  .tray button::before { content: "+"; opacity: .7; margin-right: 1px; }
  .tray button:hover { background: color-mix(in srgb, var(--accent-name) 18%, transparent); }
  .tray button:focus-visible { outline: var(--focus-on-screen); }

  .ghost { position: fixed; z-index: 60; pointer-events: none; transform: translate(-50%, -130%); padding: 0 5px; color: var(--on-name); background: var(--accent-name);
    box-shadow: 0 4px 12px rgba(0, 0, 0, .5); }
  .ghost--away { opacity: .5; text-decoration: line-through; }
</style>
