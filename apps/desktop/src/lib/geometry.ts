// SPDX-License-Identifier: AGPL-3.0-or-later
// A mirror of the engine's frame layout (crates/engine/src/ops/frame.rs) so the Size diagram
// can answer while you drag. The plan from the engine stays the authority for real numbers;
// geometry.test.ts holds this to the engine's own test cases.

import type { Anchor, Dims, Fit, Size } from '$lib/types';

export type Shape = { kind: 'original' } | { kind: 'ratio'; a: number; b: number } | { kind: 'exact'; w: number; h: number };

export interface Rect { x: number; y: number; w: number; h: number }

export interface Layout {
  canvas: Dims;
  /** Part of the content kept when cropping. */
  crop: Rect | null;
  scaled: Dims;
  offset: [number, number];
}

export function parseShape(text: string): Shape | null {
  const t = text.trim().toLowerCase();
  if (t === '' || t === 'original' || t === 'orig') return { kind: 'original' };
  const num = (s: string) => (/^\d+$/.test(s.trim()) && Number(s) > 0 ? Number(s) : null);
  const ratio = t.split(':');
  if (ratio.length === 2) {
    const [a, b] = ratio.map(num);
    return a && b ? { kind: 'ratio', a, b } : null;
  }
  const exact = t.split(/[x×]/);
  if (exact.length === 2) {
    const [w, h] = exact.map(num);
    return w && h ? { kind: 'exact', w, h } : null;
  }
  return null;
}

export function shapeText(s: Shape): string {
  return s.kind === 'original' ? 'original' : s.kind === 'ratio' ? `${s.a}:${s.b}` : `${s.w}x${s.h}`;
}

export const ANCHORS: Record<Anchor, [number, number]> = {
  center: [0.5, 0.5], top: [0.5, 0], bottom: [0.5, 1], left: [0, 0.5], right: [1, 0.5],
  top_left: [0, 0], top_right: [1, 0], bottom_left: [0, 1], bottom_right: [1, 1],
};

const round = (v: number) => Math.max(1, Math.round(v));
const clamp = (v: number, lo: number, hi: number) => Math.min(hi, Math.max(lo, v));

function ratioFrame(a: number, b: number, long: number): Dims {
  return a >= b ? { w: long, h: round((long * b) / a) } : { w: round((long * a) / b), h: long };
}

export interface FrameSpec {
  long_side: number | null;
  shape: Shape;
  fit: Fit;
  anchor: [number, number];
  /** 0.05-1 */
  fill: number;
  upscale: boolean;
}

export function specOf(size: Size): FrameSpec {
  return {
    long_side: size.long_side,
    shape: parseShape(size.shape) ?? { kind: 'original' },
    fit: size.fit,
    anchor: ANCHORS[size.anchor],
    fill: size.pad.fill / 100,
    upscale: size.upscale,
  };
}

export function layout(content: Dims, spec: FrameSpec): Layout {
  const cw = content.w, ch = content.h;
  const fill = clamp(spec.fill, 0.05, 1);
  const [ax, ay] = [clamp(spec.anchor[0], 0, 1), clamp(spec.anchor[1], 0, 1)];
  const s0 = spec.shape;

  let frame: Dims;
  if (s0.kind === 'original') {
    let s = spec.long_side == null ? 1 : spec.long_side / Math.max(cw, ch);
    if (!spec.upscale) s = Math.min(s, 1);
    const d = { w: round(cw * s), h: round(ch * s) };
    return { canvas: d, crop: null, scaled: d, offset: [0, 0] };
  } else if (s0.kind === 'exact') {
    frame = { w: Math.max(1, s0.w), h: Math.max(1, s0.h) };
  } else if (spec.long_side != null) {
    frame = ratioFrame(s0.a, s0.b, spec.long_side);
  } else {
    const r = s0.a / s0.b;
    if (spec.fit === 'crop') frame = cw / ch > r ? { w: round(ch * r), h: ch } : { w: cw, h: round(cw / r) };
    else {
      const bw = cw / fill, bh = ch / fill;
      frame = bw / bh > r ? { w: round(bw), h: round(bw / r) } : { w: round(bh * r), h: round(bh) };
    }
  }
  const fw = frame.w, fh = frame.h;

  if (spec.fit === 'crop') {
    let s = Math.max(fw / cw, fh / ch);
    let canvas = frame;
    if (!spec.upscale && s > 1) {
      canvas = { w: round(fw / s), h: round(fh / s) };
      s = 1;
    }
    const kw = canvas.w / s, kh = canvas.h / s;
    const crop = { x: Math.max(0, Math.round((cw - kw) * ax)), y: Math.max(0, Math.round((ch - kh) * ay)), w: Math.min(round(kw), cw), h: Math.min(round(kh), ch) };
    return { canvas, crop, scaled: canvas, offset: [0, 0] };
  }
  if (spec.fit === 'pad') {
    let s = Math.min((fw * fill) / cw, (fh * fill) / ch);
    if (!spec.upscale) s = Math.min(s, 1);
    const scaled = { w: Math.min(round(cw * s), frame.w), h: Math.min(round(ch * s), frame.h) };
    return { canvas: frame, crop: null, scaled, offset: [Math.round((fw - scaled.w) * ax), Math.round((fh - scaled.h) * ay)] };
  }
  let s = Math.min(fw / cw, fh / ch);
  if (!spec.upscale) s = Math.min(s, 1);
  const d = { w: round(cw * s), h: round(ch * s) };
  return { canvas: d, crop: null, scaled: d, offset: [0, 0] };
}

/** The output size of an image under a recipe's Size settings (before trim, which needs pixels). */
export function outputDims(content: Dims, size: Size): Dims {
  return layout(content, specOf(size)).canvas;
}
