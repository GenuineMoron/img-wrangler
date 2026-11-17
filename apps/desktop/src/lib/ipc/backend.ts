// SPDX-License-Identifier: AGPL-3.0-or-later
// Everything the UI asks of the outside world. Two implementations: the Tauri host (the real
// engine) and a mock with sample photos that runs in a plain browser for design work.

import type {
  QuickResult, Estimate, FormatInfo, Integrations, LaunchPayload, Plan, Preview, Recipe, RecipeEntry, RunEvent, RunInfo, Select, SourceFile, Summary, UndoReport,
} from '$lib/types';

export interface Backend {
  readonly kind: 'tauri' | 'mock';
  /** "mac", "windows" or "linux": decides shortcut glyphs and window chrome. */
  readonly platform: 'mac' | 'windows' | 'linux';

  formats(): Promise<FormatInfo[]>;
  recipes(): Promise<RecipeEntry[]>;
  /** A saved or built-in recipe; "new" gives the defaults. */
  loadRecipe(id: string): Promise<Recipe>;
  saveRecipe(recipe: Recipe): Promise<string>;
  deleteRecipe(id: string): Promise<void>;

  /** Add files and folders to the batch (or replace it); returns the whole batch. */
  scan(paths: string[], select: Select, append: boolean): Promise<SourceFile[]>;
  /** Remove files from the batch; an empty list clears it. */
  forget(paths: string[]): Promise<SourceFile[]>;
  plan(recipe: Recipe): Promise<Plan>;
  /** Sampled output size; rejects with "superseded" when a newer estimate starts. */
  estimate(recipe: Recipe, samples?: number): Promise<Estimate>;
  preview(path: string, recipe: Recipe): Promise<Preview>;

  run(recipe: Recipe, onEvent: (e: RunEvent) => void): Promise<Summary>;
  /** Run a saved recipe on paths from the file manager, apart from the main batch. */
  quickRun(recipeId: string, paths: string[], onEvent: (e: RunEvent) => void): Promise<QuickResult>;
  /** Load paths into the main window and show it. */
  openMain(paths: string[]): Promise<void>;
  cancel(): Promise<void>;
  undo(runId: string): Promise<UndoReport>;
  history(): Promise<RunInfo[]>;

  /** Paths the app was opened with from a file manager, once; each window takes its own kind. */
  takeLaunch(kind: LaunchPayload['mode']): Promise<LaunchPayload | null>;
  onLaunch(cb: () => void): Promise<() => void>;
  /** Files dropped on the window, and whether something is hovering. */
  onDrop(drop: (paths: string[]) => void, hover: (over: boolean) => void): Promise<() => void>;
  pick(kind: 'files' | 'folder'): Promise<string[]>;
  pickFolder(): Promise<string | null>;
  reveal(path: string): Promise<void>;
  thumbUrl(path: string): string;

  integrations(): Promise<Integrations>;
  setIntegrations(install: boolean, pins: string[]): Promise<Integrations>;

  /** Show the window once the first frame is drawn (avoids a white flash). */
  ready(): Promise<void>;
  close(): Promise<void>;
}

export const IMAGE_EXTENSIONS = ['jpg', 'jpeg', 'png', 'webp', 'avif', 'heic', 'heif', 'tif', 'tiff', 'gif', 'bmp', 'svg', 'jxl', 'qoi', 'ico', 'tga', 'pnm', 'ppm', 'pgm'];
