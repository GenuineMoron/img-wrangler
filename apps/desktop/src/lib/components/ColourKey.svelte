<!-- SPDX-License-Identifier: AGPL-3.0-or-later
  A colour, shown in a small well. Pressing it opens the system colour picker. `plate` fills its
  cell with the label on the left and the well on the right, like a Switch in a row of keys. -->
<script lang="ts">
  let { label, value, onchange, title, plate = false, disabled = false }: { label: string; value: string; onchange: (hex: string) => void; title?: string; plate?: boolean; disabled?: boolean } = $props();
  let input: HTMLInputElement;
</script>

<button type="button" class="colour" class:colour--plate={plate} aria-label="{label}: {value}. Press to choose a colour" {title} {disabled} onclick={() => input.click()}>
  <span class="colour__label">{label}</span>
  <span class="colour__well" style:background={value} aria-hidden="true"></span>
  <input bind:this={input} type="color" {value} tabindex="-1" aria-hidden="true" oninput={e => onchange(e.currentTarget.value)} />
</button>

<style>
  .colour { position: relative; display: inline-flex; align-items: center; gap: var(--sp-4); height: var(--key-h); padding: 0; border: 0; background: none; cursor: pointer; border-radius: var(--r-key);
    font: 600 var(--fs-key)/1.15 var(--font-ui); letter-spacing: .05em; text-transform: uppercase; text-align: left; color: var(--ink-1); }
  .colour:focus-visible { outline: var(--focus); outline-offset: var(--focus-offset); }
  .colour__label { min-width: 0; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  /* The same footprint as a switch, so a column of plates lines up. */
  .colour__well { flex: none; width: 30px; height: 16px; border-radius: 8px; box-shadow: inset 0 1px 2px rgba(0, 0, 0, .45), inset 0 0 0 1px rgba(0, 0, 0, .28), 0 1px 0 rgba(255, 255, 255, .8); }
  .colour:disabled { cursor: default; color: color-mix(in oklab, var(--ink-2) 70%, var(--surface-key)); }
  .colour:disabled .colour__well { opacity: .3; }
  .colour--plate { display: flex; width: 100%; justify-content: space-between; padding: 0 var(--sp-3) 0 var(--sp-5); background: rgba(0, 0, 0, .035); box-shadow: inset 0 0 0 1px var(--seam-dark), 0 1px 0 var(--seam-light); }
  .colour--plate .colour__label { margin-right: auto; }
  input { position: absolute; width: 1px; height: 1px; opacity: 0; pointer-events: none; }
</style>
