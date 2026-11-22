import { base, open } from './browser.mjs';

const { page, errors } = await open();
await page.goto(base + '/');
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
await page.goto(base + '/play');
await page.getByRole('button', { name: 'Run' }).click();
await page.getByRole('status').filter({ hasText: '115' }).waitFor({ timeout: 30000 });
console.log('play run output ok');
for (const [query, url] of [['Monoid', '/docs/ref/std/algebra'], ['borrow it', '/docs/guides/borrowing']]) {
  await page.goto(base + '/docs/guides/tour');
  await page.getByRole('button', { name: /^Search/ }).first().click();
  await page.getByRole('combobox', { name: 'Search' }).fill(query);
  await page.locator(`[role=option][id^="fd-search-option-${url}"]`).first().waitFor({ timeout: 30000 });
}
console.log('search finds a reference item and a guide heading');
await page.close();
console.log('page errors:', errors);
process.exit(errors.length ? 1 : 0);
