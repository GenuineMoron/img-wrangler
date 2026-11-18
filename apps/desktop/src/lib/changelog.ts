// SPDX-License-Identifier: AGPL-3.0-or-later
// The update dialog shows a version's changelog. It's Markdown from CHANGELOG.md, shown as plain
// paragraphs and list items, with code kept apart for its own type.

export type Line = { list: boolean; parts: { code: boolean; text: string }[] };

/** A changelog in Markdown as paragraphs and list items of plain text and code. */
export function notes(md: string): Line[] {
  return md
    .split('\n')
    .map(l => l.trim())
    .filter(l => l && !l.startsWith('#'))
    .map(l => {
      const list = /^[-*] /.test(l);
      const text = (list ? l.slice(2) : l).replace(/\[([^\]]+)\]\([^)]+\)/g, '$1').replace(/\*\*([^*]+)\*\*/g, '$1');
      const parts = text.split(/(`[^`]+`)/).filter(Boolean).map(p => (p.startsWith('`') ? { code: true, text: p.slice(1, -1) } : { code: false, text: p }));
      return { list, parts };
    });
}
