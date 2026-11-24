// Enforces site/R5 (handbook/guidelines/site-checklist.md) for complete/U27-U32:
// completion works the same in every kind of site editor, says when the
// compiler isn't there, and fits a phone.
import { base, done, open, until } from './browser.mjs';

const { page, errors } = await open();
const fail = (what) => {
  console.error(`completion-surfaces: ${what}`);
  done(1);
};
const menu = page.locator('.cm-tooltip-autocomplete');
const labels = () => menu.locator('.cm-completionLabel').allInnerTexts().catch(() => []);
async function shows(what, test, ms = 30000) {
  const end = Date.now() + ms;
  let got = [];
  while (Date.now() < end) {
    got = await labels();
    if (test(got)) return got;
    await page.waitForTimeout(200);
  }
  fail(`${what}: the menu shows ${JSON.stringify(got.slice(0, 8))}`);
}
const editable = () => page.locator('.cm-content[aria-label="CP Lang source"]').first();
async function typeAtEnd(text) {
  await editable().click();
  await page.keyboard.press('ControlOrMeta+End');
  await page.keyboard.insertText(text);
}
const lastLine = () => editable().locator('.cm-line').last().innerText();

// U27, U28: a std reference example, whose `use std.sparse.*` is hidden setup.
await page.goto(base + '/docs/ref/std/sparse', { waitUntil: 'networkidle' });
await typeAtEnd('\nlet s = Spars');
await page.keyboard.press('Control+Space');
await shows('std example, a name from the hidden setup', (l) => l.includes('SparseTable'));
// U31: Enter accepts, at the cursor rather than shifted by the setup's lines.
await page.keyboard.press('Enter');
if ((await lastLine()) !== 'let s = SparseTable') fail(`Enter in the std example made ${JSON.stringify(await lastLine())}`);
console.log('std reference example: offers the hidden setup\'s names; Enter accepts in place');

// U28: the practice judge editor; U31: Tab accepts, Esc closes.
await page.goto(base + '/learn/practice/prime-count', { waitUntil: 'networkidle' });
await typeAtEnd('\nlet xs = [3, 1, 2]\nprint(xs.');
await shows('practice editor after `xs.`', (l) => l.includes('len'));
await page.keyboard.insertText('le');
await shows('practice editor after `xs.le`', (l) => l[0] === 'len');
await page.keyboard.press('Tab');
if ((await lastLine()) !== 'print(xs.len') fail(`Tab in the practice editor made ${JSON.stringify(await lastLine())}`);
await page.keyboard.insertText(')\nprint(xs.');
await shows('practice editor, second `xs.`', (l) => l.includes('len'));
await page.keyboard.press('Escape');
await menu.waitFor({ state: 'hidden', timeout: 5000 }).catch(() => fail('Esc leaves the menu open'));
console.log('practice editor: offers members; Tab accepts, Esc closes');

// U29: the read-only reference solution offers nothing.
await page.getByRole('button', { name: 'Show the reference solution' }).click();
const example = page.locator('.cm-content[aria-label="CP Lang example"]').first();
await example.click();
await page.keyboard.press('ControlOrMeta+End');
await page.keyboard.press('Control+Space');
if (await until(() => menu.isVisible(), 1500)) fail('the read-only reference solution opens a completion menu');
console.log('read-only example: no menu');

// U30: before cpl.wasm loads the menu says so, then fills in.
let release;
const held = new Promise((go) => (release = go));
await page.route('**/*.wasm', async (r) => {
  await held;
  await r.continue().catch(() => {});
});
await page.goto(base + '/play');
await typeAtEnd('\nlet xs = [3, 1, 2]\nprint(xs.');
await shows('loading', (l) => l.length === 1 && /Loading the compiler/.test(l[0]), 4000);
release();
await shows('after loading', (l) => l.includes('len'));
await page.unroute('**/*.wasm');
console.log('cpl.wasm still loading: the menu says so, then shows the members');

// U30: when cpl.wasm can't be fetched the menu says so; Ctrl+Space retries.
await page.route('**/*.wasm', (r) => r.abort());
await page.goto(base + '/play');
await typeAtEnd('\nlet xs = [3, 1, 2]\nprint(xs.');
await shows('wasm down', (l) => l.length === 1 && /didn't load/.test(l[0]));
await page.unroute('**/*.wasm');
await page.keyboard.press('Control+Space');
await shows('after cpl.wasm comes back', (l) => l.includes('len'));
console.log("cpl.wasm down: the menu says it didn't load, and Ctrl+Space recovers");

// A std item from a module nothing has loaded yet shows its docs: typing
// `Spa` loads std.sparse, but the menu also lists std.interval's
// SparseIntervalMap, whose module (and so its docs) is fetched only when the
// item shows its detail (completionItem/resolve).
const fetched = [];
page.on('request', (r) => r.url().endsWith('/wasm/std/interval.cp') && fetched.push(r.url()));
await page.goto(base + '/play', { waitUntil: 'networkidle' });
await typeAtEnd('\nlet t = Spa');
await page.keyboard.press('Control+Space');
await shows('a std name from a module not loaded yet', (l) => l.includes('SparseIntervalMap'));
if (fetched.length) fail('std.interval was fetched before an item from it showed its detail');
const selected = () => menu.locator('[aria-selected=true] .cm-completionLabel').innerText().catch(() => null);
for (let i = 0; i < 12 && (await selected()) !== 'SparseIntervalMap'; i++) await page.keyboard.press('ArrowDown');
if ((await selected()) !== 'SparseIntervalMap') fail(`ArrowDown never selected SparseIntervalMap (at ${await selected()})`);
const info = page.locator('.cm-completionInfo');
await info.filter({ hasText: 'Interval partition map' }).waitFor({ timeout: 20000 })
  .catch(async () => fail(`SparseIntervalMap's detail shows ${JSON.stringify(await info.innerText().catch(() => null))}, not its docs`));
if (fetched.length !== 1) fail(`std.interval was fetched ${fetched.length} times`);
await page.keyboard.press('Escape');
console.log('std item from a module not loaded yet: its detail fetches the module and shows its docs');

// U32: on a phone the menu and its detail stay inside the screen.
await page.setViewportSize({ width: 375, height: 740 });
await page.goto(base + '/play', { waitUntil: 'networkidle' });
await typeAtEnd('\nlet xs = [3, 1, 2]\nprint(xs.');
await shows('phone', (l) => l.includes('len'));
await page.keyboard.insertText('le');
await page.locator('.cm-completionInfo').waitFor({ timeout: 10000 }).catch(() => fail('phone: no detail for `len`'));
for (const [name, sel] of [['menu', '.cm-tooltip-autocomplete'], ['detail', '.cm-completionInfo']]) {
  const b = await page.locator(sel).first().boundingBox();
  if (!b || b.x < 0 || b.x + b.width > 375 + 1) fail(`phone: the ${name} spans x ${b?.x}..${b && b.x + b.width} of 375`);
}
console.log('phone: the menu and its detail fit 375 px');

if (errors.length) fail(`page errors: ${errors.join('; ')}`);
done(0);
