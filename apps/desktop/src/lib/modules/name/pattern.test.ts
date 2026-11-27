// SPDX-License-Identifier: AGPL-3.0-or-later
import { describe, expect, it } from 'vitest';
import { label, move, parse, print, text, token } from './pattern';

describe('name patterns as pieces', () => {
  it('reads tokens and the text between them, and writes the same pattern back', () => {
    for (const p of ['spring-24-{n:000}', '{name}', '{name|slug}_{w}x{h}', '{date}-{n}', '', 'plain', '{folder}/{taken}-{n:00000}']) {
      expect(print(parse(p))).toBe(p);
    }
    expect(parse('spring-{n:000}')).toEqual([text('spring-'), { kind: 'token', key: 'n', pad: 3, filter: null }]);
    expect(parse('{name|slug}')).toEqual([{ kind: 'token', key: 'name', pad: 0, filter: 'slug' }]);
  });

  it('keeps a brace that never closes as text', () => {
    expect(parse('a{b')).toEqual([text('a{b')]);
    expect(print(parse('a{b'))).toBe('a{b');
  });

  it('labels a number by its digits', () => {
    expect(label(token('n'))).toBe('001');
    expect(label(token('n', 1))).toBe('1');
    expect(label(token('n', 5))).toBe('00001');
    expect(label(token('name'))).toBe('name');
    expect(label(token('name', 0, 'slug'))).toBe('name a-b');
  });

  it('moves a piece to before another', () => {
    const p = parse('{name}-{n:000}');
    expect(print(move(p, 2, 0))).toBe('{n:000}{name}-');
    expect(print(move(p, 0, 3))).toBe('-{n:000}{name}');
    expect(print(move(p, 1, 1))).toBe('{name}-{n:000}');
  });
});
