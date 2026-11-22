// Page weight of the pages a visitor lands on: every response a cold load
// fetches, gzipped at level 9 (the static server here does not compress), and
// the largest contentful paint. Run with `pnpm --dir web page-weight`.
import { gzipSync } from 'node:zlib';
import { base, open } from './browser.mjs';

const { page, errors } = await open();
const cdp = await page.context().newCDPSession(page);
await cdp.send('Network.enable');
await cdp.send('Network.setCacheDisabled', { cacheDisabled: true });
for (const path of ['/', '/play', '/docs/guides/tour']) {
  const bodies = [];
  const collect = (r) =>
    bodies.push(r.body().then((b) => ({ wasm: r.url().endsWith('.wasm'), size: gzipSync(b, { level: 9 }).length }), () => ({ size: 0 })));
  page.on('response', collect);
  await page.goto(base + path, { waitUntil: 'networkidle' });
  const lcp = await page.evaluate(
    () =>
      new Promise((done) =>
        new PerformanceObserver((list) => done(Math.round(list.getEntries().at(-1).startTime))).observe({
          type: 'largest-contentful-paint',
          buffered: true,
        }),
      ),
  );
  page.off('response', collect);
  const sizes = await Promise.all(bodies);
  const sum = (xs) => xs.reduce((a, x) => a + x.size, 0).toLocaleString('en-US');
  console.log(
    `page-weight: ${path} ${sum(sizes)} bytes gzipped in ${sizes.length} requests (${sum(sizes.filter((x) => x.wasm))} of it cpl.wasm), LCP ${lcp} ms`,
  );
}
await page.close();
process.exit(errors.length ? 1 : 0);
