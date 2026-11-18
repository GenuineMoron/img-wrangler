# Third-party components

Wrangler builds on these projects. Licences are as published by each project. Check them before
redistributing a modified build.

| Component | Used for | Licence |
|---|---|---|
| image, png, gif, tiff, image-webp, zune-jpeg | Decoding common formats | MIT / Apache-2.0 / Zlib |
| MozJPEG (mozjpeg crate) | JPEG encoding | IJG, BSD-3-Clause, Zlib |
| libwebp (webp, libwebp-sys) | WebP encoding | BSD-3-Clause (libwebp), MIT/Apache-2.0 (bindings) |
| ravif, rav1e | AVIF encoding | BSD-2-Clause |
| avif-decode, rav1d | AVIF decoding | BSD-3-Clause / BSD-2-Clause |
| jxl-oxide | JPEG XL decoding | MIT / Apache-2.0 |
| resvg, usvg, tiny-skia | SVG rasterising | Apache-2.0 / MIT |
| oxipng | PNG optimisation | MIT |
| **imagequant (libimagequant)** | PNG palette reduction | **GPL-3.0-or-later**, or a commercial licence from its author |
| fast_image_resize | Resizing | MIT / Apache-2.0 |
| moxcms | Colour profile conversion | BSD-3-Clause / Apache-2.0 |
| ssimulacra2 | Perceptual quality scoring | BSD-2-Clause |
| kamadak-exif, img-parts | Metadata | BSD-2-Clause, MIT / Apache-2.0 |
| Host Grotesk, Doto, Unbounded | Interface type | SIL Open Font License 1.1 |
| Departure Mono | Screen type | SIL Open Font License 1.1 (per its site, MIT per some listings) |
| ambientCG Metal050C, SurfaceImperfections015 | Case texture source | CC0 |

**Note for commercial builds.** A closed-source or app-store build cannot include libimagequant
under the GPL. Either buy its commercial licence (pngquant.org) or build without palette PNG
reduction. HEIC decoding uses the operating system's own decoder by default to avoid bundling
HEVC.
