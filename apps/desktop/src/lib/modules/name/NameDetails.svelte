<!-- SPDX-License-Identifier: AGPL-3.0-or-later -->
<script lang="ts">
  import Field from '$lib/components/Field.svelte';
  import Key from '$lib/components/Key.svelte';
  import Row from '$lib/components/Row.svelte';
  import Switch from '$lib/components/Switch.svelte';
  import { recipe } from '$lib/state/recipe.svelte';
  import { batch } from '$lib/state/batch.svelte';

  async function load(e: Event & { currentTarget: HTMLInputElement }) {
    const file = e.currentTarget.files?.[0];
    if (file) r.list = await file.text();
    e.currentTarget.value = '';
  }

  const r = $derived(recipe.value!.rename);
</script>

<div class="details" style:--accent="var(--accent-name)" style:--on-accent="var(--on-name)">
  {#if r.mode === 'keep'}
    <p class="note">Names stay as they are. Build makes each name from pieces, Replace finds and replaces in them, and List takes them from a spreadsheet. The letter case is set under the screen.</p>
  {/if}
  {#if r.mode === 'pattern'}
    <Row label="Numbers" note="Where the number block starts, and what it counts up by. Press the block on the screen to choose its digits.">
      <Field label="First number" type="number" min="0" value={r.start} unit="from" oninput={e => (r.start = Math.max(0, Number(e.currentTarget.value) || 0))} />
      <Field label="Count by" type="number" min="1" value={r.step} unit="step" oninput={e => (r.step = Math.max(1, Number(e.currentTarget.value) || 1))} />
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
    <Switch plate label="Find as a pattern" on={r.replace[0].regex} onchange={on => (r.replace[0].regex = on)} />
    <p class="note">A pattern is a regular expression, like IMG_(\d+).</p>
  {/if}
  {#if r.mode === 'pattern' || r.mode === 'replace'}
    <!-- In Replace, the first find and replace is typed on the Name screen. These come after it. -->
    {@const from = r.mode === 'replace' ? 1 : 0}
    <Row label={from ? 'Then replace' : 'Replace'} wide note={r.mode === 'pattern' ? 'Changes made to the original name before the name block uses it.' : undefined}>
      <div class="rules">
        {#each r.replace as rule, i (i)}
          {#if i >= from}
            <div class="rule">
              <Field label="Find" placeholder="find" bind:value={rule.find} />
              <Field label="Replace with" placeholder="with" bind:value={rule.with} />
              <Switch label=".*" on={rule.regex} aria-label="Find as a pattern (regular expression)" title="Find as a pattern (regular expression)" onchange={on => (rule.regex = on)} />
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
  .rules { display: grid; gap: var(--sp-3); width: 100%; justify-items: start; }
  .list { display: grid; gap: var(--sp-4); width: 100%; }
  .area { height: 132px; padding: 8px 10px; align-items: stretch; resize: vertical; font: 12px/1.5 var(--font-data); color: var(--screen-1); border: 0; white-space: pre; overflow: auto; -webkit-user-select: text; user-select: text; }
  .area:focus-visible { outline: var(--focus); outline-offset: var(--focus-offset); }
  .list__foot { display: flex; align-items: center; gap: var(--sp-5); }
  .list__foot input { position: absolute; width: 1px; height: 1px; opacity: 0; }
  .list__foot .key:focus-within { outline: var(--focus); outline-offset: var(--focus-offset); }
  .rule { display: grid; grid-template-columns: 1fr 1fr auto auto; gap: var(--sp-3); width: 100%; }
</style>
