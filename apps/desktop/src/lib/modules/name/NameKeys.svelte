<!-- SPDX-License-Identifier: AGPL-3.0-or-later -->
<script lang="ts">
  import KeyGroup from '$lib/components/KeyGroup.svelte';
  import { recipe } from '$lib/state/recipe.svelte';
  import { ui } from '$lib/state/ui.svelte';
  import type { RenameMode } from '$lib/types';

  const r = $derived(recipe.value!.rename);
  const MODES = [
    { id: 'keep', label: 'Keep' }, { id: 'pattern', label: 'Pattern' }, { id: 'replace', label: 'Replace' },
    { id: 'list', label: 'List', aria: 'From a list or spreadsheet' },
  ] as { id: RenameMode; label: string; aria?: string }[];

  function set(m: RenameMode) {
    r.mode = m;
    if (m === 'replace' && !r.replace.length) r.replace.push({ find: '', with: '', regex: false, ignore_case: false });
    if (m === 'list') ui.more = 'name';
  }
</script>

<KeyGroup label="Names" options={MODES} value={r.mode} accent="name" onchange={set} />
