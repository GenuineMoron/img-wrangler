# Research notes (November 2026)

Condensed from the research done before the rewrite. Figures were checked against GitHub and
crates.io where possible.

## Stack
- **UI:** Tauri 2 (2.12) with Svelte 5. Native Rust kits were ruled out for now: Iced and egui
  can't receive file drops on Wayland. GPUI's usable crates are unofficial weekly snapshots without
  shipped accessibility. Slint's external drop support is unverified. Risk: WebKitGTK performance
  on Linux, so the thumbnail grid stays light (virtualised, no filters or blend modes) and
  thumbnails are made in Rust. Tauri 3's CEF runtime is the escape hatch once stable.
- **Codecs:** image-rs for common decoding, MozJPEG, libwebp, ravif (pure Rust AVIF), oxipng,
  imagequant (GPL or commercial), avif-decode (rav1d) and jxl-oxide for reading AVIF and JPEG XL,
  resvg for SVG, fast_image_resize for SIMD resampling, moxcms for colour, ssimulacra2 for scoring.
- **HEIC:** HEVC is patent-encumbered. Use the OS decoder (sips, heif-dec) by default.

## Compression
- Looks target SSIMULACRA 2 scores: 90 visually lossless, 83 high, 75 balanced, 65 small.
- Search on detailed crops, then encode once. A size limit bisects quality, then optionally shrinks.
- Never write a same-format, same-size file that is bigger than the original.
- Upload targets (Shopify, Google Merchant, marketplaces) accept JPEG, PNG and WebP, not AVIF or JXL.
- JPEG XL is on by default in Chrome 155 and Firefox 158 (late 2026) but not yet a safe web default.

## What users ask for (competitors' issue trackers and forums)
Modern formats, folder recursion, safety for originals, skip already-optimised files, never
bigger, size targets, resize plus crop/pad, metadata control, rename patterns with previews, CLI
parity, before/after compare, native Linux builds, signed apps. The gap nobody fills: one local
pass doing trim → pad on a backdrop → resize → compress → rename, in place, with undo.

## E-commerce specs
- Amazon main images: pure white (RGB 255), product filling about 85%.
- Shopify: 2048 × 2048 square recommended. 20 MB, 25 MP limits.
- Google Merchant: at least 500 × 500 (from January 2027), 1500 px recommended, 16 MB.
- Bunnings MarketLink bans drop shadows. Shadows must stay optional.

## Local AI later (licences matter)
Background removal: BiRefNet or BEN2 (MIT weights. Check training-data terms). Avoid BRIA RMBG
(non-commercial), Ultralytics YOLO (AGPL), InsightFace models (non-commercial). Captions: Florence-2
(MIT). Inference via ONNX Runtime (`ort`).

## Licensing
AGPL-3.0-or-later plus a CLA keeps the project open while letting the author sell commercial and
app-store builds. A CLA, not a DCO, is what preserves relicensing. The trademark policy protects the
name. Closed builds need libimagequant's commercial licence or must drop palette PNG.

## Distribution
tauri-action for app bundles (DMG, AppImage/deb/rpm, NSIS/MSI), cargo-dist for the CLI, Homebrew
tap and AUR. macOS needs an Apple Developer account for notarisation. Azure Trusted Signing isn't
open to individuals outside the US/Canada, so Windows signing needs an OV certificate or SignPath.

## Design
Industrial hardware language after Teenage Engineering (OP-1 screens: one colour per channel,
thin wide numerals, hairline diagrams, black space) and audio plugins (RC-20 module colours).
Texture belongs to every surface (shared grain blended per surface), never to screens.
Screens: pixel grid, glow on bright marks, glass reflection. Photos never get effects over them.
