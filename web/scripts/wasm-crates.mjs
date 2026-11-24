// Per-crate gzipped bytes of cpl.wasm and their budgets (site/R7). The total
// cap in wasm-size.mjs is the hard limit; a crate's budget says which lane's
// growth is eating it. Budgets live in wasm-crate-budgets.json, measured on
// CI (the hosted build is the truth); each crate's owner is the domain
// handbook/domains/README.md says owns it.
import { readdirSync, readFileSync } from 'node:fs';
import { join } from 'node:path';
import { gzipSync } from 'node:zlib';

export const DATA = 'data (std source, strings)';
export const RUNTIME = '(runtime)';

// Rust's v0 mangling (`_R`): the first crate root, `C`, a disambiguator
// `s<base62>_`, and the crate's name, length-prefixed; for an impl, the
// crate it is in. Anything else is libc or compiler-rt.
export function crateOf(name) {
  const m = /^_R.*?C(?:s[0-9A-Za-z]*_)?(\d+)/.exec(name);
  return m ? name.substr(m.index + m[0].length, +m[1]) : RUNTIME;
}

/** [crate, raw code bytes, gzipped bytes] for a cpl.wasm built with names, largest first. */
export function contributors(wasm) {
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

/** A new budget for a crate measured at `gz`: 2% and 512 B of room, to the next 100. */
export const budgetFor = (gz) => Math.ceil((gz * 1.02 + 512) / 100) * 100;
