<!-- SPDX-License-Identifier: AGPL-3.0-or-later
  Two windows share this bundle: the main unit, and the small quick-action unit the file
  managers open (index.html#quick). -->
<script lang="ts">
  import { backend } from '$lib/ipc';
  import MainWindow from './views/MainWindow.svelte';
  import QuickWindow from './views/QuickWindow.svelte';

  const quick = location.hash === '#quick';
  document.documentElement.dataset.platform = backend.platform;
  document.documentElement.dataset.window = quick ? 'quick' : 'main';
</script>

{#if quick}<QuickWindow />{:else}<MainWindow />{/if}
