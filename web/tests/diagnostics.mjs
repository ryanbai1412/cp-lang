// Enforces site/R6 (handbook/guidelines/site-checklist.md): the playground shows
// the error `cpl check` raises, while you type. For a sample of the error
// registry's triggers (`cpl explain --json`), the editor must show the same
// code within 1.5 s of the last keystroke.
import { execFileSync } from "node:child_process";
import { existsSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { base, open } from "./browser.mjs";

const repo = dirname(dirname(dirname(fileURLToPath(import.meta.url))));
const cpl = process.env.CPL ?? ["target/release/cpl", "target/debug/cpl"].map((p) => join(repo, p)).find(existsSync) ?? "cpl";
const codes = JSON.parse(execFileSync(cpl, ["explain", "--json"], { encoding: "utf8" }))
  .filter((c) => c.mode === "check" && c.trigger && c.code.startsWith("E"));
const sample = codes.filter((_, i) => i % Math.ceil(codes.length / 12) === 0);
const budget = 1500;

const { page, errors } = await open();
const fail = (what) => {
  console.error(`diagnostics: ${what}`);
  process.exit(1);
};
await page.goto(base + "/play", { waitUntil: "networkidle" });
const editor = page.locator(".cm-content").first();
const shown = () => page.evaluate(() => [...document.querySelectorAll(".cm-lintRange-error, .cm-lintRange-warning")].length);
// The codes in the lint tooltip over the first marked range.
async function shownCodes() {
  const box = await page.locator(".cm-lintRange-error, .cm-lintRange-warning").first().boundingBox();
  if (!box) return [];
  await page.mouse.move(box.x + Math.min(box.width / 2, 4), box.y + box.height / 2);
  await page.locator(".cm-tooltip-lint .cm-diagnostic").first().waitFor({ timeout: 3000 }).catch(() => {});
  const codes = await page.evaluate(() => [...document.querySelectorAll(".cm-tooltip-lint .cm-diagnosticSource")].map((e) => e.textContent.trim()));
  await page.mouse.move(0, 0);
  return codes;
}

// The first check loads the compiler; it is not what a contestant waits for while typing.
await editor.click();
await page.keyboard.insertText(" ");
await page.waitForTimeout(3000);
const times = [];
for (const c of sample) {
  await editor.click();
  await page.keyboard.press("ControlOrMeta+A");
  await page.keyboard.press("Delete");
  await page.waitForFunction(() => !document.querySelector(".cm-lintRange-error"), null, { timeout: 5000 }).catch(() => {});
  await page.keyboard.insertText(c.trigger);
  const t0 = Date.now();
  await page.waitForFunction(() => document.querySelector(".cm-lintRange-error"), null, { timeout: 10000 })
    .catch(() => fail(`${c.code}: no error shown for its trigger ${JSON.stringify(c.trigger)}`));
  times.push([c.code, Date.now() - t0]);
  const got = await shownCodes();
  if (!got.includes(c.code)) fail(`${c.code}: the editor shows ${JSON.stringify(got)}, \`cpl check\` raises ${c.code}`);
}
const slow = times.filter(([, t]) => t > budget);
if (slow.length) fail(`errors shown later than ${budget} ms after the last keystroke: ${slow.map(([c, t]) => `${c} ${t} ms`).join(", ")}`);
if (errors.length) fail(`page errors: ${errors.join("; ")}`);
const ms = times.map(([, t]) => t).sort((a, b) => a - b);
console.log(`diagnostics: ${times.length} registry triggers show their code; ${ms[ms.length >> 1]} ms median, ${ms[ms.length - 1]} ms max`);
process.exit(0);
