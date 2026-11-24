// Enforces site/R12 (handbook/guidelines/site-checklist.md): an edit in the
// playground, an example or a practice problem survives a reload; Reset forgets it.
import { base, done, open, until } from "./browser.mjs";

const { page, errors } = await open();
const fail = (what) => {
  console.error(`drafts: ${what}`);
  done(1);
};
const editor = () => page.locator(".cm-content").first();
const text = () => editor().evaluate((e) => e.cmView?.view?.state.doc.toString() ?? e.innerText);
const mark = "// kept after reload";

for (const path of ["/play", "/learn/practice/prime-count"]) {
  await page.goto(base + path, { waitUntil: "networkidle" });
  const before = await text();
  await editor().click();
  await page.keyboard.press("ControlOrMeta+End");
  await page.keyboard.insertText(`\n${mark}`);
  await until(() => page.evaluate((m) => Object.keys(localStorage).some((k) => k.startsWith("cp-draft:") && localStorage.getItem(k).includes(m)), mark));
  await page.reload({ waitUntil: "networkidle" });
  await until(async () => (await text()).includes(mark));
  if (!(await text()).includes(mark)) fail(`${path}: the edit is gone after a reload`);
  const reset = page.locator('button[title="Reset"]').first();
  if (await reset.count()) {
    await reset.click();
    await page.reload({ waitUntil: "networkidle" });
    await until(async () => (await text()).trimEnd() === before.trimEnd());
    if ((await text()).includes(mark)) fail(`${path}: Reset did not forget the edit`);
    if ((await text()).trimEnd() !== before.trimEnd()) fail(`${path}: Reset did not bring back the starting text`);
  }
}
if (errors.length) fail(`page errors: ${errors.join("; ")}`);
console.log("drafts: edits survive a reload on /play and a practice problem; Reset forgets them");
done(0);
