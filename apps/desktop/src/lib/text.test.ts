// SPDX-License-Identifier: AGPL-3.0-or-later
import { describe, expect, it } from 'vitest';
import { bytes, change, commonFolder, kb } from './text';

describe('text', () => {
  it('writes sizes the way screens show them', () => {
    expect(bytes(1.82e9)).toBe('1.82 GB');
    expect(bytes(41.2e6)).toBe('41 MB');
    expect(bytes(5.8e6)).toBe('5.8 MB');
    expect(kb(4.3e6)).toBe('4.3M');
    expect(kb(412_000)).toBe('412K');
  });
  it('never rounds a real saving up to 100%', () => {
    expect(change(39e6, 175e3)).toBe('−99%');
    expect(change(100, 112)).toBe('+12%');
  });
  it('finds the shared folder', () => {
    expect(commonFolder(['/Users/sean/Shoot/a/1.jpg', '/Users/sean/Shoot/b/2.jpg'])).toBe('~/Shoot');
  });
});
