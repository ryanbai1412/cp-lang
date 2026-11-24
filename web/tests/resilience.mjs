// handbook/guidelines/site-checklist.md site/R17: when a service the site
// calls fails, the page degrades and says so instead of breaking.
import { base, done, open, until } from './browser.mjs';

const { page, errors } = await open();
const fail = (m) => {
  console.error(`resilience: ${m}`);
  done(1);
};
await page.context().grantPermissions(['clipboard-read', 'clipboard-write'], { origin: new URL(base).origin });

// The share API is down: the full link, which needs no service, is copied instead.
await page.route('**/api/share', (r) => r.fulfill({ status: 503, contentType: 'application/json', body: '{"error":"down"}' }));
await page.goto(base + '/play');
await page.getByRole('button', { name: 'Share' }).click();
await page.getByRole('menuitem', { name: /short link/ }).click();
await page.getByText(/full link copied/).waitFor({ timeout: 10000 }).catch(() => fail('a failed short link copies nothing'));
const copied = await page.evaluate(() => navigator.clipboard.readText());
if (!copied.includes('/play#c=')) fail(`a failed short link copied ${JSON.stringify(copied)}`);
console.log('share API down: the full link is copied');

// A short link's API is down: the playground says so and still runs. The
// worker serves /p/<id> as the playground page; the static server here doesn't.
await page.route('**/api/p/*', (r) => r.abort());
await page.route('**/p/abcdefghij', async (r) => r.fulfill({ response: await r.fetch({ url: base + '/play' }) }));
await page.goto(base + '/p/abcdefghij');
await page.getByText('This share link could not be loaded.').waitFor({ timeout: 10000 }).catch(() => fail('an unreadable short link shows no notice'));
await page.getByRole('button', { name: 'Run' }).click();
await page.getByRole('status').filter({ hasText: '115' }).waitFor({ timeout: 30000 }).catch(() => fail('the playground does not run after an unreadable short link'));
console.log('short link unreadable: the playground says so and runs');

// cpl.wasm can't be fetched: Run says why, and runs once it can.
await page.route('**/*.wasm', (r) => r.abort());
await page.goto(base + '/play');
await page.getByRole('button', { name: 'Run' }).click();
const status = page.getByRole('status').first();
let said = '';
await until(async () => /load|network|fetch/i.test((said = (await status.innerText().catch(() => '')).trim())), 8000);
console.log(`wasm down: Run shows ${JSON.stringify(said.slice(0, 200))}`);
if (!/load|network|fetch/i.test(said)) fail('Run with no cpl.wasm does not say the compiler failed to load');
await page.unroute('**/*.wasm');
await page.getByRole('button', { name: 'Run' }).click();
await status.filter({ hasText: '115' }).waitFor({ timeout: 30000 }).catch(() => fail('Run does not recover once cpl.wasm loads'));
console.log('wasm down: Run says so, and the next Run after it loads works');

// A std module's text is fetched on its first use; when that fails Run says
// which module, and the next Run after it loads works.
await page.route('**/wasm/std/*.cp', (r) => r.abort());
await page.goto(base + '/play');
await page.locator('.cm-content').first().click();
await page.keyboard.press('Control+A');
await page.keyboard.insertText('use std.sparse.{SparseTable}\nprint(SparseTable(min, [3, 1, 4]).fold(0..3))\n');
await page.getByRole('button', { name: 'Run' }).click();
const stdError = page.getByText(/can't load std\.\w+/).first();
await stdError.waitFor({ timeout: 30000 }).catch(() => fail('Run with std unreachable does not say which module failed to load'));
console.log(`std down: Run shows ${JSON.stringify((await stdError.innerText()).trim().slice(0, 200))}`);
await page.unroute('**/wasm/std/*.cp');
await page.getByRole('button', { name: 'Run' }).click();
await page
  .waitForFunction(() => /\nOutput\n+1\n*$/.test(document.body.innerText), null, { timeout: 30000 })
  .catch(() => fail('Run does not recover once std loads'));
console.log('std down: Run names the module, and the next Run after it loads works');

if (errors.length) fail(`page errors: ${errors.join('; ')}`);
done(0);
