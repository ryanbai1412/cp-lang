// One browser page for a smoke test: an already running Chrome when CDP is
// set, otherwise a headless Chromium that Playwright starts.
import { chromium } from '@playwright/test';

export const base = process.env.BASE ?? 'http://127.0.0.1:3000';

export async function open() {
  const browser = process.env.CDP ? await chromium.connectOverCDP(process.env.CDP) : await chromium.launch();
  const context = browser.contexts()[0] ?? (await browser.newContext());
  const page = await context.newPage();
  const errors = [];
  page.on('pageerror', (e) => errors.push(e.message));
  return { page, errors };
}
