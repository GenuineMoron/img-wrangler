// SPDX-License-Identifier: AGPL-3.0-or-later
// A name pattern as pieces. The engine reads a pattern as text with tokens in braces
// (`shoot-{n:000}`, see naming.rs). The Name screen shows it as blocks: one for each token, and
// one for each run of text typed between them.

export type Piece =
  | { kind: 'text'; text: string }
  /** `pad` is how many digits a number has. `filter` is a change to the value, like `slug`. */
  | { kind: 'token'; key: string; pad: number; filter: string | null };

/** The tokens the engine knows, in the order they're offered. */
export const TOKENS = [
  { key: 'name', add: 'name', means: 'The original name' },
  { key: 'n', add: 'number', means: 'A number that counts up' },
  { key: 'date', add: 'date', means: 'The file’s date' },
  { key: 'taken', add: 'taken', means: 'The date the photo was taken' },
  { key: 'w', add: 'width', means: 'The new width in pixels' },
  { key: 'h', add: 'height', means: 'The new height in pixels' },
  { key: 'folder', add: 'folder', means: 'The name of the folder the image is in' },
] as const;

export const token = (key: string, pad = 0, filter: string | null = null): Piece => ({ kind: 'token', key, pad: key === 'n' && !pad ? 3 : pad, filter });
export const text = (t: string): Piece => ({ kind: 'text', text: t });

export function parse(pattern: string): Piece[] {
  const out: Piece[] = [];
  const say = (t: string) => {
    const last = out[out.length - 1];
    if (!t) return;
    if (last?.kind === 'text') last.text += t;
    else out.push(text(t));
  };
  let rest = pattern;
  for (;;) {
    const open = rest.indexOf('{');
    const close = open < 0 ? -1 : rest.indexOf('}', open);
    if (close < 0) { say(rest); return out; }
    say(rest.slice(0, open));
    const [head, filter = null] = rest.slice(open + 1, close).split('|').map(s => s.trim());
    const [key, pad = ''] = head.split(':').map(s => s.trim());
    out.push({ kind: 'token', key, pad: pad.length, filter });
    rest = rest.slice(close + 1);
  }
}

export function print(pieces: Piece[]): string {
  return pieces.map(p => (p.kind === 'text' ? p.text : `{${p.key}${p.pad ? `:${'0'.repeat(p.pad)}` : ''}${p.filter ? `|${p.filter}` : ''}}`)).join('');
}

/** What a block says on the screen. */
export function label(p: Piece): string {
  if (p.kind === 'text') return p.text;
  if (p.key === 'n') return p.pad > 1 ? `${'0'.repeat(p.pad - 1)}1` : '1';
  const known = TOKENS.find(t => t.key === p.key)?.add ?? p.key;
  return p.filter ? `${known} ${p.filter === 'slug' ? 'a-b' : p.filter === 'lower' ? 'abc' : p.filter === 'upper' ? 'ABC' : p.filter}` : known;
}

/** A block in words, for screen readers and tooltips. */
export function describe(p: Piece): string {
  if (p.kind === 'text') return `Text “${p.text}”`;
  if (p.key === 'n') return `A number that counts up, ${p.pad > 1 ? `${p.pad} digits` : 'no leading zeros'}`;
  return TOKENS.find(t => t.key === p.key)?.means ?? `{${p.key}}`;
}

/** Move the piece at `from` so it ends up before what is now at `to` (`to` may be the length). */
export function move(pieces: Piece[], from: number, to: number): Piece[] {
  const out = [...pieces];
  const [p] = out.splice(from, 1);
  out.splice(to > from ? to - 1 : to, 0, p);
  return out;
}
