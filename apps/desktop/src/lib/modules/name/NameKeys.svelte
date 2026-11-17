<!-- SPDX-License-Identifier: AGPL-3.0-or-later -->
<script lang="ts">
  import KeyGroup from '$lib/components/KeyGroup.svelte';
  import { recipe } from '$lib/state/recipe.svelte';
  import type { RenameMode } from '$lib/types';

  const r = $derived(recipe.value!.rename);
  const MODES = [{ id: 'keep', label: 'Keep' }, { id: 'pattern', label: 'Pattern' }, { id: 'replace', label: 'Replace' }] as { id: RenameMode; label: string }[];

  function set(m: RenameMode) {
    r.mode = m;
    if (m === 'replace' && !r.replace.length) r.replace.push({ find: '', with: '', regex: false, ignore_case: false });
  }
</script>

<KeyGroup label="Names" options={MODES} value={r.mode} accent="name" onchange={set} />
