<!-- SPDX-License-Identifier: AGPL-3.0-or-later
  A module's further settings, opened right above its control: a channel's header, a step
  key, or the Output panel's Save line. Whichever control has id "more-<module>" opens it, so a
  new module gets this for free. Add-ons carry their on switch in the header. -->
<script lang="ts">
  import { tick } from 'svelte';
  import Switch from '$lib/components/Switch.svelte';
  import Popover from '$lib/components/Popover.svelte';
  import { addonOf, moduleById } from '$lib/modules/registry';
  import { recipe } from '$lib/state/recipe.svelte';
  import { ui } from '$lib/state/ui.svelte';

  const m = $derived(ui.more ? moduleById(ui.more) : undefined);
  const addon = $derived(m ? addonOf(recipe.value, m) : undefined);
  let trigger = $state<HTMLElement | null>(null);

  // The control is found once it's drawn. It lines up with the region it sits in.
  $effect(() => {
    const id = ui.more;
    trigger = null;
    if (id) tick().then(() => { if (ui.more === id) trigger = document.getElementById(`more-${id}`); });
  });
  const anchor = $derived(trigger ? (trigger.closest<HTMLElement>('[data-more-anchor]') ?? trigger) : null);

  function off() {
    if (addon) addon.enabled = false;
    ui.more = null;
  }
</script>

{#if m && trigger && anchor}
  {#key m.id}
    <Popover {anchor} {trigger} title={m.label} width={m.id === 'name' ? 400 : 340} onclose={() => (ui.more = null)}>
      {#snippet head()}
        {#if m.kind === 'addon'}<Switch label="On" on={!!addon?.enabled} onchange={off} title="Switch {m.label.toLowerCase()} off. Its settings are kept" />{/if}
      {/snippet}
      <div style:--accent="var(--accent-{m.accent})" style:--on-accent="var(--on-{m.accent})">
        {#if m.kind === 'addon'}
          {#if addon}<m.Details value={addon as never} />{/if}
        {:else if m.Details}
          <m.Details />
        {/if}
      </div>
    </Popover>
  {/key}
{/if}
