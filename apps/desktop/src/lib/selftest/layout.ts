// SPDX-License-Identifier: AGPL-3.0-or-later
// Finds layout faults in whatever is on screen: text cut off without an ellipsis, labels
// shortened by one, controls spilling out of their panel, controls overlapping each other, and
// controls pushed off the window. Pure DOM measurement, so it works in every webview.

export type IssueKind = 'clipped' | 'truncated' | 'spills' | 'overlap' | 'offscreen';

export interface Issue {
  kind: IssueKind;
  /** A short selector path to the element. */
  where: string;
  /** Its text, shortened. */
  text: string;
  /** How far over, in CSS pixels. */
  by: number;
}

const INTERACTIVE = 'button, input, textarea, select, a[href], [role="slider"], [role="tab"], [role="option"], [role="radio"], [tabindex="0"]';

function visible(el: Element): boolean {
  const r = el.getBoundingClientRect();
  if (r.width < 2 || r.height < 2) return false;
  for (let e: Element | null = el; e; e = e.parentElement) {
    const s = getComputedStyle(e);
    if (s.display === 'none' || s.visibility === 'hidden' || Number(s.opacity) === 0) return false;
    if (e.tagName === 'DIALOG' && !(e as HTMLDialogElement).open) return false;
  }
  return true;
}

function describe(el: Element): string {
  const parts: string[] = [];
  for (let e: Element | null = el, i = 0; e && i < 4 && e !== document.body; e = e.parentElement, i++) {
    const cls = [...e.classList].filter(c => !c.startsWith('svelte-')).slice(0, 2).join('.');
    parts.unshift(`${e.tagName.toLowerCase()}${e.id ? `#${e.id}` : ''}${cls ? `.${cls}` : ''}`);
  }
  return parts.join(' > ');
}

const short = (s: string | null | undefined) => (s ?? '').replace(/\s+/g, ' ').trim().slice(0, 48);

function ownText(el: Element): string {
  let t = '';
  for (const n of el.childNodes) if (n.nodeType === Node.TEXT_NODE) t += n.textContent;
  return t.trim();
}

const clips = (s: CSSStyleDeclaration) => /hidden|clip|auto|scroll/.test(s.overflowX + s.overflowY);
const scrolls = (s: CSSStyleDeclaration) => /auto|scroll/.test(s.overflowX + s.overflowY);

/** The part of an element you can actually see: its box cut by every scrolling list it sits in.
 *  Null when it's scrolled right out of view. */
function visiblePart(el: Element): DOMRect | null {
  let r = el.getBoundingClientRect();
  for (let p = el.parentElement; p && p !== document.body; p = p.parentElement) {
    if (!scrolls(getComputedStyle(p))) continue;
    const q = p.getBoundingClientRect();
    const left = Math.max(r.left, q.left), top = Math.max(r.top, q.top), right = Math.min(r.right, q.right), bottom = Math.min(r.bottom, q.bottom);
    if (right - left < 2 || bottom - top < 2) return null;
    r = new DOMRect(left, top, right - left, bottom - top);
  }
  return r;
}

/** Screen-reader-only text is clipped on purpose. */
const srOnly = (el: Element) => !!el.closest('.sr-only');

export function audit(): Issue[] {
  const issues: Issue[] = [];
  // With a modal dialog open, the page behind it is inert: check only the dialog.
  const root = document.querySelector('dialog[open]') ?? document.body;
  const all = [...root.querySelectorAll<HTMLElement>('*')].filter(el => !srOnly(el) && !(el instanceof SVGElement));
  const shown = all.filter(visible);

  // Text that doesn't fit its box.
  for (const el of shown) {
    const text = ownText(el);
    if (!text || el instanceof HTMLInputElement || el instanceof HTMLTextAreaElement) continue;
    const s = getComputedStyle(el);
    const overX = el.scrollWidth - el.clientWidth, overY = el.scrollHeight - el.clientHeight;
    if (!clips(s) || scrolls(s)) continue;
    if (overX > 1 || overY > 2) {
      const ellipsis = s.textOverflow === 'ellipsis' && overX > 1;
      // File names, folders and recipe names are the person's own and can be any length, so
      // they're allowed to end in an ellipsis (marked data-long). Labels aren't.
      if (ellipsis && el.closest('[data-long]')) continue;
      issues.push({ kind: ellipsis ? 'truncated' : 'clipped', where: describe(el), text: short(text), by: Math.max(overX, overY) });
    }
  }

  // Controls and labels that spill out of a panel that clips them.
  for (const el of shown) {
    if (!el.matches(INTERACTIVE) && !ownText(el)) continue;
    const r = el.getBoundingClientRect();
    for (let p = el.parentElement; p && p !== document.body; p = p.parentElement) {
      const s = getComputedStyle(p);
      if (!clips(s)) continue;
      if (scrolls(s)) break;
      const q = p.getBoundingClientRect();
      const by = Math.max(q.left - r.left, r.right - q.right, q.top - r.top, r.bottom - q.bottom);
      if (by > 1.5) issues.push({ kind: 'spills', where: describe(el), text: short(el.textContent || el.getAttribute('aria-label')), by: Math.round(by) });
      break;
    }
  }

  // Controls overlapping each other (not one inside the other), as far as they can be seen.
  const visibleControls = shown.filter(el => el.matches(INTERACTIVE) && !el.closest('.sheet')).map(el => [el, visiblePart(el)] as const).filter(([, r]) => r);
  const controls = visibleControls.map(([el]) => el);
  const rects = visibleControls.map(([, r]) => r!);
  for (let i = 0; i < controls.length; i++) {
    for (let j = i + 1; j < controls.length; j++) {
      if (controls[i].contains(controls[j]) || controls[j].contains(controls[i])) continue;
      const a = rects[i], b = rects[j];
      const w = Math.min(a.right, b.right) - Math.max(a.left, b.left), h = Math.min(a.bottom, b.bottom) - Math.max(a.top, b.top);
      if (w > 2 && h > 2) issues.push({ kind: 'overlap', where: `${describe(controls[i])}  ×  ${describe(controls[j])}`, text: short(controls[i].textContent || controls[i].getAttribute('aria-label')), by: Math.round(Math.min(w, h)) });
    }
  }

  // Controls outside the window.
  for (let i = 0; i < controls.length; i++) {
    const r = rects[i];
    const by = Math.max(-r.left, r.right - innerWidth, -r.top, r.bottom - innerHeight);
    if (by > 1) issues.push({ kind: 'offscreen', where: describe(controls[i]), text: short(controls[i].textContent || controls[i].getAttribute('aria-label')), by: Math.round(by) });
  }

  // One entry per element and kind.
  const seen = new Set<string>();
  return issues.filter(i => (seen.has(i.kind + i.where) ? false : (seen.add(i.kind + i.where), true)));
}
