<!-- SPDX-License-Identifier: AGPL-3.0-or-later -->
<script lang="ts">
  import Field from '$lib/components/Field.svelte';
  import Switch from '$lib/components/Switch.svelte';
  import Row from '$lib/components/Row.svelte';
  import { recipe } from '$lib/state/recipe.svelte';

  const s = $derived(recipe.value!.select);
  const list = (text: string) => text.split(/[\s,]+/).map(t => t.trim()).filter(Boolean);
</script>

<div class="details">
  <Switch plate label="Look in subfolders too" on={s.recursive} onchange={on => (s.recursive = on)} />
  <Row label="Only" note="Name patterns, e.g. *.jpg IMG_*">
    <Field label="Only names matching" placeholder="every image" value={s.include.join(' ')} onchange={e => (s.include = list(e.currentTarget.value))} />
  </Row>
  <Row label="Skip">
    <Field label="Skip names matching" placeholder="nothing" value={s.exclude.join(' ')} onchange={e => (s.exclude = list(e.currentTarget.value))} />
  </Row>
  <Row label="Smaller than">
    <Field label="Skip files smaller than, in kilobytes" type="number" min="0" placeholder="none" unit="KB" value={s.min_kb ?? ''}
      oninput={e => (s.min_kb = Number(e.currentTarget.value) || null)} />
  </Row>
</div>
