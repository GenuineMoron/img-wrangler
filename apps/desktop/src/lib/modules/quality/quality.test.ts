// SPDX-License-Identifier: AGPL-3.0-or-later
import { describe, expect, it } from 'vitest';
import { kbText, parseKb, roundKb } from './index';

describe('size limits', () => {
  it('read the way people type them', () => {
    expect(parseKb('400')).toBe(400);
    expect(parseKb('400 KB')).toBe(400);
    expect(parseKb('1.5 MB')).toBe(1500);
    expect(parseKb('2m')).toBe(2000);
    expect(parseKb('2,5mb')).toBe(2500);
    expect(parseKb('5')).toBeNull();
    expect(parseKb('big')).toBeNull();
  });
  it('say megabytes past a thousand kilobytes', () => {
    expect(kbText(400)).toBe('400 KB');
    expect(kbText(1500)).toBe('1.5 MB');
    expect(kbText(12000)).toBe('12 MB');
  });
  it('round to steps that grow with the size', () => {
    expect(roundKb(413)).toBe(410);
    expect(roundKb(1234)).toBe(1200);
    expect(roundKb(23456)).toBe(23000);
  });
});
