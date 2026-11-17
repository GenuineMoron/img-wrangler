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

/** One image processed exactly as the run would, nothing written. */
export interface Preview {
  /** URLs the webview can show. */
  before: string;
  after: string;
  before_bytes: number;
  after_bytes: number;
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
}

export interface Integrations {
  targets: Integration[];
  pins: string[];
}
