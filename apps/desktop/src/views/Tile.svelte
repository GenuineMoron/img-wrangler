<!-- SPDX-License-Identifier: AGPL-3.0-or-later
  One image in the sheet, as it will come out: new name, size before and after, and a flag when
  something needs a look. -->
<script lang="ts">
  import { backend } from '$lib/ipc';
  import { batch, type ItemStatus } from '$lib/state/batch.svelte';
  import { change, fileName, kb } from '$lib/text';
  import type { PlanItem } from '$lib/types';

  let { item, status, selected, index, total, onselect, left, top, width }: {
    item: PlanItem; status: ItemStatus; selected: boolean; index: number; total: number; onselect: () => void; left: number; top: number; width: number;
  } = $props();

  const FLAGS: Partial<Record<ItemStatus, { text: string; alarm: boolean; say: string }>> = {
    clash: { text: '! CLASH', alarm: true, say: 'name taken, numbered' },
    over: { text: '! OVER', alarm: true, say: 'over the size limit' },
    failed: { text: '! FAILED', alarm: true, say: 'failed' },
    kept: { text: '= KEPT', alarm: false, say: 'original kept, already smaller' },
    skipped: { text: '- SKIP', alarm: false, say: 'skipped' },
    done: { text: '✓ DONE', alarm: false, say: 'done' },
  };
  const flag = $derived(FLAGS[status]);
  const inB = $derived(item.probe?.bytes ?? 0);
  const out = $derived(batch.outBytes(item));
  const reason = $derived(item.action.kind === 'skip' ? item.action.reason : batch.reports[item.id]?.message);
</script>

<button
  type="button"
  class="tile"
  role="option"
  id="tile-{item.id}"
  aria-selected={selected}
  aria-posinset={index + 1}
  aria-setsize={total}
  tabindex={selected ? 0 : -1}
  style:left="{left}px"
  style:top="{top}px"
  style:width="{width}px"
  title={reason ?? undefined}
  aria-label="{fileName(item.output)}, {kb(inB)}{out != null ? ` to ${kb(out)}` : ''}{flag ? `, ${flag.say}` : ''}"
  onclick={onselect}
>
  <div class="tile__img">
    <img src={backend.thumbUrl(item.source)} alt="" loading="lazy" decoding="async" draggable="false" />
    {#if flag}<span class="flag" class:flag--alarm={flag.alarm}>{flag.text}</span>{/if}
  </div>
  <div class="tile__name">{fileName(item.output)}</div>
  <div class="tile__meta">
    <span>{kb(inB)} → <b>{out != null ? kb(out) : '…'}</b></span>
    <span class="pct">{status === 'kept' ? 'kept' : out != null ? change(inB, out) : ''}</span>
  </div>
</button>

<style>
  .tile { position: absolute; }
  .pct { color: var(--screen-1); }
</style>
