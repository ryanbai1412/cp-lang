// Enforces site/R6 (handbook/guidelines/site-checklist.md): the playground shows
// the lints `cpl lint` raises and applies their fixes as `cpl lint` would. On the
// lint registry's triggers (`cpl explain --json`), every one with a fix and a
// sample of the rest, the editor must show the lint's code, and clicking its
// first fix must give the trigger with that fix's edits applied.
import { execFileSync } from "node:child_process";
import { mkdtempSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { base, done, open, until } from "./browser.mjs";
import { cplBin } from "../scripts/cpl-bin.mjs";

const repo = dirname(dirname(dirname(fileURLToPath(import.meta.url))));
const cpl = cplBin();
const dir = mkdtempSync(join(tmpdir(), "lint-fixes-"));
const native = (src) => {
  const f = join(dir, "t.cp");
  writeFileSync(f, src);
  let out;
  try {
    out = execFileSync(cpl, ["lint", "--json", f], { encoding: "utf8" });
  } catch (e) {
    out = e.stdout;
  }
  return JSON.parse(out).files[0];
};
const apply = (src, edits) =>
  [...edits].sort((a, b) => b.start.offset - a.start.offset).reduce((s, e) => s.slice(0, e.start.offset) + e.text + s.slice(e.end.offset), src);

const cases = JSON.parse(execFileSync(cpl, ["explain", "--json"], { encoding: "utf8" }))
  .filter((c) => c.code.startsWith("L") && c.mode === "lint" && c.trigger)
  .map((c) => ({ ...c, file: native(c.trigger) }))
  // The editor lints only a program that checks; a trigger that doesn't is the registry's bug.
  .filter((c) => c.file.ok)
  .map((c) => ({ ...c, raised: c.file.diagnostics.find((d) => d.code === c.code) }))
  .filter((c) => c.raised);
const fixed = cases.filter((c) => c.raised.fixes.length);
const step = (xs, n) => xs.filter((_, i) => i % Math.max(1, Math.ceil(xs.length / n)) === 0);
const sample = [...fixed, ...step(cases.filter((c) => !c.raised.fixes.length), 6)];

const { page, errors } = await open();
const fail = (what) => {
  console.error(`lint-fixes: ${what}`);
  done(1);
};
await page.goto(base + "/play", { waitUntil: "networkidle" });
const editor = page.locator(".cm-content").first();
// The editor's text, without the inlay hints drawn into its lines.
const text = () =>
  page.evaluate(() =>
    [...document.querySelectorAll(".cm-content .cm-line")]
      .map((l) => {
        const c = l.cloneNode(true);
        c.querySelectorAll(".cm-inlay-hint").forEach((h) => h.remove());
        return c.textContent;
      })
      .join("\n"),
  );
const ranges = page.locator(".cm-lintRange-warning, .cm-lintRange-info, .cm-lintRange-error");
// Hovers each marked range until one's tooltip lists `code`; returns the codes seen.
async function hoverFor(code) {
  const seen = [];
  for (let i = 0; i < (await ranges.count()); i++) {
    const box = await ranges.nth(i).boundingBox();
    if (!box) continue;
    await page.mouse.move(0, 0);
    await page.mouse.move(box.x + Math.min(box.width / 2, 4), box.y + box.height / 2);
    await page.locator(".cm-tooltip-lint .cm-diagnostic").first().waitFor({ timeout: 3000 }).catch(() => {});
    const codes = await page.evaluate(() => [...document.querySelectorAll(".cm-tooltip-lint .cm-diagnosticSource")].map((e) => e.textContent.trim()));
    seen.push(...codes);
    if (codes.includes(code)) return seen;
  }
  return seen;
}

let applied = 0;
for (const c of sample) {
  // Focus, not click: the last hover's tooltips can still cover the editor.
  await page.mouse.move(0, 0);
  await editor.focus();
  await page.keyboard.press("ControlOrMeta+A");
  await page.keyboard.insertText(c.trigger);
  const shown = await page
    .waitForFunction(() => document.querySelector(".cm-lintRange-warning, .cm-lintRange-info, .cm-lintRange-error"), null, { timeout: 10000 })
    .then(() => true, () => false);
  if (!shown) fail(`${c.code}: \`cpl lint\` raises it on its trigger, the editor shows nothing: ${JSON.stringify(c.trigger)}`);
  let codes = [];
  await until(async () => (codes = await hoverFor(c.code)).includes(c.code), 3000);
  if (!codes.includes(c.code)) fail(`${c.code}: the editor shows ${JSON.stringify(codes)} where \`cpl lint\` raises ${c.code}`);
  for (const fix of c.raised.fixes.slice(0, 1)) {
    const button = page.locator(".cm-tooltip-lint .cm-diagnostic", { has: page.locator(".cm-diagnosticSource", { hasText: c.code }) })
      .locator(".cm-diagnosticAction", { hasText: fix.title }).first();
    if (!(await button.count())) fail(`${c.code}: the editor offers no fix "${fix.title}"`);
    await button.click();
    const want = apply(c.trigger, fix.edits);
    const got = await text();
    if (got !== want) fail(`${c.code}: "${fix.title}" gives\n${got}\nwhere \`cpl lint\`'s edits give\n${want}`);
    applied++;
  }
}
if (errors.length) fail(`page errors: ${errors.join("; ")}`);
console.log(`lint-fixes: ${sample.length} of ${cases.length} lint triggers show their code; ${applied} fixes apply as \`cpl lint\`'s do (${fixed.length} lints have one)`);
done(0);
