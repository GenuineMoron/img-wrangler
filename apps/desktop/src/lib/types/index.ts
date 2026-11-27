// SPDX-License-Identifier: AGPL-3.0-or-later
// Types generated from the Rust engine (see .cargo/config.toml), plus the desktop host's own.

export type { Recipe } from './engine/Recipe';
export type { Size } from './engine/Size';
export type { Fit } from './engine/Fit';
export type { Anchor } from './engine/Anchor';
export type { Trim } from './engine/Trim';
export type { Pad } from './engine/Pad';
export type { Filter } from './engine/Filter';
export type { FormatChoice } from './engine/FormatChoice';
export type { Format } from './engine/Format';
export type { FormatInfo } from './engine/FormatInfo';
export type { Quality } from './engine/Quality';
export type { Rename } from './engine/Rename';
export type { RenameMode } from './engine/RenameMode';
export type { Case } from './engine/Case';
export type { Replacement } from './engine/Replacement';
export type { Metadata } from './engine/Metadata';
export type { KeepMeta } from './engine/KeepMeta';
export type { Destination } from './engine/Destination';
export type { DestinationMode } from './engine/DestinationMode';
export type { OnConflict } from './engine/OnConflict';
export type { AddOn } from './engine/AddOn';
export type { Rotate } from './engine/Rotate';
export type { Sharpen } from './engine/Sharpen';
export type { Watermark } from './engine/Watermark';
export type { Select } from './engine/Select';
export type { SourceFile } from './engine/SourceFile';
export type { Plan } from './engine/Plan';
export type { PlanItem } from './engine/PlanItem';
export type { Planned } from './engine/Planned';
export type { Probe } from './engine/Probe';
export type { Action } from './engine/Action';
export type { Estimate } from './engine/Estimate';
export type { Event as RunEvent } from './engine/Event';
export type { ItemReport } from './engine/ItemReport';
export type { Summary } from './engine/Summary';
export type { Status } from './engine/Status';
export type { RecipeEntry } from './engine/RecipeEntry';
export type { RunInfo } from './engine/RunInfo';
export type { UndoReport } from './engine/UndoReport';
export type { Dims } from './engine/Dims';

import type { Dims } from './engine/Dims';
import type { Format } from './engine/Format';

/** The selected image as the recipe makes it, nothing written: first a quick draft at screen
 *  size, then the exact result. */
export interface Preview {
  exact: boolean;
  /** URLs the webview can show: the original at screen size and at full size, and the result
   *  (screen size in a draft, the real output once exact). */
  before: string;
  before_full: string;
  after: string;
  before_bytes: number;
  /** Known once exact. */
  after_bytes: number | null;
  source_dims: Dims;
  dims: Dims;
  format: Format;
  quality: number | null;
  score: number | null;
  kept_original: boolean;
  over_limit: boolean;
  downscaled: boolean;
}

export interface QuickResult {
  recipe: import('./engine/Recipe').Recipe;
  summary: import('./engine/Summary').Summary;
  first_output: string | null;
}

export type LaunchPayload = { mode: 'open'; paths: string[] } | { mode: 'quick'; recipe: string; paths: string[] };

export interface Integration {
  target: string;
  label: string;
  installed: boolean;
  /** Where the menu shows, when that isn't simply the right-click menu. */
  place: string | null;
}

export interface Integrations {
  targets: Integration[];
}

/** What the self-test should do (see src/lib/selftest). */
export interface SelfTest {
  /** Files and folders to add, as a person would drop them. */
  images: string[];
  /** Time the everyday steps. */
  measure: boolean;
  /** Also run the batch, then undo it. */
  run: boolean;
  /** Also check the layout at several window sizes. */
  layout: boolean;
  /** What the app runs on, as name and value pairs. */
  host?: [string, string][];
  /** Effects to switch off, to measure what each costs (see src/lib/selftest/fx.css). */
  fx?: string[];
  /** Full or light effects for this test, instead of the automatic choice. */
  effects?: 'full' | 'light' | null;
  /** Look for a newer version, or also install it (the app restarts into it). */
  update?: 'check' | 'install' | null;
}

/** How the app itself behaves (settings.json), and what this system allows. */
export interface AppSettings {
  menu_bar: boolean;
  /** The system can show a menu bar or tray icon. */
  menu_bar_available: boolean;
  auto_update: boolean;
  /** Starts at login, in the menu bar. */
  login: boolean;
  /** The recipe images dropped on the menu bar icon run. Null opens them in the window. */
  drop_runs: string | null;
}

/** A newer version, with its changelog in Markdown. */
export interface UpdateInfo {
  version: string;
  current: string;
  date: string | null;
  notes: string;
  /** Set when this copy can't update itself: the command that does it. */
  manual: string | null;
}

/** One line of a right-click menu: a command, or a choice with a tick when `checked` is set.
 *  Null draws a separator. */
export type MenuLine = { text: string; action: () => void; enabled?: boolean; checked?: boolean } | null;
