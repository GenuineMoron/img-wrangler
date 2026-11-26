<!-- SPDX-License-Identifier: AGPL-3.0-or-later -->
<script lang="ts">
  import Field from '$lib/components/Field.svelte';
  import Key from '$lib/components/Key.svelte';
  import KeyGroup from '$lib/components/KeyGroup.svelte';
  import Row from '$lib/components/Row.svelte';
  import { recipe } from '$lib/state/recipe.svelte';
  import type { Case } from '$lib/types';
  import { TOKENS } from './index';
  import { batch } from '$lib/state/batch.svelte';

  async function load(e: Event & { currentTarget: HTMLInputElement }) {
    const file = e.currentTarget.files?.[0];
    if (file) r.list = await file.text();
    e.currentTarget.value = '';
  }

  const r = $derived(recipe.value!.rename);
  const CASES: { id: Case; label: string; aria: string }[] = [
    { id: 'keep', label: 'Aa', aria: 'Keep case' }, { id: 'lower', label: 'aa', aria: 'lower case' }, { id: 'upper', label: 'AA', aria: 'UPPER CASE' },
    { id: 'title', label: 'Title', aria: 'Title Case' }, { id: 'slug', label: 'slug', aria: 'web-safe slug' },
  ];
</script>

<div class="details" style:--accent="var(--accent-name)" style:--on-accent="var(--on-name)">
  <Row label="Case">
    <div class="case"><KeyGroup label="Letter case" options={CASES} value={r.case} accent="name" onchange={v => (r.case = v)} /></div>
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
  {#if r.mode === 'list'}
    <Row label="Names" wide note="Two columns, old name then new, matched by original name (case and extension don’t matter). Or one new name per line, given out in batch order. Files not on the list keep their name.">
      <div class="list">
        <textarea class="field area" spellcheck="false" placeholder={'DSC_4471.JPG, blue-mug-front\nDSC_4472.JPG, blue-mug-back'} aria-label="List of names" bind:value={r.list}></textarea>
        <div class="list__foot">
          <label class="key"><input type="file" accept=".csv,.tsv,.txt,text/csv,text/plain" onchange={load} /><span>Load file…</span></label>
          <span class="note">{batch.plan ? (batch.plan.unlisted ? `${batch.plan.unlisted} of ${batch.items.length} files aren’t on it` : `matches all ${batch.items.length} files`) : ''}</span>
        </div>
      </div>
    </Row>
  {/if}
  {#if r.mode === 'replace' && r.replace[0]}
    <Row label="Find as" note="A pattern is a regular expression, like IMG_(\d+).">
      <Key label="Pattern" pressed={r.replace[0].regex} onclick={() => (r.replace[0].regex = !r.replace[0].regex)} />
    </Row>
  {/if}
  {#if r.mode === 'pattern' || r.mode === 'replace'}
    <!-- In Replace, the first find and replace is typed on the Name screen. These come after it. -->
    {@const from = r.mode === 'replace' ? 1 : 0}
    <Row label={from ? 'Then replace' : 'Replace'} wide note={r.mode === 'pattern' ? 'Applied to the original name before {name} is used.' : undefined}>
      <div class="rules">
        {#each r.replace as rule, i (i)}
          {#if i >= from}
            <div class="rule">
              <Field label="Find" placeholder="find" bind:value={rule.find} />
              <Field label="Replace with" placeholder="with" bind:value={rule.with} />
              <Key label=".*" pressed={rule.regex} aria-label="Find as a pattern (regular expression)" onclick={() => (rule.regex = !rule.regex)} />
              <Key icon="x" aria-label="Remove this rule" onclick={() => r.replace.splice(i, 1)} />
            </div>
          {/if}
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
  .case { flex: 1; display: grid; }
  /* keys print in capitals; these show what the case looks like */
  .case :global(.key) { text-transform: none; letter-spacing: 0; font-size: 11.5px; }
  .list { display: grid; gap: var(--sp-4); width: 100%; }
  .area { height: 132px; padding: 8px 10px; align-items: stretch; resize: vertical; font: 12px/1.5 var(--font-data); color: var(--screen-1); border: 0; white-space: pre; overflow: auto; -webkit-user-select: text; user-select: text; }
  .area:focus-visible { outline: var(--focus); outline-offset: var(--focus-offset); }
  .list__foot { display: flex; align-items: center; gap: var(--sp-5); }
  .list__foot input { position: absolute; width: 1px; height: 1px; opacity: 0; }
  .list__foot .key:focus-within { outline: var(--focus); outline-offset: var(--focus-offset); }
  .rule { display: grid; grid-template-columns: 1fr 1fr auto auto; gap: var(--sp-3); width: 100%; }
</style>
