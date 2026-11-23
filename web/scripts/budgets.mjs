// Size budgets the static build (web/out) must stay within, so CI's site
// build fails on growth instead of a reviewer noticing it later. cpl.wasm
// has its own check (wasm-size.mjs); `/` is measured in a browser (page-weight).
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { gzipSync } from 'node:zlib';

const out = join(dirname(dirname(fileURLToPath(import.meta.url))), 'out');
const budgets = [['search index', 'api/search', 600_000]];
let over = 0;
for (const [what, file, cap] of budgets) {
  const size = gzipSync(readFileSync(join(out, file)), { level: 9 }).length;
  const ok = size <= cap;
  if (!ok) over++;
  console[ok ? 'log' : 'error'](
    `budgets: ${what} ${size.toLocaleString('en-US')} of ${cap.toLocaleString('en-US')} B gz${ok ? '' : ' (over: index fewer words per page, or split the index)'}`,
  );
}
process.exit(over ? 1 : 0);
