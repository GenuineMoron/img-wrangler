<!-- SPDX-License-Identifier: AGPL-3.0-or-later
  A key showing a colour swatch; pressing it opens the system colour picker. -->
<script lang="ts">
  let { label, value, onchange, checked }: { label: string; value: string; onchange: (hex: string) => void; checked?: boolean } = $props();
  let input: HTMLInputElement;
</script>

<button type="button" class="key" role={checked !== undefined ? 'radio' : undefined} aria-checked={checked} aria-label="{label}, now {value}" onclick={() => input.click()}>
  <span class="swatch" style:background={value}></span><span>{label}</span>
  <input bind:this={input} type="color" {value} tabindex="-1" aria-hidden="true" oninput={e => onchange(e.currentTarget.value)} />
</button>

<style>
  .swatch { width: 11px; height: 11px; border-radius: 2px; box-shadow: inset 0 0 0 1px rgba(0, 0, 0, .3); flex: none; }
  input { position: absolute; width: 1px; height: 1px; opacity: 0; pointer-events: none; }
  button { position: relative; }
</style>
