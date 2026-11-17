// SPDX-License-Identifier: AGPL-3.0-or-later
// The engine's frame tests, run against the TypeScript mirror.

import { describe, expect, it } from 'vitest';
import { layout, parseShape, shapeText, type FrameSpec, type Shape } from './geometry';

const PHOTO = { w: 6000, h: 4000 };
const spec = (long: number | null, shape: Shape, fit: FrameSpec['fit'], extra: Partial<FrameSpec> = {}): FrameSpec =>
  ({ long_side: long, shape, fit, anchor: [0.5, 0.5], fill: 1, upscale: false, ...extra });
const sq: Shape = { kind: 'ratio', a: 1, b: 1 };

describe('frame layout matches the engine', () => {
  it('resizes the long side', () => {
    const l = layout(PHOTO, spec(2048, { kind: 'original' }, 'pad'));
    expect(l.canvas).toEqual({ w: 2048, h: 1365 });
  });
  it('never upscales unless asked', () => {
    expect(layout({ w: 800, h: 600 }, spec(2048, { kind: 'original' }, 'pad')).canvas).toEqual({ w: 800, h: 600 });
    expect(layout({ w: 800, h: 600 }, spec(1600, { kind: 'original' }, 'pad', { upscale: true })).canvas).toEqual({ w: 1600, h: 1200 });
  });
  it('pads a square at the fill', () => {
    const l = layout(PHOTO, spec(2048, sq, 'pad', { fill: 0.85 }));
    expect(l.canvas).toEqual({ w: 2048, h: 2048 });
    expect(l.scaled.w).toBe(1741);
    expect(l.offset).toEqual([154, 444]);
  });
  it('crops a square from the centre', () => {
    const l = layout(PHOTO, spec(1000, sq, 'crop'));
    expect(l.canvas).toEqual({ w: 1000, h: 1000 });
    expect(l.crop).toEqual({ x: 1000, y: 0, w: 4000, h: 4000 });
  });
  it('crops from the top when anchored there', () => {
    expect(layout({ w: 1000, h: 3000 }, spec(1000, sq, 'crop', { anchor: [0.5, 0] })).crop).toEqual({ x: 0, y: 0, w: 1000, h: 1000 });
  });
  it('shrinks the frame instead of upscaling a crop', () => {
    expect(layout({ w: 600, h: 400 }, spec(2048, sq, 'crop')).canvas).toEqual({ w: 400, h: 400 });
  });
  it('makes exact canvases', () => {
    expect(layout(PHOTO, spec(null, { kind: 'exact', w: 1200, h: 628 }, 'crop')).canvas).toEqual({ w: 1200, h: 628 });
  });
  it('keeps its own shape inside a box', () => {
    expect(layout(PHOTO, spec(null, { kind: 'exact', w: 1000, h: 1000 }, 'inside')).canvas).toEqual({ w: 1000, h: 667 });
  });
});

describe('shapes', () => {
  it('parse and print', () => {
    for (const t of ['original', '1:1', '4:5', '1200x628']) expect(shapeText(parseShape(t)!)).toBe(t);
    expect(parseShape('0:1')).toBeNull();
    expect(parseShape('square')).toBeNull();
  });
});
