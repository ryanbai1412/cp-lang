import { base, open } from './browser.mjs';

const { page, errors } = await open();
let wasm = 0;
page.on('request', (r) => (wasm += r.url().endsWith('.wasm')));
await page.goto(base + '/', { waitUntil: 'networkidle' });
if (wasm) {
  console.error('home fetched cpl.wasm before anything ran');
  process.exit(1);
}
console.log('home loads without cpl.wasm');
await page.getByRole('button', { name: 'Run' }).first().click();
const out = page.getByRole('status').first();
await out.filter({ hasText: /^115\s*$/ }).waitFor({ timeout: 30000 });
console.log('home run output:', JSON.stringify(await out.innerText()));
const [viewer] = await Promise.all([
  page.context().waitForEvent('page'),
  page.getByRole('button', { name: 'Trace' }).first().click(),
]);
await viewer.getByText('main.cp').first().waitFor({ state: 'attached', timeout: 30000 });
console.log('trace viewer ok');
const replay = await viewer.evaluate(async () => {
  const original = 'fn f(x: Int) -> Int:\n    x + 1\nprint(f(1))\n';
  const edited = original.replace('x + 1', 'x + 2');
  if (sessionStorage.getItem('cp-live-input') === null) return { error: 'no cp-live-input' };
  let last;
  for (let step = 0; step < 8; step++) {
    last = await CpCompiler.call('replay', original, edited, '', step, 0);
    if (last.returned) return last;
  }
  return last;
});
if (replay.returned?.before !== 2 || replay.returned?.after !== 3) {
  console.error('replay:', JSON.stringify(replay));
  process.exit(1);
}
console.log('trace viewer replays a call with an edited fn');
await viewer.close();
await page.getByRole('button', { name: 'Stress' }).first().click();
await page.getByText('Shrunk to the smallest input that still fails').waitFor({ timeout: 60000 });
console.log('stress finds and shrinks the planted bug');
await page.goto(base + '/');
await page.locator('.cm-content').first().click();
await page.keyboard.press('Control+End');
await page.keyboard.type('\nprint(nope)');
await page.locator('.cm-lint-marker-error').first().waitFor({ timeout: 30000 });
console.log('an edit on home is checked live');
await page.goto(base + '/play');
await page.getByRole('button', { name: 'Run' }).click();
await page.getByRole('status').filter({ hasText: '115' }).waitFor({ timeout: 30000 });
console.log('play run output ok');
// C++17 and Explain come from cpl-build.wasm, which the worker loads on first use.
await page.getByRole('tab', { name: 'C++17' }).click();
await page.getByText('#include').first().waitFor({ timeout: 60000 });
await page.getByRole('tab', { name: 'Explain' }).click();
await page.getByText('Explaining…').waitFor({ state: 'detached', timeout: 60000 });
console.log('play C++17 and Explain load cpl-build.wasm');
wasm = 0;
await page.goto(base + '/docs/guides/tour', { waitUntil: 'networkidle' });
if (wasm) {
  console.error('the tour fetched cpl.wasm before a Run');
  process.exit(1);
}
await page.locator('.live-code').getByRole('button', { name: 'Run' }).first().click();
const live = page.locator('.live-code [role=status]').first();
await live.filter({ hasNotText: 'Loading' }).waitFor({ timeout: 30000 });
if (!(await live.innerText()).trim() || (await live.getAttribute('class')).includes('text-destructive')) {
  console.error('tour run:', JSON.stringify(await live.innerText()));
  process.exit(1);
}
console.log('a guide block runs in place, wasm loaded on Run:', JSON.stringify(await live.innerText()));
for (const [query, url] of [['Monoid', '/docs/ref/std/algebra'], ['borrow it', '/docs/guides/borrowing'], ['last_true', '/docs/ref/core/04-indices']]) {
  await page.goto(base + '/docs/guides/tour');
  await page.getByRole('button', { name: /^Search/ }).first().click();
  await page.getByRole('combobox', { name: 'Search' }).fill(query);
  await page.locator(`[role=option][id^="fd-search-option-${url}"]`).first().waitFor({ timeout: 30000 });
}
console.log('search finds a reference item, a guide heading and a core table row');
// A share link carries the program and its input in the URL; opening it restores both.
const shared = Buffer.from(JSON.stringify({ v: 1, src: 'let n: Int = read()\nprint(n * 7)\n', in: '6\n' })).toString('base64url');
await page.goto(base + '/play#c=' + shared);
await page.getByText('print(n * 7)').waitFor({ timeout: 30000 });
await page.getByRole('button', { name: 'Run' }).click();
await page.getByRole('status').filter({ hasText: '42' }).waitFor({ timeout: 30000 });
console.log('a share link restores its program and input');
// Lint pages run their examples: Lint on Flagged reports the lint, on Write instead it reports none.
await page.goto(base + '/lints/search_loop');
const [flagged, instead] = [page.locator('section').filter({ hasText: 'Flagged' }), page.locator('section').filter({ hasText: 'Write instead' })];
await flagged.getByRole('button', { name: 'Lint' }).click();
await flagged.getByRole('status').filter({ hasText: 'L0501' }).waitFor({ timeout: 30000 });
await instead.getByRole('button', { name: 'Lint' }).click();
await instead.getByRole('status').filter({ hasText: 'no warnings' }).waitFor({ timeout: 30000 });
await instead.getByRole('button', { name: 'Run' }).click();
await instead.getByRole('status').filter({ hasText: 'Yes' }).waitFor({ timeout: 30000 });
console.log('a lint page lints and runs its examples');
await page.close();
console.log('page errors:', errors);
process.exit(errors.length ? 1 : 0);
