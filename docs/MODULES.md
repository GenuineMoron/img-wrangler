# Adding a feature

Every feature is a module. A module has an engine half (Rust) and an app half (Svelte), and each half is registered in one list. Nothing else changes: the recipe panel, the processing-order display, the CLI and quick actions pick it up from the registries.

## Add-on module (the watermark is a worked example)

**Engine**
1. `crates/engine/src/modules/<name>.rs`: a settings struct (`serde`, `Default`, `ts-rs` export) and an `impl Stage` with `id`, `order` (pick a free slot from the table in ARCHITECTURE.md), `plan` (usually returns the dimensions unchanged) and `apply`. `modules/watermark.rs` is a complete, short example.
2. Put pixel work in `crates/engine/src/ops/` as pure functions if it is reusable.
3. Add the variant to `AddOn` in `crates/engine/src/modules/mod.rs`:
   ```rust
   #[serde(tag = "kind", rename_all = "snake_case")]
   pub enum AddOn { Rotate(Rotate), Sharpen(Sharpen), Watermark(Watermark) }
   ```
   and one arm in `AddOn::stage()`.
4. Tests in the same file: settings round-trip through TOML, `plan` dimensions, and `apply` on a small fixture.

**App**
1. Regenerate the types: `cargo test -p wrangler-engine --features ts export_bindings`.
2. `apps/desktop/src/lib/modules/watermark/index.ts`:
   ```ts
   const watermark: AddOnModule<'watermark'> = {
     kind: 'addon', addon: 'watermark', id: 'watermark', label: 'Watermark',
     blurb: 'Your logo in a corner of every image.', icon: 'image', accent: 'tool', order: 60,
     create: () => ({ kind: 'watermark', enabled: true, /* defaults */ }),
     active: r => r.addons.some(a => a.kind === 'watermark' && a.enabled),
     summary: r => '…',
     Details,
   };
   export default watermark;
   ```
3. `WatermarkDetails.svelte` takes `{ value }` (the add-on's settings, edited in place) and is built from the shared components (`Row`, `Key`, `KeyGroup`, `Slider`, `Field`, `ColourKey`, `AnchorPad`).
4. Add it to `MODULES` in `apps/desktop/src/lib/modules/registry.ts`. It now appears under Add step, in the Recipe panel in run order, and in the command palette.

**CLI**: add-ons are read from recipe files automatically. Add a flag in `crates/cli` only if it is common enough to deserve one.

## Output format (example: JPEG XL encode)
1. `crates/engine/src/codec/encode/jxl.rs` implementing `Encoder`.
2. Flip `encode: true` on its `FormatInfo` in `crates/engine/src/codec/mod.rs` and route it in `encoder_for`.
3. Round-trip test. The app shows it under Format › More with no UI change.

## Core modules
Core modules are always in a recipe: Files, Size, Format, Quality, Metadata, Name and Save. A core module may have a channel (`channel: { slot, glyph, Screen, Keys }`) for the everyday controls and `Details` for the rest. Size, Format, Quality and Name have channels, in slots 1 to 4 (also their number keys). A channel screen is the control itself: draw it with `ScreenChoice`, drag handles and typed values rather than a picture beside separate inputs.

The four channels have fixed places in the strip. Change their settings structs and components freely, but keep the recipe backward compatible: new fields need defaults, and removing a field means bumping `RECIPE_VERSION` with a migration in `recipe.rs`.

## Rules every module follows
- The UI reads and writes the recipe directly (`recipe.value.<section>`). No copies, no events.
- Settings have defaults. A module with default settings changes nothing.
- `plan` never decodes pixels.
- `apply` never does IO.
- One accent colour per module, from the tokens, and a glyph so colour is never the only cue.
- Every control is keyboard reachable with a visible label or `aria-label`.
