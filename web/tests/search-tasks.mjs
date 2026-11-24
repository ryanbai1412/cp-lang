// handbook/guidelines/site-checklist.md site/R18: a contestant who describes
// the task, not the item's name, finds the item in the first results.
import { existsSync } from 'node:fs';
import { base, done, open } from './browser.mjs';

// Task phrasing -> the pages that answer it (any one counts): the std module
// or the topic page in /docs/ref/lib.
const tasks = [
  ['binary search on the answer', ['/docs/ref/std/search', '/docs/ref/lib/search-dp']],
  ['prefix sums', ['/docs/ref/std/prefix', '/docs/ref/lib/ops-ranges', '/docs/ref/lib/sequence-patterns']],
  ['range sum with point updates', ['/docs/ref/std/fenwick', '/docs/ref/std/segtree', '/docs/ref/lib/ops-ranges']],
  ['range update and range query', ['/docs/ref/std/lazy', '/docs/ref/std/segtree', '/docs/ref/lib/ops-ranges']],
  ['union find', ['/docs/ref/std/dsu', '/docs/ref/lib/graphs-trees', '/docs/ref/lib/containers']],
  ['shortest path', ['/docs/ref/std/graph', '/docs/ref/lib/graphs-trees']],
  ['minimum spanning tree', ['/docs/ref/std/graph', '/docs/ref/std/dsu', '/docs/ref/lib/graphs-trees']],
  ['maximum flow', ['/docs/ref/std/flow', '/docs/ref/lib/flows']],
  ['lowest common ancestor', ['/docs/ref/std/tree', '/docs/ref/lib/graphs-trees']],
  ['modular inverse', ['/docs/ref/std/mod', '/docs/ref/lib/math']],
  ['polynomial multiplication', ['/docs/ref/std/ntt', '/docs/ref/std/poly', '/docs/ref/lib/math']],
  ['priority queue', ['/docs/ref/std/heap', '/docs/ref/lib/containers']],
  ['sliding window maximum', ['/docs/ref/std/window', '/docs/ref/std/deque', '/docs/ref/lib/sequence-patterns', '/docs/ref/lib/ops-ranges']],
  ['convex hull', ['/docs/ref/std/geometry', '/docs/ref/lib/geometry']],
  ['string hashing', ['/docs/ref/std/strings', '/docs/ref/lib/strings']],
  ['range minimum query', ['/docs/ref/std/sparse', '/docs/ref/std/segtree', '/docs/ref/lib/ops-ranges']],
  ['dp over subsets', ['/docs/ref/std/subset', '/docs/ref/std/dp', '/docs/ref/lib/search-dp']],
  ['ordered multiset', ['/docs/ref/std/bag', '/docs/ref/lib/containers']],
  ['hash map', ['/docs/ref/std/hashmap', '/docs/ref/lib/containers']],
  ['matrix exponentiation', ['/docs/ref/std/matrix', '/docs/ref/lib/math']],
  ['random numbers', ['/docs/ref/std/rand', '/docs/ref/lib/random-stress']],
  ['read the input', ['/docs/ref/std/io', '/docs/guides/io-and-formatting']],
  ['offline range queries', ['/docs/ref/std/mo', '/docs/ref/lib/ops-ranges']],
  ['merge overlapping intervals', ['/docs/ref/std/interval', '/docs/ref/lib/containers', '/docs/ref/lib/ops-ranges']],
  ['gcd', ['/docs/ref/std/math', '/docs/ref/lib/math']],
  ['bfs on a grid', ['/docs/ref/std/grid', '/docs/ref/lib/graphs-trees']],
  ['knapsack', ['/docs/ref/std/dp', '/docs/ref/lib/search-dp']],
  ['check my solution against a brute force', ['/docs/guides/stress', '/docs/ref/lib/random-stress']],
  ['convert from c++', ['/docs/guides/cpp-guide']],
];
const TOP = 5;
// An answer page that no longer exists fails here, so this list can't go stale.
const out = new URL('../out', import.meta.url).pathname;
const gone = [...new Set(tasks.flatMap(([, want]) => want))].filter((u) => !existsSync(`${out}${u}.html`) && !existsSync(`${out}${u}/index.html`));
if (gone.length) {
  console.error(`search-tasks: answer pages not in the site: ${gone.join(' ')}`);
  done(1);
}

const { page } = await open();
await page.goto(base + '/docs/guides/tour');
await page.getByRole('button', { name: /^Search/ }).first().click();
const box = page.getByRole('combobox', { name: 'Search' });
const misses = [];
for (const [query, want] of tasks) {
  await box.fill(query);
  await page.waitForTimeout(400);
  await page.locator('[role=option]').first().waitFor({ timeout: 10000 }).catch(() => {});
  const ids = await page.locator('[role=option]').evaluateAll((os) => os.map((o) => o.id.replace('fd-search-option-', '')));
  const pages = [...new Set(ids.map((id) => id.replace(/-\d+$/, '')))].slice(0, TOP);
  const rank = pages.findIndex((p) => want.some((w) => p === w || p.startsWith(w + '/')));
  if (rank < 0) misses.push(query);
  console.log(`${rank < 0 ? 'MISS' : `#${rank + 1}  `} ${query}${rank < 0 ? `  (got ${pages.join(' ') || 'nothing'})` : ''}`);
}
console.log(`search-tasks: ${tasks.length - misses.length} of ${tasks.length} task phrasings find their page in the top ${TOP}`);
if (misses.length) {
  console.error(`search-tasks: missed: ${misses.join('; ')}`);
  done(1);
}
done(0);
