<!-- SPDX-License-Identifier: AGPL-3.0-or-later
  A modal panel on the native <dialog>, so focus, Escape and the backdrop behave as the OS does. -->
<script lang="ts">
  import type { Snippet } from 'svelte';
  import Key from './Key.svelte';

  let { open, onclose, title, wide = false, children, actions }: {
    open: boolean;
    onclose: () => void;
    title: string;
    wide?: boolean;
    children: Snippet;
    actions?: Snippet;
  } = $props();

  let el: HTMLDialogElement;
  $effect(() => {
    if (open && !el.open) el.showModal();
    else if (!open && el.open) el.close();
  });
</script>

<dialog bind:this={el} class="dialog mat-case" class:dialog--wide={wide} aria-label={title} onclose={onclose} onclick={e => e.target === el && onclose()}>
  <header class="dialog__head">
    <h2 class="legend">{title}</h2>
    <Key icon="x" aria-label="Close" onclick={onclose} />
  </header>
  <div class="dialog__body">{@render children()}</div>
  {#if actions}<footer class="dialog__foot">{@render actions()}</footer>{/if}
</dialog>
