---
version: 1
slug: "app-src-app-svelte"
primary_target: "app/src/App.svelte"
related_targets: []
---

# Surface brief: main window

Scope: the desktop app's main window (Operate mode), plus its quick-action panel, compare view and settings, which inherit this world.

Audience and job: marketers, e-commerce managers and web developers preparing hundreds to thousands of images. They drop files or folders, pick or tweak a recipe, check what will happen to every file, run it, and keep an undo.

Constraints: Tauri 2 webview (WebKitGTK on Linux, so keep the grid cheap: transform/opacity motion only, no backdrop-filter on list items), follows OS light/dark, full keyboard operation, WCAG 2.2 AA.

Direction chosen: Shelf ticket (assigned by the roll; user dismissed the pick, proceeding unattended with the assigned direction, switchable later through the token layer).

## Direction contract

THESIS: The batch reads like a price list: every image shows what it was, what it will be and what you save, before anything is written. Refuses the category default of a grey pro tool with a blue Run button and savings buried in a status bar.

OWN-WORLD: Australian retail shelf tickets. Warm paper ground and white panels so photographs read true, near-black ink, ticket yellow owning the Run bar, savings tags and the active step; the SAVE figure inverted black-on-yellow. Archivo variable throughout: UI at normal width, ticket numerals at width 62, weight 900, tabular. Flat colour only. Red means failed, amber means kept original or needs attention, an ink outline means selected.

STORY: Drop a folder, watch thumbnails arrive, choose a recipe, read the batch ticket (was, now, save, kept originals, name clashes), run it, get a receipt with undo.

FIRST VIEWPORT: Top bar with app name, source summary and filter field with the command-palette shortcut. Centre: virtualised grid of tiles, each with image, new name, old to new size and a yellow minus-percent tag. Right: 340 px recipe panel with recipe name, numbered step cards, add step, output mode. Bottom: 92 px ticket-yellow Run bar with WAS struck through, NOW in large condensed black numerals, the inverted SAVE box, counts, and the black Run button at the right.

FORM: Grounded candidate 3 of 7 (retail shelf ticket), dealt by the roll. Raises: from ikeda-datamatics, every number in tabular figures, right-aligned in columns; from aww2-zoo-gardens-guide-map, flat colour commitment, no gradients or glass; from notation-diagram-systems-star-atlas, one fixed savings scale used everywhere; from vernacular-ephemera-cassette-j-card, size budgets drawn as a limit line with overflow flagged; from editorial-sequence-montage-botanical-folio-season, before and after always registered at identical scale and position. seed:85d54fe6409d5718

FINISH: unreviewed and undocumented is unfinished; this build ends with the finish review, the verdict, DESIGN.md, and every shipping raster carrying its provenance

Memorable moment: the batch ticket counting down live while a run progresses, then becoming the receipt.

Unresolved: none blocking. App icon and wordmark still to be drawn.
