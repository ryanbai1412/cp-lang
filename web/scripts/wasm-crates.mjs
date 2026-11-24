// Per-crate gzipped bytes of cpl.wasm and their budgets (site/R7). The total
// cap in wasm-size.mjs is the hard limit; a crate's budget says which lane's
// growth is eating it. Budgets live in wasm-crate-budgets.json, measured on
// CI (the hosted build is the truth); each crate's owner is the domain whose
// charter in handbook/domains/ describes it.
import { readdirSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';
import { gzipSync } from 'node:zlib';

export const DATA = 'data (std source, strings)';
export const RUNTIME = '(runtime)';

// Rust's v0 mangling (`_R`): each crate root is `C`, an optional
// disambiguator `s<base62>_`, and the crate's name, length-prefixed. A std
// generic (`core`, `alloc`, `hashbrown`) instantiated on a cpl type is
// charged to the first cpl crate in its path or type arguments, else to its
// own crate. A root ending the symbol is the crate the linker kept the copy
// from, which depends on link order, not on who uses it, so it is skipped.
// Anything else is libc or compiler-rt.
const ROOT = /C(?:s[0-9A-Za-z]*_)?(\d+)/g;
export function crateOf(name) {
  if (!name.startsWith('_R')) return RUNTIME;
  const roots = [...name.matchAll(ROOT)].map((m) => {
    const at = m.index + m[0].length;
    return [name.substr(at, +m[1]), at + +m[1]];
  });
  if (roots.length > 1 && roots.at(-1)[1] === name.length) roots.pop();
  return (roots.find(([c]) => c.startsWith('cpl_')) ?? roots[0])?.[0] ?? RUNTIME;
}

/** The code bodies, function names (index -> name) and data segment of a cpl.wasm built with names. */
function sections(wasm) {
  let at = 8;
  const leb = () => {
    let v = 0, shift = 0, b;
    do {
      b = wasm[at++];
      v += (b & 0x7f) * 2 ** shift;
      shift += 7;
    } while (b & 0x80);
    return v;
  };
  const str = () => {
    const len = leb();
    const s = wasm.toString('utf8', at, at + len);
    at += len;
    return s;
  };
  let imported = 0;
  const bodies = [];
  const names = new Map();
  let data = Buffer.alloc(0);
  while (at < wasm.length) {
    const id = wasm[at++];
    const size = leb();
    const end = at + size;
    if (id === 2) {
      for (let i = leb(); i > 0; i--) {
        str();
        str();
        const kind = wasm[at++];
        if (kind === 0) (leb(), imported++);
        else if (kind === 1) (at++, wasm[at++] & 1 ? (leb(), leb()) : leb());
        else if (kind === 2) (wasm[at++] & 1 ? (leb(), leb()) : leb());
        else if (kind === 3) at += 2;
      }
    } else if (id === 10) {
      for (let i = leb(); i > 0; i--) {
        const len = leb();
        bodies.push(wasm.subarray(at, at + len));
        at += len;
      }
    } else if (id === 11) {
      data = wasm.subarray(at, end);
    } else if (id === 0 && str() === 'name') {
      while (at < end) {
        const sub = wasm[at++];
        const subEnd = leb() + at;
        if (sub === 1) for (let i = leb(); i > 0; i--) names.set(leb(), str());
        at = subEnd;
      }
    }
    at = end;
  }
  return { bodies, imported, names, data };
}

/** Every function name in a cpl.wasm built with names. */
export const functionNames = (wasm) => [...sections(wasm).names.values()];

/** [crate, raw code bytes, gzipped bytes] for a cpl.wasm built with names, largest first. */
export function contributors(wasm) {
  const { bodies, imported, names, data } = sections(wasm);
  const by = new Map([[DATA, [data]]]);
  bodies.forEach((body, i) => {
    const c = crateOf(names.get(i + imported) ?? '');
    by.set(c, [...(by.get(c) ?? []), body]);
  });
  return [...by]
    .map(([c, parts]) => {
      const all = Buffer.concat(parts);
      return [c, all.length, gzipSync(all, { level: 9 }).length];
    })
    .sort((a, b) => b[1] - a[1]);
}

// Each element type and order a crate sorts by compiles its own copy of core's
// sort (ipnsort for sort_unstable*, driftsort_main for sort* and BTreeMap's
// collect), a few KB each. The count only shrinks: sort through a key type that
// already has a copy, or keep the list sorted as it is built.
const SORT_ENTRY = /4core5slice4sort(?:8unstable7ipnsort|6stable14driftsort_main)/;
export const sortCopiesFile = 'web/scripts/wasm-sort-copies.json';

/** The function names in `names` that are a copy of core's sort, one per element type and order. */
export const sortCopies = (names) => [...new Set(names.filter((name) => SORT_ENTRY.test(name)))].sort();

// The cpl crate a sort copy was compiled for: its last cpl root, which for a
// sort on a std type (`&str`) is the crate that calls it, not its type's crate.
const sortedIn = (name) => [...name.matchAll(ROOT)].map((m) => name.substr(m.index + m[0].length, +m[1])).filter((c) => c.startsWith('cpl_')).at(-1) ?? crateOf(name);

/** Why `copies` (from sortCopies) is over `max`, or the lower `max` to write, or neither. */
export function checkSortCopies(copies, max) {
  if (copies.length > max) {
    const list = copies.map((c) => `\n  ${sortedIn(c)}: ${c}`).join('');
    return {
      error: `cpl.wasm has ${copies.length} copies of core's sort, over the ${max} in ${sortCopiesFile}: sort through a key type`
        + ` that already has one, or keep the list sorted as it is built (never raise the count)${list}`,
      lower: null,
    };
  }
  return { error: null, lower: copies.length < max ? `${copies.length} copies of core's sort: lower "max" in ${sortCopiesFile} to ${copies.length}` : null };
}

/**
 * crate (`cpl-check`) -> domain, from the `Describes` row of each charter,
 * `charters` being domain -> the text of handbook/domains/<domain>.md. A
 * whole crate is described as `crates/cpl-x` or `crates/cpl-x/`; a file in
 * one (`crates/cpl-interp/src/trace.rs`) does not own the crate.
 */
export function owners(charters) {
  const out = new Map();
  for (const [domain, text] of Object.entries(charters)) {
    const row = text.match(/^\| Describes \| (.*) \|$/m);
    for (const c of row?.[1].matchAll(/`crates\/(cpl-[a-z-]+)\/?`/g) ?? []) out.set(c[1], domain);
  }
  return out;
}

/** domain -> charter text, for every handbook/domains/<domain>.md but the README. */
export function readCharters(dir) {
  return Object.fromEntries(
    readdirSync(dir)
      .filter((f) => f.endsWith('.md') && f !== 'README.md')
      .map((f) => [f.slice(0, -3), readFileSync(join(dir, f), 'utf8')]),
  );
}

/** The domain whose lane answers for `crate`'s bytes, or why none does. */
export function ownerOf(crate, map) {
  if (crate === DATA) return 'stdlib';
  if (!crate.startsWith('cpl_')) return 'infra';
  return map.get(crate.replaceAll('_', '-')) ?? null;
}

/** Budget failures, and budgets with room to lower, for `rows` from contributors. */
export function checkBudgets(rows, budgets, map) {
  const errors = [];
  const lower = [];
  for (const [crate, , gz] of rows) {
    const owner = ownerOf(crate, map);
    const who = owner ? `the ${owner} lane (handbook/domains/${owner}.md)` : `no domain: add crates/${crate.replaceAll('_', '-')} to the Describes row of its charter in handbook/domains/`;
    const budget = budgets[crate];
    if (budget === undefined) errors.push(`${crate} (${who}) has no budget in web/scripts/wasm-crate-budgets.json; it is ${gz.toLocaleString('en-US')} B gzipped`);
    else if (gz > budget) errors.push(`${crate} is ${gz.toLocaleString('en-US')} B gzipped, over its budget of ${budget.toLocaleString('en-US')}: ${who} grew it`);
    else if (budget - budgetFor(gz) > Math.max(1000, budget * 0.03)) lower.push(`${crate}: lower its budget to ${budgetFor(gz).toLocaleString('en-US')}`);
  }
  const seen = new Set(rows.map(([c]) => c));
  for (const crate of Object.keys(budgets)) if (!seen.has(crate)) lower.push(`${crate} is no longer in cpl.wasm: delete its budget`);
  return { errors, lower };
}

/**
 * The commit gate's crate budgets: `errors` for the crates in `touched` (and the
 * data segment) over budget, `warnings` for the others, whose growth isn't this commit's.
 */
export function touchedOverBudget(rows, budgets, map, touched) {
  const mine = (crate) => crate === DATA || touched.includes(crate);
  const errors = checkBudgets(rows.filter(([c]) => mine(c)), budgets, map).errors;
  const warnings = checkBudgets(rows.filter(([c]) => !mine(c)), budgets, map).errors;
  const data = dataOverBudget(rows, budgets);
  return { errors: data ? [data, ...errors.filter((e) => !e.startsWith(DATA))] : errors, warnings };
}

/**
 * The land gate's shrink-only ratchet against the rebased parent build (`parent`:
 * crate -> gz): a crate within its budget in the parent stays within it, and one
 * over it may not grow by a byte; growth of GREW_WARN B or more within budget warns.
 * With `touched`, only those crates and the data segment are held; another crate
 * that moved only warns, since deleting functions anywhere renumbers the calls and
 * shifts every crate's gzip by a few bytes.
 */
export function ratchet(rows, budgets, map, parent, touched = null) {
  const errors = [];
  const warnings = [];
  const held = [];
  const mine = (crate) => touched === null || crate === DATA || touched.includes(crate);
  for (const row of rows) {
    const [crate, , gz] = row;
    const over = crate === DATA ? dataOverBudget(rows, budgets) : checkBudgets([row], budgets, map).errors[0];
    if (!over) continue;
    const was = parent[crate];
    const wasOver = was !== undefined && budgets[crate] !== undefined && was > budgets[crate];
    const b = (n) => n.toLocaleString('en-US');
    const out = mine(crate) ? errors : warnings;
    if (wasOver && gz <= was) held.push(`${over}, but no more than its parent's ${b(was)} B`);
    else if (wasOver) out.push(`${over}; it grew ${b(gz - was)} B against its parent's ${b(was)} B`);
    else out.push(over);
  }
  const grew = rows
    .filter(([c, , gz]) => parent[c] !== undefined && gz - parent[c] >= GREW_WARN && gz <= (budgets[c] ?? -1))
    .map(([c, , gz]) => `${c} grew ${(gz - parent[c]).toLocaleString('en-US')} B gzipped against its parent's ${parent[c].toLocaleString('en-US')} B`);
  return { errors, warnings, held, grew };
}

/** A lowered budget for a crate measured at `gz`: 1% or 256 B of room, to the next 100. */
export const lowerTo = (gz) => Math.ceil((gz + Math.max(gz * 0.01, 256)) / 100) * 100;

/**
 * The budgets after a build within them: each budget with more room than lowerTo
 * gives over its crate's measurement comes down to lowerTo, so slack a cut freed
 * can't be regrown. Budgets only go down. Null when nothing lowers.
 */
export function lowered(rows, budgets) {
  const next = { ...budgets };
  for (const [crate, , gz] of rows) if (budgets[crate] !== undefined && lowerTo(gz) < budgets[crate]) next[crate] = lowerTo(gz);
  return Object.keys(next).some((c) => next[c] !== budgets[c]) ? next : null;
}

/** A touched crate growing at least this many B gz against its parent warns at land. */
export const GREW_WARN = 200;

/** A new budget for a crate measured at `gz`: 2% and 512 B of room, to the next 100. */
export const budgetFor = (gz) => Math.ceil((gz * 1.02 + 512) / 100) * 100;

/** Why the data segment of a names build's `rows` is over its budget, or null. */
export function dataOverBudget(rows, budgets) {
  const gz = rows.find(([crate]) => crate === DATA)?.[2];
  const budget = budgets[DATA];
  if (gz === undefined || budget === undefined || gz <= budget) return null;
  return `${DATA} is ${gz.toLocaleString('en-US')} B gzipped, over its budget of ${budget.toLocaleString('en-US')}:`
    + ' cut what the browser build embeds (crates/cpl-check/src/std_bundle.rs) or std itself, never the budget';
}

// `node web/scripts/wasm-crates.mjs --data DIR/cpl.wasm [--touched cpl_a,cpl_b]
// [--parent SIZES.json]` (a scripts/build-wasm.sh --names build): the commit gate's
// check that the data segment and every touched crate are within their budgets; other
// crates over theirs only warn. With `--parent`, a parent build's sizes, the touched
// crates (every crate without `--touched`) are held to the shrink-only ratchet, and
// the sort copies only grow if they grew. `--sizes DIR/cpl.wasm SIZES.json` writes a build's sizes for `--parent`, and
// `--data` takes such a file in place of a build. `--lower-budgets` lowers each budget
// with slack after a pass (lowered).
if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href && process.argv[2] === '--sizes') {
  const wasm = readFileSync(process.argv[3]);
  const crates = Object.fromEntries(contributors(wasm).map(([c, , gz]) => [c, gz]));
  writeFileSync(process.argv[4], `${JSON.stringify({ crates, sorts: sortCopies(functionNames(wasm)).length })}\n`);
} else if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  const at = process.argv.indexOf('--data');
  if (at < 0 || !process.argv[at + 1]) {
    console.error('usage: node web/scripts/wasm-crates.mjs --data DIR/cpl.wasm [--touched cpl_a,cpl_b] [--parent SIZES.json] | --sizes DIR/cpl.wasm SIZES.json');
    process.exit(2);
  }
  const t = process.argv.indexOf('--touched');
  const touched = t < 0 ? [] : (process.argv[t + 1] ?? '').split(',').filter(Boolean);
  const wasm = readFileSync(process.argv[at + 1]);
  // A --sizes file in place of the build (scripts/test_land.py's builds) has no names to read sorts from.
  const sized = wasm[0] === 0x7b && JSON.parse(wasm.toString('utf8'));
  const rows = sized ? Object.entries(sized.crates).map(([c, gz]) => [c, 0, gz]) : contributors(wasm);
  const here = dirname(fileURLToPath(import.meta.url));
  const budgets = JSON.parse(readFileSync(join(here, 'wasm-crate-budgets.json'), 'utf8'));
  const map = owners(readCharters(join(here, '../../handbook/domains')));
  const flag = (name) => (process.argv.indexOf(name) < 0 ? null : process.argv[process.argv.indexOf(name) + 1]);
  const parent = flag('--parent') && JSON.parse(readFileSync(flag('--parent'), 'utf8'));
  const { errors, warnings, held = [], grew = [] } = parent
    ? ratchet(rows, budgets, map, parent.crates, t < 0 ? null : touched)
    : touchedOverBudget(rows, budgets, map, touched);
  let next;
  const copies = sized ? Array(sized.sorts ?? 0).fill('') : sortCopies(functionNames(wasm));
  const sorts = checkSortCopies(copies, JSON.parse(readFileSync(join(here, '../..', sortCopiesFile), 'utf8')).max);
  if (flag('--lower-budgets') !== null && !errors.length && (next = lowered(rows, budgets))) {
    writeFileSync(join(here, 'wasm-crate-budgets.json'), `${JSON.stringify(next, null, 2)}\n`);
    console.error('wasm-crates: lowered each budget a crate is under to its measurement (wasm-crate-budgets.json)');
  }
  if (sorts.error && parent && copies.length <= parent.sorts) held.push(`${sorts.error.split(':')[0]}, but no more than its parent's ${parent.sorts}`);
  else if (sorts.error) errors.push(sorts.error);
  if (sorts.lower) console.error(`wasm-crates: ${sorts.lower}`);
  for (const w of warnings) console.error(`wasm-crates: warning: ${w} (not touched here)`);
  for (const h of held) console.error(`wasm-crates: warning: ${h}`);
  for (const g of grew) console.error(`wasm-crates: warning: ${g}; cut it at its source`);
  for (const e of errors) console.error(`wasm-crates: ${e}`);
  if (errors.length) process.exit(1);
  const gz = rows.find(([crate]) => crate === DATA)?.[2] ?? 0;
  console.log(`wasm-crates: ${DATA} is ${gz.toLocaleString('en-US')} B gzipped, within its budget of ${budgets[DATA].toLocaleString('en-US')}`
    + (touched.length ? `; ${touched.join(', ')} within theirs${held.length ? ' or no larger than in the parent' : ''}` : ''));
}
