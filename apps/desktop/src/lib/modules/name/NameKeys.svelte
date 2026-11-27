<!-- SPDX-License-Identifier: AGPL-3.0-or-later
  Under the Name screen: how names are made, and the letter case of the result. -->
<script lang="ts">
  import Segmented from '$lib/components/Segmented.svelte';
  import { recipe } from '$lib/state/recipe.svelte';
  import { ui } from '$lib/state/ui.svelte';
  import type { Case, RenameMode } from '$lib/types';

  const r = $derived(recipe.value!.rename);
  const MODES: { id: RenameMode; label: string; aria: string; title: string }[] = [
    { id: 'keep', label: 'Keep', aria: 'Keep the names', title: 'Names stay as they are' },
    { id: 'pattern', label: 'Build', aria: 'Build names from pieces', title: 'Build each name from pieces: the old name, a number, a date, your own text' },
    { id: 'replace', label: 'Replace', aria: 'Find and replace in the names', title: 'Find and replace in the names' },
    { id: 'list', label: 'List', aria: 'Names from a list or spreadsheet', title: 'Names from a list or a spreadsheet' },
  ];
  const CASES: { id: Case; label: string; aria: string; title: string }[] = [
    { id: 'keep', label: 'Aa', aria: 'Letters as they are', title: 'Letters as they are' },
    { id: 'lower', label: 'aa', aria: 'lower case', title: 'all lower case' },
    { id: 'upper', label: 'AA', aria: 'UPPER CASE', title: 'ALL UPPER CASE' },
    { id: 'title', label: 'Ab Cd', aria: 'Title Case', title: 'Each Word Capitalised' },
    { id: 'slug', label: 'a-b', aria: 'Web safe: lower case with dashes', title: 'web-safe: lower case, with dashes for spaces' },
  ];

  function set(m: RenameMode) {
    r.mode = m;
    if (m === 'pattern' && !r.pattern) r.pattern = '{name}';
    if (m === 'replace' && !r.replace.length) r.replace.push({ find: '', with: '', regex: false, ignore_case: false });
    if (m === 'list') ui.more = 'name';
  }
</script>

<Segmented label="How names are made" options={MODES} value={r.mode} accent="name" onchange={set} />
<Segmented label="Letter case" class="case" options={CASES} value={r.case} accent="name" onchange={v => (r.case = v)} />

<style>
  /* The case examples are shown as written, not in capitals. */
  :global(.seg.case > button) { text-transform: none; letter-spacing: 0; font-size: 11px; }
</style>
