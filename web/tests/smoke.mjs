import { chromium } from '@playwright/test';

const base = process.env.BASE ?? 'http://127.0.0.1:3000';
const browser = await chromium.connectOverCDP(process.env.CDP ?? 'http://localhost:29229');
const page = await browser.contexts()[0].newPage();
const errors = [];
page.on('pageerror', (e) => errors.push(e.message));
await page.goto(base + '/');
await page.getByRole('button', { name: 'Run' }).first().click();
const out = page.getByRole('status').first();
await out.filter({ hasText: /^115\s*$/ }).waitFor({ timeout: 30000 });
console.log('home run output:', JSON.stringify(await out.innerText()));
const [viewer] = await Promise.all([
  page.context().waitForEvent('page'),
  page.getByRole('button', { name: 'Trace' }).first().click(),
]);
await viewer.getByText('main.cp').first().waitFor({ state: 'attached', timeout: 30000 });
console.log('trace viewer ok');
await viewer.close();
await page.getByRole('button', { name: 'Stress' }).first().click();
await page.getByText('Shrunk to the smallest input that still fails').waitFor({ timeout: 60000 });
console.log('stress finds and shrinks the planted bug');
await page.goto(base + '/play');
await page.getByRole('button', { name: 'Run' }).click();
await page.getByRole('status').filter({ hasText: '14' }).waitFor({ timeout: 30000 });
console.log('play run output ok');
await page.close();
console.log('page errors:', errors);
process.exit(errors.length ? 1 : 0);
