<!-- SPDX-License-Identifier: AGPL-3.0-or-later
  Under the Size screen: the shape, and whether images smaller than the size are enlarged.
  "W:H" opens the fields for any other ratio or an exact size in pixels. -->
<script lang="ts">
  import Segmented from '$lib/components/Segmented.svelte';
  import Switch from '$lib/components/Switch.svelte';
  import { recipe } from '$lib/state/recipe.svelte';
  import { ui } from '$lib/state/ui.svelte';
  import { SHAPES } from './index';

  const OTHER = 'other';
  const size = $derived(recipe.value!.size);
  const listed = $derived(SHAPES.some(s => s.id === size.shape));
  /** The W:H place shows the shape it holds, when that's short enough to print there. */
  const other = $derived({
    id: OTHER,
    label: !listed && size.shape.length <= 4 ? size.shape : 'W:H',
    aria: listed ? 'Another shape: any ratio, or an exact size in pixels' : `Another shape, now ${size.shape.replace('x', ' by ')}`,
    title: 'Any other ratio, or an exact size in pixels',
  });

  function choose(v: string) {
    if (v === OTHER) ui.more = 'size';
    else size.shape = v;
  }
</script>

<Segmented label="Shape" options={[...SHAPES, other]} value={listed ? size.shape : OTHER} accent="size" onchange={choose} />
<Switch plate label="Enlarge smaller images" on={size.upscale} onchange={on => (size.upscale = on)}
  title="Images smaller than the size are made bigger to match. Off, they stay as they are" />
