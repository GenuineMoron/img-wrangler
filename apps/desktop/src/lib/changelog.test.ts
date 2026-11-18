// SPDX-License-Identifier: AGPL-3.0-or-later
import { describe, expect, it } from 'vitest';
import { notes } from './changelog';

describe('changelog notes', () => {
  it('turns Markdown into paragraphs and list items', () => {
    const md = 'Faster everywhere.\n\n## Not shown\n- One **bold** thing.\n* A [link](https://example.com) and `code`.';
    expect(notes(md)).toEqual([
      { list: false, parts: [{ code: false, text: 'Faster everywhere.' }] },
      { list: true, parts: [{ code: false, text: 'One bold thing.' }] },
      { list: true, parts: [{ code: false, text: 'A link and ' }, { code: true, text: 'code' }, { code: false, text: '.' }] },
    ]);
  });
});
