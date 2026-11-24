// Enforces site/R7 (handbook/guidelines/site-checklist.md).
// Fails when the browser compiler is over its download budget. The cap is on
// the gzipped size of cpl.wasm, which is what a visitor waits for before any
// editor on the site has diagnostics; cpl-build.wasm (adds the C++ back end,
// explain and stress) loads on first use and is reported, not capped.
//
//   node scripts/wasm-size.mjs [dist/wasm/cpl.wasm]
//   node scripts/wasm-size.mjs --why   the crates cpl.wasm's bytes come from
//   node scripts/wasm-size.mjs --update-budgets   rewrite each crate's budget
//
// With WASM_NAMES=<cpl.wasm built by scripts/build-wasm.sh --names>, it also
// holds each crate to its gzipped budget in wasm-crate-budgets.json and names
// the lane that owns a crate over it (wasm-crates.mjs). CI's site job always
// passes it: the hosted measurement is the one that counts.
//
// With WASM_SIZE_HISTORY=<file> (CI keeps it across main runs), it prints the
// gzipped change since the commit recorded there, then records this one, so
// growth names its commits before the cap fails.
// Over the cap, it prints --why's table too, so a lane sees what grew.
import { execFileSync } from 'node:child_process';
import { existsSync, mkdirSync, mkdtempSync, readFileSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { gzipSync } from 'node:zlib';
import { budgetFor, checkBudgets, contributors, owners } from './wasm-crates.mjs';
import { sizeDelta } from './wasm-delta.mjs';

// A shrink-only ratchet: after each cut, lower the cap to the new size plus
// about 2% in the same commit. It went up once, from 1,720,000, when the VM
// came into cpl.wasm (+153 KB gzipped, the browser runs ~5x faster); the
// commit that deletes the tree-walker lowers it to the new size plus slack.
// Building the interpreter at "s" cut it to 1,804,697; the in-browser TLE
// shrink (55e93f978) brought it to 1,841,845; 66 feature commits over
// 0d43fd76e..654c50c92 brought it to 1,862,540 in CI (decision 440);
// building the interpreter at "z" like the rest took 29 KB off (decision 446).
const cap = 1_875_000;
const repo = dirname(dirname(dirname(fileURLToPath(import.meta.url))));
const args = process.argv.slice(2);
const why = args.includes('--why');
const update = args.includes('--update-budgets');
const budgetsFile = join(repo, 'web/scripts/wasm-crate-budgets.json');
const file = args.find((a) => !a.startsWith('--')) ?? join(repo, 'dist/wasm/cpl.wasm');
const n = (x) => x.toLocaleString('en-US');

function explain() {
  const out = mkdtempSync(join(tmpdir(), 'wasm-why-'));
  console.error('wasm-size: building cpl.wasm with function names to see what is in it');
  try {
    execFileSync(join(repo, 'scripts/build-wasm.sh'), ['--names', out], { stdio: 'pipe' });
  } catch (error) {
    console.error(String(error.stderr ?? error));
    return;
  }
  const rows = contributors(readFileSync(join(out, 'cpl.wasm')));
  const total = rows.reduce((s, [, b]) => s + b, 0);
  console.error('wasm-size: bytes by crate (raw, and gzipped on its own):');
  for (const [c, b, g] of rows.slice(0, 15)) console.error(`  ${n(b).padStart(10)}  ${((100 * b) / total).toFixed(1).padStart(5)}%  ${n(g).padStart(9)} gz  ${c}`);
  console.error('  Leave what the editors do not need out of cpl.wasm (crates/cpl-wasm feature `build`, or cfg on wasm32).');
}

if (!existsSync(file)) {
  console.error(`wasm-size: ${file} is missing; run scripts/build-wasm.sh`);
  process.exit(1);
}
const raw = readFileSync(file);
const gz = gzipSync(raw, { level: 9 }).length;
const line = `wasm-size: cpl.wasm is ${n(raw.length)} bytes, ${n(gz)} gzipped (cap ${n(cap)})`;
const history = process.env.WASM_SIZE_HISTORY;
if (history) {
  const head = (() => {
    try {
      return execFileSync('git', ['rev-parse', 'HEAD'], { cwd: repo, encoding: 'utf8' }).trim();
    } catch {
      return 'unknown';
    }
  })();
  const [was, wasGz] = existsSync(history) ? readFileSync(history, 'utf8').trim().split(/\s+/) : [];
  if (was && Number.isFinite(+wasGz) && was !== head) {
    const { delta, grew } = sizeDelta(was, +wasGz, head, gz);
    console.log(delta);
    // A warning, not a failure: the cap is the only gate (decision 446).
    if (grew && process.env.GITHUB_ACTIONS) console.log(`::warning title=cpl.wasm grew::${delta}`);
    if (process.env.GITHUB_STEP_SUMMARY) writeFileSync(process.env.GITHUB_STEP_SUMMARY, `${delta}\n`, { flag: 'a' });
  }
  mkdirSync(dirname(history), { recursive: true });
  writeFileSync(history, `${head} ${gz}\n`);
}
const full = join(dirname(file), 'cpl-build.wasm');
const fullLine = existsSync(full)
  ? `wasm-size: cpl-build.wasm (loaded on the first build, explain or stress) is ${n(gzipSync(readFileSync(full), { level: 9 }).length)} gzipped`
  : null;
const names = process.env.WASM_NAMES;
let over = [];
if (names && existsSync(names)) {
  const rows = contributors(readFileSync(names));
  if (update) {
    writeFileSync(budgetsFile, `${JSON.stringify(Object.fromEntries(rows.map(([c, , g]) => [c, budgetFor(g)]).sort(([a], [b]) => (a < b ? -1 : 1))), null, 2)}\n`);
    console.log(`wasm-size: wrote ${rows.length} crate budgets to web/scripts/wasm-crate-budgets.json`);
  }
  const budgets = JSON.parse(readFileSync(budgetsFile, 'utf8'));
  const { errors, lower } = checkBudgets(rows, budgets, owners(readFileSync(join(repo, 'handbook/domains/README.md'), 'utf8')));
  over = errors;
  for (const e of errors) {
    console.error(`wasm-size: ${e}`);
    if (process.env.GITHUB_ACTIONS) console.log(`::error title=cpl.wasm crate over budget::${e}`);
  }
  for (const l of lower) console.log(`wasm-size: ${l} (node scripts/wasm-size.mjs --update-budgets)`);
  if (!errors.length) console.log(`wasm-size: ${rows.length} crates within their gzipped budgets`);
} else if (process.env.GITHUB_ACTIONS) {
  console.error('wasm-size: CI must set WASM_NAMES to a names build (scripts/build-wasm.sh --names DIR) for the per-crate budgets');
  process.exit(1);
} else {
  console.log('wasm-size: per-crate budgets skipped; set WASM_NAMES=DIR/cpl.wasm after scripts/build-wasm.sh --names DIR');
}
if (gz > cap) {
  console.error(`${line}: over the cap`);
  explain();
  process.exit(1);
}
if (over.length) process.exit(1);
console.log(line);
if (fullLine) console.log(fullLine);
if (gz * 1.05 < cap) console.log(`wasm-size: over 5% headroom; lower the cap to about ${Math.ceil((gz * 1.02) / 1e4) * 1e4}`);
if (why) explain();
