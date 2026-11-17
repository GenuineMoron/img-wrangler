<!-- SPDX-License-Identifier: AGPL-3.0-or-later -->
<script lang="ts">
  import Field from '$lib/components/Field.svelte';
  import Key from '$lib/components/Key.svelte';
  import KeyGroup from '$lib/components/KeyGroup.svelte';
  import Row from '$lib/components/Row.svelte';
  import { recipe } from '$lib/state/recipe.svelte';
  import type { Case } from '$lib/types';
  import { TOKENS } from './index';

  const r = $derived(recipe.value!.rename);
  const CASES: { id: Case; label: string; aria: string }[] = [
    { id: 'keep', label: 'Aa', aria: 'Keep case' }, { id: 'lower', label: 'aa', aria: 'lower case' }, { id: 'upper', label: 'AA', aria: 'UPPER CASE' },
    { id: 'title', label: 'Title', aria: 'Title Case' }, { id: 'slug', label: 'slug', aria: 'web-safe slug' },
  ];
</script>

<div class="details" style:--accent="var(--accent-name)" style:--on-accent="var(--on-name)">
  <Row label="Case">
    <KeyGroup label="Letter case" options={CASES} value={r.case} accent="name" onchange={v => (r.case = v)} />
  </Row>
  {#if r.mode === 'pattern'}
    <Row label="Numbers">
      <Field label="First number" type="number" min="0" value={r.start} unit="from" oninput={e => (r.start = Math.max(0, Number(e.currentTarget.value) || 0))} />
      <Field label="Count by" type="number" min="1" value={r.step} unit="step" oninput={e => (r.step = Math.max(1, Number(e.currentTarget.value) || 1))} />
    </Row>
    <Row label="Tokens" wide>
      <dl class="tokens">
        {#each TOKENS as t (t.token)}<dt>{t.token}</dt><dd>{t.means}</dd>{/each}
      </dl>
    </Row>
  {/if}
  {#if r.mode !== 'keep'}
    <Row label="Replace" wide note={r.mode === 'pattern' ? 'Applied to the original name before {name} is used.' : undefined}>
      <div class="rules">
        {#each r.replace as rule, i (i)}
          <div class="rule">
            <Field label="Find" placeholder="find" bind:value={rule.find} />
            <Field label="Replace with" placeholder="with" bind:value={rule.with} />
            <Key label=".*" pressed={rule.regex} aria-label="Regular expression" onclick={() => (rule.regex = !rule.regex)} />
            <Key icon="x" aria-label="Remove this rule" onclick={() => r.replace.splice(i, 1)} />
          </div>
        {/each}
        <Key insert label="Rule" onclick={() => r.replace.push({ find: '', with: '', regex: false, ignore_case: false })} />
      </div>
    </Row>
  {/if}
</div>

<style>
  .tokens { display: grid; grid-template-columns: max-content 1fr; gap: 2px 10px; font: var(--fs-small)/1.4 var(--font-data); margin: 0; }
  .tokens dt { color: var(--ink-1); }
  .tokens dd { margin: 0; color: var(--ink-2); }
  .rules { display: grid; gap: var(--sp-3); width: 100%; justify-items: start; }
  .rule { display: grid; grid-template-columns: 1fr 1fr auto auto; gap: var(--sp-3); width: 100%; }
</style>
