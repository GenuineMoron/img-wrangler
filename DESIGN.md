# Design

Wrangler looks and behaves like a piece of hardware: an aluminium unit with keys, sliders and small black screens, in the line of Teenage Engineering, Braun and the TR-808. It is an Operate surface, so the character lives in materials, type and precise details, while the layout stays plain and every control does one obvious thing.

The system lives in [`design/system`](design/system) and the app imports it directly, so there is one copy of every value. Tokens marked *tuned* were set by hand in the prototype's Tune drawer and are the reference.

## Principles
1. **The screen is the control.** A diagram you can only look at is a missed control. Drag the Size frame's corner, drag the red size-limit line, pick a format from the list on the screen, type the number. Every one of these has a keyboard path too.
2. **Everyday first.** Resize, reformat, compress and rename are the four channels, always visible. Everything else is one step deeper, in the Recipe panel.
3. **Show the result, not the setting.** Sizes before and after, new names, the saving, the run order: all visible before Run, all in the units people think in.
4. **Colour has jobs.** One accent per channel, with a glyph so colour is never the only cue. Orange is output. Red is an alarm. Nothing else gets colour.
5. **Never draw over a photo.** Screen effects (grid, glass, bloom) sit under images, not on them.

## Materials
| Surface | Token | Notes |
|---|---|---|
| Case | `--surface-case` *tuned* `oklch(93.5% 0.0053 85)` | bead-blasted anodised aluminium, light from the top (`--light-amt` 0.54) |
| Grain | `--tex-fine` 304 px, sd 16.5 *tuned* | neutral-grey tile, blended soft-light so it never shifts colour |
| Scratches, dust | `--tex-scratch` 1920 px, `--tex-dust` 896 px *tuned* | CC0 ambientCG scans (Metal050C, SurfaceImperfections015), neutralised |
| Keys | `--surface-key`, wall `--key-depth` 2.75 px, soft `--key-soft` 8 px *tuned* | light keycaps, `.key--dark` rubber keys for utility actions |
| Screens | `--surface-screen` #050505 | OLED: pixel grid `--px-grid` 0.08 at `--px-pitch` 3 px, glass `--glass` 0.09, bloom `--bloom` 4.5 px *tuned* |
| Panels | `--r-panel` 13 px, engraved seam | panels are pressed into the case, not floating cards |

No screw heads, no faux bevels beyond the key walls, no glassmorphism.

## Type
| Role | Face | Token |
|---|---|---|
| Printed on the case: legends, labels, keys | Host Grotesk 500 to 800 | `--font-ui`, legends 10 px uppercase, tracking 0.09em *tuned* |
| Data on screens | Departure Mono | `--font-data`, `--fs-small` 11.5 px *tuned* |
| Big numerals on screens | Unbounded, weight 270 *tuned* | `--font-value`, 30 px |
| The meter | Doto, roundness 45 *tuned* | `--font-led`, 40 px, glow 14 px |

All four are SIL OFL, bundled in `design/system/fonts`.

## Colour
| Job | Token | Value |
|---|---|---|
| Size channel ■ | `--accent-size` / text on black `--accent-size-text` | #3d5cff / #7b8fff |
| Format channel ● | `--accent-format` | #18c06a |
| Quality channel ▲ | `--accent-quality` | #ffc61a |
| Name channel ◆ | `--accent-name` | #ff4f9a |
| Trim (inside Size) | `--accent-trim` | #16b5a2 |
| Canvas: pad, backdrop (inside Size) | `--accent-canvas` | #7c4dff |
| Utility steps | `--accent-tool` | graphite #4a4e57 |
| Output: Run, meter, saving | `--output` | #ff5a1f |
| Alarm: clash, over limit, failed | `--status-alarm` | #ff4438 |

Text pairs meet WCAG AA: ink on case 14.6:1, screen text from 17:1 down to the 4.8:1 floor (`--screen-3`). A lit key's legend uses `--on-<accent>`, checked for contrast per accent.

## Components
All in `design/system/components.css`, as Svelte components in `apps/desktop/src/lib/components`.

- **Key**: the one button. Latching (`aria-pressed`), radio (`role=radio`), or plain. Lit keys take the nearest `--accent`. `.key--insert` prints a + for keys that add rather than latch.
- **KeyGroup**: a row of radio keys. Arrows move and select.
- **ScreenChoice**: a choice printed on a screen (the size ruler, the format list, crop/pad/inside).
- **Slider**: recessed slot, cap with an accent stripe, printed stops that are presses too.
- **Field**: a text field set into the case like a small screen.
- **Screen**: the OLED container.
- **Ladder** and **LED readout**: the meter.
- **Tabs**: on screens, each with a count. Alarm tabs light their count red.
- **AnchorPad**: nine positions in a 3×3 pad.
- **Dialog**: native `<dialog>` in the case material.
- **Icons**: 16 px grid, 1.5 px stroke, round caps and joins (Lucide geometry), in `icons.ts`.

## Layout
The window is the unit: a 56 px header, the display and the Recipe panel (`clamp(312px, 25vw, 372px)`), and a 252 px strip of four channels beside the Output panel. Gutter 12 px *tuned*. Minimum window 1200 × 760.

## Motion
Keys press in 60 ms and latch in 120 ms. Screens change in 160 ms. All with `--ease-out`. Only transform and opacity animate. `prefers-reduced-motion` sets every duration to zero.

## Accessibility
Full keyboard: 1 to 4 focus the channels, arrows inside groups and the sheet, `⌘/Ctrl ↵` runs, `⌘/Ctrl K` opens commands, Space toggles the original. Every drag handle is a `role=slider` with arrow keys. Focus rings are 2 px, black on the case and white on screens. Colour is never the only cue: glyphs on channels, text on flags (`! CLASH`, `= KEPT`). `prefers-contrast: more` removes the pixel grid.

## Voice
Plain words, sentence case on screens, uppercase only for printed legends. Say what happens: "Saves 48 files into Wrangled/ beside each original", "Replaces 48 originals · undo for 30 days".
