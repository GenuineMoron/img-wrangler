// SPDX-License-Identifier: AGPL-3.0-or-later
// How numbers, sizes and names are written on screens.

/** 1.82 GB, 411 MB, 412 KB. */
export function bytes(n: number): string {
  if (n >= 1e9) return `${(n / 1e9).toFixed(2)} GB`;
  if (n >= 1e7) return `${Math.round(n / 1e6)} MB`;
  if (n >= 1e6) return `${(n / 1e6).toFixed(1)} MB`;
  if (n >= 1e3) return `${Math.round(n / 1e3)} KB`;
  return `${n} B`;
}

/** Compact: 4.3M, 412K. */
export function kb(n: number): string {
  if (n >= 1e7) return `${Math.round(n / 1e6)}M`;
  if (n >= 1e6) return `${(n / 1e6).toFixed(1)}M`;
  return `${Math.max(1, Math.round(n / 1e3))}K`;
}

export const count = (n: number) => n.toLocaleString('en');

/** −86% or +12%, with a true minus sign. */
export function change(from: number, to: number): string {
  if (!from) return '';
  let pct = Math.round((1 - to / from) * 100);
  if (pct === 100 && to > 0) pct = 99;
  return pct >= 0 ? `−${pct}%` : `+${-pct}%`;
}

export const fileName = (path: string) => path.split(/[\\/]/).pop() ?? path;
export const folderOf = (path: string) => path.slice(0, Math.max(path.lastIndexOf('/'), path.lastIndexOf('\\')));

/** The folder shared by every path, with the home folder shortened to ~. */
export function commonFolder(paths: string[]): string {
  if (!paths.length) return '';
  const split = (p: string) => folderOf(p).split(/[\\/]/);
  let common = split(paths[0]);
  for (const p of paths.slice(1, 400)) {
    const s = split(p);
    let i = 0;
    while (i < common.length && common[i] === s[i]) i++;
    common = common.slice(0, i);
  }
  return common.join('/').replace(/^\/(Users|home)\/[^/]+/, '~') || '/';
}

export const plural = (n: number, one: string, many = `${one}s`) => `${count(n)} ${n === 1 ? one : many}`;
