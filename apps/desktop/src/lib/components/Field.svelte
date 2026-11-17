<!-- SPDX-License-Identifier: AGPL-3.0-or-later
  A text field set into the case like a small screen. -->
<script lang="ts">
  import type { HTMLInputAttributes } from 'svelte/elements';

  let { value = $bindable(''), label, unit, el = $bindable(), ...rest }: HTMLInputAttributes & { value?: string | number; label: string; unit?: string; el?: HTMLInputElement } = $props();
</script>

<label class="field">
  <span class="sr-only">{label}</span>
  <input bind:this={el} bind:value spellcheck="false" autocomplete="off" {...rest} />
  {#if unit}<span class="field__unit" aria-hidden="true">{unit}</span>{/if}
</label>
