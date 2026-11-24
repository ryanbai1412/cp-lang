// Enforces site/R5, site/R6 (handbook/guidelines/site-checklist.md).
// Hover, completion and go to definition in a site editor come from the
// language server in the wasm build.
import { base, done, open, until } from "./browser.mjs";

const { page, errors } = await open();
const fail = (what) => {
  console.error(`intelligence: ${what}`);
  done(1);
};
await page.goto(base + "/play", { waitUntil: "networkidle" });
const source =
  "use std.segtree.*\nfn twice(x: Int) -> Int = 2 * x\nlet t = SegTree(min, [3, 1, 2])\nprint(twice(t.fold(0..2)))\nprint(t.)\n";
const editor = page.locator(".cm-content").first();
await editor.click();
await page.keyboard.press("ControlOrMeta+A");
await page.keyboard.insertText(source);
await page.keyboard.press("Escape");
// The box of character `at` of the `nth` occurrence of `needle` in the editor.
const coords = (needle, nth = 0, at = 1) =>
  page.evaluate(
    ([needle, nth, at]) => {
      for (const line of document.querySelectorAll(".cm-content .cm-line")) {
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
    const lines = [...document.querySelectorAll(".cm-content .cm-line")];
    return {
      text: s.toString(),
      line: lines.indexOf(s.anchorNode?.parentElement?.closest(".cm-line")) + 1,
    };
  });

const t = await coords("t.fold");
await page.mouse.move(t.x, t.y);
const hover = page.locator(".cm-tooltip-hover .cm-lsp-doc").first();
await hover.waitFor({ timeout: 30000 }).catch(() => fail("no hover on `t`"));
if (!/SegTree</.test(await hover.innerText()))
  fail(`hover on \`t\` says ${JSON.stringify(await hover.innerText())}`);
console.log("hover:", JSON.stringify((await hover.innerText()).split("\n")[0]));

const fold = await coords("fold(");
await page.mouse.move(0, 0);
await page.mouse.move(fold.x, fold.y);
await page
  .locator(".cm-tooltip-hover .cm-lsp-doc", {
    hasText: "The fold of the values",
  })
  .first()
  .waitFor({ timeout: 30000 })
  .catch(() => fail("hover on a std method shows no doc text"));
console.log("hover on fold shows its std doc");
await page.mouse.move(0, 0);
const dot = await coords("t.)", 0, 2);
await page.mouse.click(dot.left + 0.5, dot.y);
await page.keyboard.press("Control+Space");
const menu = page.locator(".cm-tooltip-autocomplete").first();
await menu
  .getByText("fold", { exact: true })
  .first()
  .waitFor({ timeout: 30000 })
  .catch(() => fail("no `fold` completion after `t.`"));
console.log("completion after `t.` offers fold");
await page.keyboard.press("Escape");

// Over CDP a click's own modifiers don't reach the page; hold the key instead.
const ctrlClick = async ({ x, y }) => {
  await page.keyboard.down("Control");
  await page.mouse.click(x, y);
  await page.keyboard.up("Control");
};
const call = await coords("twice", 1);
await ctrlClick(call);
let sel;
await until(async () => (sel = await selected()).text === "twice" && sel.line === 2, 15000);
if (sel.text !== "twice" || sel.line !== 2)
  fail(`ctrl-click on a call selected ${JSON.stringify(sel)}`);
console.log("go to definition selects `twice` on line 2");

const ty = await coords("SegTree(");
const [docs] = await Promise.all([
  page.context().waitForEvent("page", { timeout: 30000 }),
  ctrlClick(ty),
]).catch(() => fail("ctrl-click on a std name opened no page"));
await docs.waitForLoadState("domcontentloaded");
if (!docs.url().endsWith("/docs/ref/std/segtree#segtree"))
  fail(`std definition opened ${docs.url()}`);
console.log(
  "go to definition on SegTree opens",
  new URL(docs.url()).pathname + new URL(docs.url()).hash,
);
await docs.close();

// Inlay hints: the type of an untyped `let`, and a toolbar switch to hide them.
const hints = page.locator(".cm-inlay-hint");
await page
  .waitForFunction(
    () => [...document.querySelectorAll(".cm-inlay-hint")].some((h) => /SegTree/.test(h.textContent)),
    null,
    { timeout: 30000 },
  )
  .catch(async () => fail(`no SegTree inlay hint on \`let t\`: ${JSON.stringify(await hints.allInnerTexts())}`));
console.log("inlay hints:", JSON.stringify(await hints.allInnerTexts()));
await page.getByTitle("Hide inlay type hints").click();
await page
  .waitForFunction(() => !document.querySelector(".cm-inlay-hint"), null, { timeout: 10000 })
  .catch(() => fail("inlay hints stay after switching them off"));
await page.reload({ waitUntil: "networkidle" });
await page.locator(".cm-content").first().click();
await page.keyboard.press("ControlOrMeta+A");
await page.keyboard.insertText(source);
if (await until(() => hints.count(), 1500)) fail("inlay hints came back after a reload with them off");
await page.getByTitle("Show inlay type hints").click();
await hints.first().waitFor({ timeout: 30000 }).catch(() => fail("inlay hints don't come back on"));
console.log("inlay hints switch off, stay off across a reload, and back on");

await page.locator(".cm-content").first().click();
await page.keyboard.press("ControlOrMeta+End");
await page.keyboard.insertText("\nprint(twice(");
const sig = page.locator(".cm-signature-help");
await sig.waitFor({ timeout: 30000 }).catch(() => fail("no parameter hints after typing `twice(`"));
if ((await sig.locator("b").innerText()) !== "x: Int") fail(`parameter hints for \`twice(\` don't bold \`x: Int\`: ${await sig.innerHTML()}`);
console.log("parameter hints:", await sig.innerText());
// A std function's hints list its named options, as the language server's do.
await page.keyboard.press("ControlOrMeta+A");
await page.keyboard.insertText("let a = [3, 1, 2]\nprint(a.sorted(");
await page.waitForFunction(() => /by/.test(document.querySelector(".cm-signature-help")?.textContent ?? ""), null, { timeout: 30000 })
  .catch(async () => fail(`parameter hints for \`sorted(\` don't list \`by:\`: ${await sig.innerText().catch(() => "none")}`));
console.log("std parameter hints:", await sig.innerText());
await page.keyboard.press("Escape");

// Ctrl/Cmd-click on a search result opens it in a new tab.
await page.keyboard.press("ControlOrMeta+k");
await page.keyboard.insertText("SegTree");
const result = page.locator("#fd-search-list [role=option]").first();
await result.waitFor({ timeout: 30000 }).catch(() => fail("search for SegTree shows no result"));
const here = page.url();
const [tab] = await Promise.all([
  page.context().waitForEvent("page", { timeout: 15000 }),
  result.click({ modifiers: ["ControlOrMeta"] }),
]).catch(() => fail("ctrl-click on a search result opened no tab"));
await tab.waitForLoadState("domcontentloaded");
if (!new URL(tab.url()).pathname.startsWith("/docs/")) fail(`search result tab opened ${tab.url()}`);
if (page.url() !== here) fail(`ctrl-click on a search result also navigated this tab to ${page.url()}`);
console.log("ctrl-click on a search result opens", new URL(tab.url()).pathname);
await tab.close();
await page.keyboard.press("Escape");

// A guide's block becomes an editor on Edit, with the same intelligence.
await page.goto(base + "/docs/guides/tour", { waitUntil: "networkidle" });
await page
  .locator(".live-code")
  .first()
  .getByRole("button", { name: "Edit" })
  .click();
const block = page.locator(".live-code .cm-content").first();
await block
  .waitFor({ timeout: 10000 })
  .catch(() => fail("Edit on a guide block shows no editor"));
await block.click();
await page.keyboard.press("ControlOrMeta+End");
await page.keyboard.insertText("\nprint(ma");
await page.keyboard.press("Control+Space");
await page
  .locator(".cm-tooltip-autocomplete")
  .getByText("max", { exact: true })
  .first()
  .waitFor({ timeout: 30000 })
  .catch(() => fail("no completion in an edited guide block"));
console.log("a guide block edits in place, with completion");

// A std reference example is an editor where it stands, with the language server's hover (site/R21).
await page.goto(base + "/docs/ref/std/sparse", { waitUntil: "networkidle" });
const example = page.locator(".cm-editor", { hasText: "SparseTable(min" }).first();
await example
  .scrollIntoViewIfNeeded({ timeout: 10000 })
  .catch(() => fail("the std reference example is not an editor"));
// Hover loads the compiler only once the reader works in the editor.
await example.locator(".cm-content").click();
const sparse = await coords("SparseTable(min", 0, 2);
const typed = page.locator(".cm-tooltip-hover .cm-lsp-doc", { hasText: "SparseTable" }).first();
for (let i = 0; i < 30 && !(await typed.isVisible()); i++) {
  await page.mouse.move(0, 0);
  await page.mouse.move(sparse.x, sparse.y);
  await typed.waitFor({ timeout: 1000 }).catch(() => {});
}
if (!(await typed.isVisible())) fail("hover in a std reference example shows no type");
console.log("std reference example hovers:", JSON.stringify((await typed.innerText()).split("\n")[0]));
if (errors.length) fail(`page errors: ${errors.join("; ")}`);
await page.close();
done(0);
