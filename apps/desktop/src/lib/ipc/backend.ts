// SPDX-License-Identifier: AGPL-3.0-or-later
// Everything the UI asks of the outside world. Two implementations: the Tauri host (the real
// engine) and a mock with sample photos that runs in a plain browser for design work.

import type {
  AppSettings, QuickResult, Estimate, FormatInfo, Integrations, LaunchPayload, MenuLine, Plan, Preview, Recipe, RecipeEntry, RunEvent, RunInfo, Select, SelfTest, SourceFile, Summary, UndoReport, UpdateInfo,
} from '$lib/types';

export interface Backend {
  readonly kind: 'tauri' | 'mock';
  /** "mac", "windows" or "linux": decides shortcut glyphs and window chrome. */
  readonly platform: 'mac' | 'windows' | 'linux';

  formats(): Promise<FormatInfo[]>;
  recipes(): Promise<RecipeEntry[]>;
  /** A saved or built-in recipe; "new" gives the defaults. */
  loadRecipe(id: string): Promise<Recipe>;
  /** Save under the recipe's name. Gives its id. */
  saveRecipe(recipe: Recipe): Promise<string>;
  /** Save under a new name, leaving the recipe it came from (`from`) alone. Gives the new id. */
  saveRecipeAs(recipe: Recipe, from: string): Promise<string>;
  deleteRecipe(id: string): Promise<void>;
  /** Gives the new id. */
  renameRecipe(id: string, name: string): Promise<string>;
  /** A copy, placed after the original. Gives the copy's id. */
  duplicateRecipe(id: string): Promise<string>;
  /** Put a changed starter recipe back as it came. */
  resetRecipe(id: string): Promise<void>;
  /** Bring back the starter recipes that were deleted. */
  restoreRecipes(): Promise<void>;
  /** A recipe that's on is offered in the window, the right-click menus and the menu bar. */
  setRecipeOn(id: string, on: boolean): Promise<void>;
  setRecipeOrder(ids: string[]): Promise<void>;

  /** Add files and folders to the batch (or replace it); returns the whole batch. */
  scan(paths: string[], select: Select, append: boolean): Promise<SourceFile[]>;
  /** Remove files from the batch; an empty list clears it. */
  forget(paths: string[]): Promise<SourceFile[]>;
  plan(recipe: Recipe): Promise<Plan>;
  /** Sampled output size; rejects with "superseded" when a newer estimate starts. */
  estimate(recipe: Recipe, samples?: number): Promise<Estimate>;
  /** The quick draft (or the exact result when it's ready already), or the exact result. */
  preview(path: string, recipe: Recipe, exact: boolean): Promise<Preview>;
  /** Stop working on previews of anything but these files under this recipe. */
  previewFocus(paths: string[], recipe: Recipe): Promise<void>;

  /** Run the batch, or only these files of it. */
  run(recipe: Recipe, onEvent: (e: RunEvent) => void, only?: string[]): Promise<Summary>;
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
  /** Files dropped on the window, and whether (and where, in CSS pixels) something is hovering. */
  onDrop(drop: (paths: string[], at: { x: number; y: number } | null) => void, hover: (over: boolean, at: { x: number; y: number } | null) => void): Promise<() => void>;
  pick(kind: 'files' | 'folder'): Promise<string[]>;
  pickFolder(): Promise<string | null>;
  /** One image file, for a logo. */
  pickImage(): Promise<string | null>;
  reveal(path: string): Promise<void>;
  /** Open an image in the app the system uses for it. */
  openImage(path: string): Promise<void>;
  thumbUrl(path: string): string;

  integrations(): Promise<Integrations>;
  /** Put Wrangler in every file manager's right-click menu, or take it out. */
  setMenus(on: boolean): Promise<Integrations>;

  appSettings(): Promise<AppSettings>;
  setAppSettings(s: { menu_bar: boolean; auto_update: boolean; login: boolean }): Promise<AppSettings>;
  /** The recipe images dropped on the menu bar icon run. Null opens them in the window. */
  setDropRecipe(id: string | null): Promise<void>;
  /** Items from the menu bar menu that need the main window ("settings", "updates"). */
  onMenu(cb: (what: string) => void): Promise<() => void>;

  /** Look for a newer version now. */
  updateCheck(): Promise<UpdateInfo | null>;
  /** What the last look found. */
  updateFound(): Promise<UpdateInfo | null>;
  /** Told when a look in the background finds something. */
  onUpdate(cb: (u: UpdateInfo) => void): Promise<() => void>;
  /** Download, verify and install, then restart into the new version. */
  updateInstall(progress: (done: number, total: number | null) => void): Promise<void>;
  openUrl(url: string): Promise<void>;

  /** A native right-click menu at the pointer. */
  contextMenu(lines: MenuLine[]): Promise<void>;

  /** Show the window once the first frame is drawn (avoids a white flash). */
  ready(): Promise<void>;
  close(): Promise<void>;

  /** The self-test to run, when the app was started for one. */
  selftest(): Promise<SelfTest | null>;
  /** Hand back the self-test's report; the app writes it out and quits. */
  selftestDone(report: string): Promise<void>;
  /** Resize the window, if this backend can; false in a browser. */
  setWindowSize(w: number, h: number): Promise<boolean>;
}

