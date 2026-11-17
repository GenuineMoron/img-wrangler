# Adding a feature

Every feature is a module. A module has an engine half (Rust) and an app half (Svelte), and each half is registered in one list. Nothing else changes: the recipe panel, the processing-order display, the CLI and quick actions pick it up from the registries.

## Add-on module (example: watermark)

**Engine**
1. `crates/engine/src/modules/watermark.rs`: a settings struct (`serde`, `Default`, `ts-rs` export) and an `impl Stage` with `id`, `order` (pick a free slot from the table in ARCHITECTURE.md), `plan` (usually returns the dimensions unchanged) and `apply`.
2. Put pixel work in `crates/engine/src/ops/` as pure functions if it is reusable.
3. Add the variant to `AddOn` in `crates/engine/src/modules/mod.rs`:
   ```rust
   #[serde(tag = "kind", rename_all = "snake_case")]
   pub enum AddOn { Rotate(Rotate), Sharpen(Sharpen), Watermark(Watermark) }
   ```
   and one arm in `AddOn::stage()`.
4. Tests in the same file: settings round-trip through TOML, `plan` dimensions, and `apply` on a small fixture.

**App**
1. `apps/desktop/src/lib/modules/watermark/`: `index.ts` exporting the `UiModule` (id, label, accent, glyph, order, defaults, summary), and `Details.svelte` built from the shared components.
2. Add it to `apps/desktop/src/lib/modules/registry.ts`.

**CLI**: add-ons are read from recipe files automatically. Add a flag in `crates/cli` only if it is common enough to deserve one.

## Output format (example: JPEG XL encode)
1. `crates/engine/src/codec/encode/jxl.rs` implementing `Encoder`.
2. Flip `encode: true` on its `FormatInfo` in `crates/engine/src/codec/mod.rs` and route it in `encoder_for`.
3. Round-trip test. The app shows it under Format › More with no UI change.

## Core module changes
The four core modules (size, format, quality, name) have fixed places in the channel strip. Change their settings structs and components freely, but keep the recipe backward compatible: new fields need defaults, and removing a field means bumping `RECIPE_VERSION` with a migration in `recipe.rs`.

## Rules every module follows
- Settings have defaults; a module with default settings changes nothing.
- `plan` never decodes pixels.
- `apply` never does IO.
- One accent colour per module, from the tokens, and a glyph so colour is never the only cue.
- Every control is keyboard reachable with a visible label or `aria-label`.
