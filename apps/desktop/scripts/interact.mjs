// SPDX-License-Identifier: AGPL-3.0-or-later
// Uses the interface the way a person does, in a browser with the stand-in backend, and checks
// what happened: building a name from pieces by pressing and dragging, sizing past the original,
// other shapes, and looking after recipes. Run by `pnpm selftest browser` after the layout checks.

/** Runs every check on a fresh page. Gives the checks that failed, as text. */
export async function interact(browser, base) {
  const failed = [];
  let count = 0;
  const check = (ok, what, got) => { count++; if (!ok) failed.push(`${what}${got === undefined ? '' : `: got ${JSON.stringify(got)}`}`); };
  const p = await browser.newPage({ viewport: { width: 1440, height: 900 } });
  p.on('pageerror', e => failed.push(`page error: ${e.message}`));
  await p.goto(`${base}/?selftest=layout`);
  await p.waitForFunction(() => window.__selftest, null, { timeout: 60000 });
  await p.waitForTimeout(1200);
  const R = f => p.evaluate(f);
  const pattern = () => R(() => window.__wrangler.recipe.value.rename.pattern);

  // ---- names from pieces
  await p.click('#chan-name .seg button:has-text("Build")');
  check(await pattern() === '{name}', 'Build starts from the original name', await pattern());
  await p.click('#chan-name .tray button:has-text("001")');
  check(await pattern() === '{name}-{n:000}', 'adding a number puts a dash before it', await pattern());
  // drag the number block to the front
  const from = await p.locator('#chan-name .build .block', { hasText: '001' }).boundingBox();
  const to = await p.locator('#chan-name .build .block', { hasText: 'name' }).boundingBox();
  await p.mouse.move(from.x + from.width / 2, from.y + from.height / 2);
  await p.mouse.down();
  await p.mouse.move(from.x - 10, from.y + 4, { steps: 4 });
  await p.mouse.move(to.x + 3, to.y + to.height / 2, { steps: 6 });
  await p.mouse.up();
  check(await pattern() === '{n:000}{name}-', 'dragging a block reorders the name', await pattern());
  // type into the text block
  await p.locator('#chan-name .build .block--text').click();
  await p.keyboard.type('_final');
  await p.keyboard.press('Enter');
  check(await pattern() === '{n:000}{name}_final', 'pressing a text block edits it', await pattern());
  // number options from its menu
  await p.locator('#chan-name .build .block', { hasText: '001' }).click();
  await p.click('.mock-menu button:has-text("0001, 0002")');
  check(await pattern() === '{n:0000}{name}_final', 'a number block offers its digits', await pattern());
  // drag a block away to remove it
  const n = await p.locator('#chan-name .build .block', { hasText: '0001' }).boundingBox();
  await p.mouse.move(n.x + 8, n.y + 8); await p.mouse.down(); await p.mouse.move(n.x + 30, n.y - 120, { steps: 6 }); await p.mouse.up();
  check(await pattern() === '{name}_final', 'dragging a block away removes it', await pattern());
  // drag a new piece from the tray into place
  const tray = await p.locator('#chan-name .tray button', { hasText: 'date' }).boundingBox();
  const first = await p.locator('#chan-name .build .block').first().boundingBox();
  await p.mouse.move(tray.x + 8, tray.y + 8); await p.mouse.down(); await p.mouse.move(tray.x + 12, tray.y - 10, { steps: 3 }); await p.mouse.move(first.x + 2, first.y + 8, { steps: 6 }); await p.mouse.up();
  check(await pattern() === '{date}{name}_final', 'a piece can be dragged in from the tray', await pattern());
  await p.waitForTimeout(600);
  const shown = await p.locator('#chan-name .out').textContent();
  check(/^→ 20\d\d-\d\d-\d\dDSC_4471_final\.webp$/.test(shown.trim()), 'the new name shows as it is built', shown);

  // ---- size: enlarging and other shapes
  const size = () => R(() => { const s = window.__wrangler.recipe.value.size; return { long: s.long_side, shape: s.shape, up: s.upscale }; });
  await p.click('#chan-size .value-btn'); await p.keyboard.type('9000'); await p.keyboard.press('Enter');
  check((await p.locator('#chan-size .sub').textContent()).includes('smaller: stays 6000×4000'), 'a size above the image says the image stays as it is', await p.locator('#chan-size .sub').textContent());
  await p.click('#chan-size .switch');
  check((await size()).up === true && (await p.locator('#chan-size .sub').textContent()).includes('9000×6000'), 'the switch enlarges it', await p.locator('#chan-size .sub').textContent());
  // drag the corner to the outside of the diagram: past the original
  const dia = await p.locator('#chan-size .diagram').boundingBox();
  await p.click('#chan-size .choice button:has-text("1200")');
  const h = await p.locator('#chan-size .handle').boundingBox();
  await p.mouse.move(h.x + 6, h.y + 6); await p.mouse.down(); await p.mouse.move(dia.x + dia.width - 1, dia.y + 1, { steps: 8 }); await p.mouse.up();
  const dragged = (await size()).long;
  check(dragged > 6000 && dragged <= 12000, 'with enlarging on, the corner drags past the original (6000)', dragged);
  await p.click('#chan-size .seg button:has-text("W:H")');
  await p.getByLabel('Shape: width').fill('2'); await p.getByLabel('Shape: height').fill('3'); await p.keyboard.press('Enter');
  check((await size()).shape === '2:3', 'two small numbers make a ratio', (await size()).shape);
  check((await p.locator('#chan-size .seg button[aria-checked="true"]').textContent()).trim() === '2:3', 'and the selector shows it', await p.locator('#chan-size .seg button[aria-checked="true"]').textContent());
  await p.getByLabel('Shape: width').fill('1920'); await p.getByLabel('Shape: height').fill('1080'); await p.keyboard.press('Enter');
  check((await size()).shape === '1920x1080', 'pixels make an exact size', (await size()).shape);
  await p.click('button[aria-label="Swap width and height"]');
  check((await size()).shape === '1080x1920', 'swap turns it on its side', (await size()).shape);
  await p.keyboard.press('Escape');

  // ---- recipes
  const ids = () => R(() => window.__wrangler.recipe.entries.map(e => e.id + (e.on ? '' : ' (off)')));
  const cur = () => R(() => window.__wrangler.recipe.id);
  check(await R(() => window.__wrangler.recipe.edited), 'the recipe shows as edited');
  await p.click('.head .key:has-text("Save as new")');
  await p.keyboard.type('Tall banners'); await p.keyboard.press('Enter');
  await p.waitForTimeout(300);
  check(await cur() === 'tall-banners' && (await ids())[1] === 'tall-banners', 'Save as new makes a recipe after the one it came from', (await ids()).slice(0, 3));
  check((await p.evaluate(() => window.__wrangler.recipe.entries[0].recipe.size.shape)) === 'original', 'and leaves that one as it was');
  await p.click('.recipe__name');
  await p.waitForTimeout(300);
  await p.click('[data-row="tall-banners"] button[aria-label^="Make a copy"]');
  await p.waitForTimeout(200); await p.keyboard.type('Wide banners'); await p.keyboard.press('Enter'); await p.waitForTimeout(300);
  check((await ids())[2] === 'wide-banners', 'a copy sits after the original and takes its new name', (await ids()).slice(0, 4));
  await p.click('[data-row="just-compress"] .switch'); await p.waitForTimeout(200);
  check((await ids()).includes('just-compress (off)'), 'a recipe can be switched off');
  check(!(await R(() => window.__wrangler.recipe.shown.map(e => e.id))).includes('just-compress'), 'and then the header skips it');
  // drag the last recipe to the top
  const g = await p.locator('[data-row="wide-banners"] .grip').boundingBox();
  const top = await p.locator('[data-row="compress-for-web"]').boundingBox();
  await p.mouse.move(g.x + 8, g.y + 12); await p.mouse.down(); await p.mouse.move(g.x + 8, g.y - 20, { steps: 4 }); await p.mouse.move(top.x + 30, top.y + 4, { steps: 8 });
  await p.mouse.up(); await p.waitForTimeout(300);
  check((await ids())[0] === 'wide-banners', 'recipes drag into a new order', (await ids()).slice(0, 3));
  await p.click('[data-row="wide-banners"] button[aria-label^="Delete"]'); await p.click('[data-row="wide-banners"] .sure .key:has-text("Delete")'); await p.waitForTimeout(300);
  check(!(await ids()).includes('wide-banners'), 'a recipe is deleted after a second press');
  await p.click('[data-row="convert-to-jpeg"] button[aria-label^="Delete"]'); await p.click('[data-row="convert-to-jpeg"] .sure .key:has-text("Delete")'); await p.waitForTimeout(300);
  await p.click('.dialog .key:has-text("Bring back starters")'); await p.waitForTimeout(300);
  check((await ids()).includes('convert-to-jpeg'), 'a deleted starter comes back');
  await p.click('[data-row="1080p-16-9"] .pick'); await p.waitForTimeout(400);
  check(await cur() === '1080p-16-9' && !(await p.locator('dialog[open]').count()), 'pressing a recipe uses it and closes the list', await cur());
  // changes wait with their recipe
  await p.click('#chan-quality .switch:has-text("Lossless")');
  await p.click('.head button[aria-label="Next recipe"]'); await p.click('.head button[aria-label="Previous recipe"]'); await p.waitForTimeout(300);
  check(await R(() => window.__wrangler.recipe.edited && window.__wrangler.recipe.value.quality.look === 'lossless'), 'changes not saved wait with their recipe while you look at another');
  await p.click('.head button[aria-label^="Undo your changes"]'); await p.waitForTimeout(300);
  check(!(await R(() => window.__wrangler.recipe.edited)), 'and one key takes them back');
  await p.close();
  return { count, failed };
}
