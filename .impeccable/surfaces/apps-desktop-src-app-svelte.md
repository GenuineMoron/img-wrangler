---
version: 1
slug: "apps-desktop-src-app-svelte"
primary_target: "apps/desktop/src/App.svelte"
related_targets: ["apps/desktop/src/views/QuickWindow.svelte", "design/system/tokens.css", "design/system/components.css"]
---

# Surface brief: the Wrangler unit (main window and quick window)

Scope: the desktop app's main window (Operate mode) and the quick-action window, which share one bundle and one design system.

Audience and job: marketers, e-commerce managers and web developers preparing hundreds to thousands of images. They drop files or folders, choose or tweak a recipe, see what will happen to every file, run it, and keep an undo.

Constraints: Tauri 2 webview (WebKitGTK on Linux: transform and opacity motion only, the sheet is virtualised, no backdrop-filter on list items), full keyboard operation, WCAG 2.2 AA, offline (fonts bundled).

Direction chosen by the user (Nov 2026, over several rounds): perfect vintage hardware with gritty personality. Teenage Engineering (OP-1, TP-7, TX-6), Braun, TR-808, XLN RC-20, Relab LX480. Then refined: cleaner and more TE, no knobs (keys and sliders), more pops of colour, everyday basics first, OP-1 screen diagrams, no screw heads, textures that blend like real grain, CRT/OLED glow on screens, and control things directly on the diagrams.

## Direction contract

THESIS: A batch image tool as a piece of hardware you'd want on your desk. Every screen is an instrument you act on directly: drag the frame's corner, drag the size limit, pick the format from the list on the screen.

OWN-WORLD: bead-blasted aluminium case with soft-light grain, scratches and dust from real scans. Light keycaps with a 2.75 px wall. Black OLED screens with a faint pixel grid, glass and bloom. Photographs are never drawn over. Host Grotesk printed on the case, Unbounded thin numerals and Departure Mono data on screens, Doto dot-matrix for the meter. One accent per channel with a glyph (■ size blue, ● format green, ▲ quality yellow, ◆ name pink). Trim teal and canvas violet inside Size. Orange only for output (Run, meter). Red only for alarms.

FIRST VIEWPORT: header (grille = Add, wordmark, recipe selector, utility keys). The big display (path and tabs, virtualised sheet, monitor with result/original/split and facts, status line). The Recipe panel listing every step in run order. The four channels. The Output panel (meter, Save to, Undo, Run).

Memorable moment: the meter's LED readout counting the saving while a run fills the OUT ladder, and the quick window doing the same from a right-click.

FINISH: tokens tuned by the user in the Tune drawer (design/prototype) are the source of truth in design/system/tokens.css. DESIGN.md documents the system.
