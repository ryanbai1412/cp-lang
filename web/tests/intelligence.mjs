// Hover, completion and go to definition in a site editor come from the
// language server in the wasm build.
import { base, open } from './browser.mjs';

const { page, errors } = await open();
const fail = (what) => {
  console.error(`intelligence: ${what}`);
  process.exit(1);
};
await page.goto(base + '/play', { waitUntil: 'networkidle' });
const source = 'use std.segtree.*\nfn twice(x: Int) -> Int = 2 * x\nlet t = SegTree(min, [3, 1, 2])\nprint(twice(t.fold(0..2)))\nprint(t.)\n';
const editor = page.locator('.cm-content').first();
await editor.click();
await page.keyboard.press('ControlOrMeta+A');
await page.keyboard.insertText(source);
await page.keyboard.press('Escape');
// The box of character `at` of the `nth` occurrence of `needle` in the editor.
const coords = (needle, nth = 0, at = 1) =>
  page.evaluate(
    ([needle, nth, at]) => {
      for (const line of document.querySelectorAll('.cm-content .cm-line')) {
        let i = -1;
        while ((i = line.textContent.indexOf(needle, i + 1)) >= 0) {
          if (nth-- > 0) continue;
          let target = i + at;
          const walk = document.createTreeWalker(line, NodeFilter.SHOW_TEXT);
          for (let n = walk.nextNode(); n; n = walk.nextNode()) {
            if (target < n.length) {
              const r = document.createRange();
              r.setStart(n, target);
              r.setEnd(n, target + 1);
              const b = r.getBoundingClientRect();
              return { x: b.left + 2, left: b.left, y: (b.top + b.bottom) / 2 };
            }
            target -= n.length;
          }
        }
      }
      return null;
    },
    [needle, nth, at],
  );
const selected = () =>
  page.evaluate(() => {
    const s = getSelection();
    const lines = [...document.querySelectorAll('.cm-content .cm-line')];
    return { text: s.toString(), line: lines.indexOf(s.anchorNode?.parentElement?.closest('.cm-line')) + 1 };
  });

const t = await coords('t.fold');
await page.mouse.move(t.x, t.y);
const hover = page.locator('.cm-tooltip-hover .cm-lsp-doc').first();
await hover.waitFor({ timeout: 30000 }).catch(() => fail('no hover on `t`'));
if (!/SegTree</.test(await hover.innerText())) fail(`hover on \`t\` says ${JSON.stringify(await hover.innerText())}`);
console.log('hover:', JSON.stringify((await hover.innerText()).split('\n')[0]));

await page.mouse.move(0, 0);
const dot = await coords('t.)', 0, 2);
await page.mouse.click(dot.left + 0.5, dot.y);
await page.keyboard.press('Control+Space');
const menu = page.locator('.cm-tooltip-autocomplete').first();
await menu.getByText('fold', { exact: true }).first().waitFor({ timeout: 30000 }).catch(() => fail('no `fold` completion after `t.`'));
console.log('completion after `t.` offers fold');
await page.keyboard.press('Escape');

// Over CDP a click's own modifiers don't reach the page; hold the key instead.
const ctrlClick = async ({ x, y }) => {
  await page.keyboard.down('Control');
  await page.mouse.click(x, y);
  await page.keyboard.up('Control');
};
const call = await coords('twice', 1);
await ctrlClick(call);
let sel;
for (let i = 0; i < 60 && !((sel = await selected()).text === 'twice' && sel.line === 2); i++) await page.waitForTimeout(250);
if (sel.text !== 'twice' || sel.line !== 2) fail(`ctrl-click on a call selected ${JSON.stringify(sel)}`);
console.log('go to definition selects `twice` on line 2');

const ty = await coords('SegTree(');
const [docs] = await Promise.all([page.context().waitForEvent('page', { timeout: 30000 }), ctrlClick(ty)]).catch(() =>
  fail('ctrl-click on a std name opened no page'),
);
await docs.waitForLoadState('domcontentloaded');
if (!docs.url().endsWith('/docs/ref/std/segtree#segtree')) fail(`std definition opened ${docs.url()}`);
console.log('go to definition on SegTree opens', new URL(docs.url()).pathname + new URL(docs.url()).hash);
await docs.close();

// A guide's block becomes an editor on Edit, with the same intelligence.
await page.goto(base + '/docs/guides/tour', { waitUntil: 'networkidle' });
await page.locator('.live-code').first().getByRole('button', { name: 'Edit' }).click();
const block = page.locator('.live-code .cm-content').first();
await block.waitFor({ timeout: 10000 }).catch(() => fail('Edit on a guide block shows no editor'));
await block.click();
await page.keyboard.press('ControlOrMeta+End');
await page.keyboard.insertText('\nprint(ma');
await page.keyboard.press('Control+Space');
await page.locator('.cm-tooltip-autocomplete').getByText('max', { exact: true }).first().waitFor({ timeout: 30000 }).catch(() => fail('no completion in an edited guide block'));
console.log('a guide block edits in place, with completion');
if (errors.length) fail(`page errors: ${errors.join('; ')}`);
await page.close();
process.exit(0);
