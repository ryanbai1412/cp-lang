// complete/U18 (handbook/product/editor/completion-usecases.md): every context
// in crates/cpl-lsp/tests/completion-contexts.json offers, in the playground,
// the labels the native language server must offer there, among the options
// the menu shows (CodeMirror renders the first 100). A context that ends in
// `(`, `, ` or `{` must also open the menu as that character is typed; for a
// name in scope, which the menu after `(` leaves out, as its first letter is.
import { readFileSync } from "node:fs";
import { base, done, open } from "./browser.mjs";

const cases = JSON.parse(
  readFileSync(new URL("../../crates/cpl-lsp/tests/completion-contexts.json", import.meta.url), "utf8"),
);
const { page } = await open();
await page.goto(base + "/play", { waitUntil: "networkidle" });
await page.locator(".cm-content").first().click();
const menu = page.locator(".cm-tooltip-autocomplete").first();

async function offered(before, after, typed, more = "") {
  await page.keyboard.press("ControlOrMeta+A");
  await page.keyboard.press("Delete");
  await page.keyboard.insertText(typed ? before.slice(0, -1) : before);
  await page.keyboard.insertText(after);
  for (let i = 0; i < after.length; i++) await page.keyboard.press("ArrowLeft");
  await page.keyboard.press("Escape");
  if (typed) await page.keyboard.type(before.slice(-1) + more);
  else await page.keyboard.press("Control+Space");
  const shown = await menu.waitFor({ timeout: typed ? 10000 : 30000 }).then(() => true, () => false);
  return shown ? await menu.locator(".cm-completionLabel").allInnerTexts() : [];
}

const misses = [];
let triggers = 0;
for (const c of cases) {
  const [before, after] = c.source.split("$0");
  const ways = [false, ...(/[(,{] ?$/.test(before) ? [true] : [])];
  for (const typed of ways) {
    const more = typed && c.context.startsWith("name/") ? c.want[0][0] : "";
    const labels = await offered(before, after, typed, more);
    // As in completion_contexts.rs, a wanted `name` also matches the label `name:`.
    const missing = c.want.filter((w) => !labels.some((l) => l === w || l.replace(/:$/, "") === w));
    const name = typed ? `${c.context} (typing ${JSON.stringify(before.slice(-1) + more)})` : c.context;
    triggers += typed;
    if (missing.length) misses.push({ name, why: `no ${missing.join(", ")} (offered ${labels.slice(0, 8).join(" ") || "nothing"})` });
  }
}
for (const m of misses) console.error(`completion-parity: ${m.name}: ${m.why}`);
console.log(`completion-parity: ${cases.length} contexts and ${triggers} trigger characters, ${misses.length} misses`);
done(misses.length ? 1 : 0);
