import { readFileSync } from 'node:fs';
import { chromium } from '@playwright/test';

const base = process.env.BASE ?? 'http://127.0.0.1:3000';
const browser = await chromium.connectOverCDP(process.env.CDP ?? 'http://localhost:29229');
const page = await browser.contexts()[0].newPage();
const errors = [];
page.on('pageerror', (e) => errors.push(e.message));

await page.goto(base + '/learn/read/optionals');
await page.getByRole('button', { name: 'Run' }).first().click();
await page.getByRole('status').first().filter({ hasText: /\S/ }).waitFor({ timeout: 30000 });
console.log('chapter example runs');

for (const id of ['halving-game', 'leaderboard']) {
  await page.goto(`${base}/learn/practice/${id}`);
  await page.evaluate((k) => localStorage.removeItem(k), `cp-solved:${id}`);
  await page.reload();
  const submit = page.getByRole('button', { name: 'Submit' });
  await submit.click();
  await page.getByText(/Wrong answer on test|Compile or runtime error/).waitFor({ timeout: 30000 });
  await page.locator('.cm-content').first().click();
  await page.keyboard.press('ControlOrMeta+a');
  await page.keyboard.insertText(readFileSync(new URL(`../content/practice/${id}/ref.cp`, import.meta.url), 'utf8'));
  await submit.click();
  await page.getByText('All tests pass').waitFor({ timeout: 30000 });
  console.log(`${id}: starter rejected, reference accepted`);
}
await page.close();
console.log('page errors:', errors);
process.exit(errors.length ? 1 : 0);
