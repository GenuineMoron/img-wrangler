<!-- SPDX-License-Identifier: AGPL-3.0-or-later
  A panel that opens right above the control it belongs to, for that control's further
  settings. It stays while you look around the display (to see a change on another image), and
  closes on Escape, its close key, or a press anywhere else. -->
<script lang="ts">
  import { onMount, type Snippet } from 'svelte';
  import Key from './Key.svelte';

  let { anchor, trigger, title, onclose, children, head, width = 340 }: {
    /** What it sits above and lines up with. */
    anchor: HTMLElement;
    /** The control that opened it: pressing that again closes it, and focus returns there. */
    trigger: HTMLElement;
    title: string;
    onclose: () => void;
    children: Snippet;
    /** Extra controls for the header, beside the title. */
    head?: Snippet;
    width?: number;
  } = $props();

  let el: HTMLDivElement;
  let pos = $state({ left: 8, bottom: 8, width: 340, maxHeight: 400 });

  function place() {
    const a = anchor.getBoundingClientRect();
    const w = Math.min(width, innerWidth - 16);
    pos = { left: Math.max(8, Math.min(a.left, innerWidth - w - 8)), bottom: innerHeight - a.top + 8, width: w, maxHeight: Math.max(160, a.top - 24) };
  }
  $effect(() => { void anchor; void width; place(); });

  onMount(() => {
    const outside = (e: PointerEvent) => {
      const t = e.target as Node;
      if (el.contains(t) || trigger.contains(t) || (t instanceof Element && t.closest('.display'))) return;
      onclose();
    };
    const key = (e: KeyboardEvent) => {
      if (e.key === 'Escape') { e.stopPropagation(); onclose(); }
    };
    addEventListener('resize', place);
    addEventListener('pointerdown', outside, true);
    addEventListener('keydown', key, true);
    el.querySelector<HTMLElement>('.popover__body button, .popover__body input, .popover__body textarea, .popover__body [tabindex="0"]')?.focus({ preventScroll: true });
    return () => {
      removeEventListener('resize', place);
      removeEventListener('pointerdown', outside, true);
      removeEventListener('keydown', key, true);
      if (el.contains(document.activeElement)) trigger.focus({ preventScroll: true });
    };
  });
</script>

<div bind:this={el} class="popover mat-case" role="dialog" aria-label={title}
  style:left="{pos.left}px" style:bottom="{pos.bottom}px" style:width="{pos.width}px" style:max-height="{pos.maxHeight}px">
  <header class="popover__head">
    <h2 class="legend">{title}</h2>
    <span class="popover__extra">{@render head?.()}</span>
    <Key icon="x" aria-label="Close" onclick={onclose} />
  </header>
  <div class="popover__body">{@render children()}</div>
</div>

<style>
  .popover { position: fixed; z-index: 30; display: grid; grid-template-rows: auto minmax(0, 1fr); border-radius: 14px; color: var(--ink-1);
    box-shadow: 0 0 0 1px rgba(0, 0, 0, .28), 0 18px 44px rgba(0, 0, 0, .42), inset 0 1px 0 rgba(255, 255, 255, .9); }
  :global(html.lite) .popover { box-shadow: 0 0 0 1px rgba(0, 0, 0, .45), inset 0 1px 0 rgba(255, 255, 255, .9); }
  .popover__head { display: flex; align-items: center; gap: var(--sp-5); padding: 10px 10px 6px 16px; }
  .popover__head h2 { margin-right: auto; }
  .popover__extra { display: flex; gap: var(--sp-3); }
  .popover__body { overflow-y: auto; padding: 0 16px 14px; scrollbar-width: thin; }
</style>
