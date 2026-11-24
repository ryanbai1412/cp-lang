// One browser page for a smoke test: the chromium run.mjs shares between
// suites (PW_WS), an already running Chrome when CDP is set, otherwise a
// headless Chromium that Playwright starts.
import { chromium } from '@playwright/test';

export const base = process.env.BASE ?? 'http://127.0.0.1:3000';

let current = null;

export async function open() {
  const browser = process.env.PW_WS
    ? await chromium.connect(process.env.PW_WS)
    : process.env.CDP
      ? await chromium.connectOverCDP(process.env.CDP)
      : await chromium.launch();
  const context = (!process.env.PW_WS && browser.contexts()[0]) || (await browser.newContext());
  const page = await context.newPage();
  const errors = [];
  page.on('pageerror', (e) => errors.push(e.message));
  // A draft save changes no DOM, so storage writes wake `until` too.
  await page.addInitScript(() => {
    const set = Storage.prototype.setItem;
    Storage.prototype.setItem = function (...args) {
      set.apply(this, args);
      document.dispatchEvent(new Event('cpl-storage'));
    };
  });
  current = page;
  return { page, errors };
}

/** The last thing a suite does: prints the verdict line run.mjs waits for, then exits. */
export function done(code = 0) {
  console.log(`${RESULT} ${code ? 'fail' : 'pass'}`);
  process.exit(code);
}

export const RESULT = 'suite-result:';

/** The one wait in the browser suites: rechecks `ready` each time the page
 * changes (a DOM mutation, a storage write, a navigation) until it gives
 * something truthy or `ms` pass, and returns its last answer. A check that
 * something does not happen passes `ms` as how long it must stay away. */
export async function until(ready, ms = 10000) {
  const end = Date.now() + ms;
  for (;;) {
    const got = await ready();
    const left = end - Date.now();
    if (got || left <= 0) return got;
    await changed(left);
  }
}

/** Resolves at the page's next change, or after `ms` with none. */
function changed(ms) {
  const navigated = current.waitForEvent('framenavigated', { timeout: ms }).catch(() => {});
  const mutated = current
    .evaluate(
      (ms) =>
        new Promise((wake) => {
          const stop = () => {
            seen.disconnect();
            document.removeEventListener('cpl-storage', stop);
            wake();
          };
          const seen = new MutationObserver(stop);
          seen.observe(document, { subtree: true, childList: true, attributes: true, characterData: true });
          document.addEventListener('cpl-storage', stop);
          AbortSignal.timeout(ms).addEventListener('abort', stop);
        }),
      ms,
    )
    .catch(() => {});
  return Promise.race([navigated, mutated]);
}
