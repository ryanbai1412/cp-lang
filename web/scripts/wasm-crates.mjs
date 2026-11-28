// Per-crate gzipped bytes of cpl.wasm and their budgets (site/R7). The total
// cap in wasm-size.mjs is the hard limit; a crate's budget says which lane's
// growth is eating it. Budgets live in wasm-crate-budgets.json, measured on
// CI (the hosted build is the truth); each crate's owner is the domain whose
// charter in handbook/domains/ describes it.
import { readdirSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';
import { createHash } from 'node:crypto';
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

// The crate whose change can change `name`'s code, for the land gate's per-crate
// normalized functions: a generic instantiated on a closure (`NC`) belongs to the
// closure's crate, the only one that can name it, else crateOf. Budgets still use
// crateOf, so this file has two attributions: debt until plan/QUESTIONS.md's
// "site: charge a generic instance to the crate that instantiates it?" is answered.
export function gateCrateOf(name) {
  return closureCrate(name) ?? crateOf(name);
}

// The last closure's crate: a closure passed into another crate's generic closure comes
// after it. A closure's path ends at a root or a backref (`B<base62>_`, an offset after `_R`).
const BASE62 = '0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ';
function closureCrate(name) {
  if (!name.startsWith('_R')) return null;
  // `<base62>_`: empty is 0, else the digits' value + 1. Returns [value, end].
  const base62 = (at) => {
    let v = 0;
    const from = at;
    for (; name[at] !== '_'; at++) {
      const d = BASE62.indexOf(name[at]);
      if (d < 0) return [null, at];
      v = v * 62 + d;
    }
    return [at === from ? 0 : v + 1, at + 1];
  };
  let found = null;
  for (const m of name.matchAll(/NC/g)) {
    let at = m.index + 2;
    for (let hops = 0; hops < 64; hops++) {
      const c = name[at];
      if (c === 'N') at += 2;
      else if (c === 'I') at++;
      else if (c === 'M' || c === 'X') {
        at++;
        if (name[at] === 's') at = base62(at + 1)[1];
      } else if (c === 'B') {
        const [v] = base62(at + 1);
        if (v === null || v + 2 >= at) break;
        at = v + 2;
      } else if (c === 'C') {
        const r = /^C(?:s[0-9A-Za-z]*_)?(\d+)/.exec(name.slice(at));
        const crate = r && name.substr(at + r[0].length, +r[1]);
        if (crate?.startsWith('cpl_')) found = crate;
        break;
      } else break;
    }
  }
  return found;
}

/** `name` without the root of the crate the linker kept the copy from (see crateOf). */
export function sharedName(name) {
  const roots = [...name.matchAll(ROOT)];
  const last = roots.at(-1);
  return roots.length > 1 && last.index + last[0].length + +last[1] === name.length ? name.slice(0, last.index) : name;
}

/**
 * The code bodies, function names (index -> name), data segment, type signatures,
 * data segment ranges and memory size of a cpl.wasm built with names.
 */
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
  const imports = [];
  const types = [];
  const segments = [];
  const table = [];
  let memory = 0;
  let data = Buffer.alloc(0);
  const limits = () => (wasm[at++] & 1 ? [leb(), leb()][0] : leb());
  while (at < wasm.length) {
    const id = wasm[at++];
    const size = leb();
    const end = at + size;
    if (id === 1) {
      for (let i = leb(); i > 0; i--) {
        const from = at;
        if (wasm[at++] !== 0x60) throw new Error(`wasm-crates: type ${types.length} is not a function type`);
        for (let v = 0; v < 2; v++) {
          const n = leb();
          at += n;
        }
        types.push(wasm.subarray(from, at));
      }
    } else if (id === 2) {
      for (let i = leb(); i > 0; i--) {
        const field = `${str()}.${str()}`;
        const kind = wasm[at++];
        if (kind === 0) (leb(), imports.push(field), imported++);
        else if (kind === 1) (at++, wasm[at++] & 1 ? (leb(), leb()) : leb());
        else if (kind === 2) (wasm[at++] & 1 ? (leb(), leb()) : leb());
        else if (kind === 3) at += 2;
      }
    } else if (id === 5 && leb()) {
      memory = limits() * 65536;
    } else if (id === 9) {
      for (let i = leb(); i > 0; i--) {
        const flags = leb();
        if (flags !== 0 && flags !== 2) throw new Error(`wasm-crates: element segment kind ${flags}`);
        if (flags === 2) leb();
        if (wasm[at++] !== 0x41) throw new Error('wasm-crates: an element segment not at an i32.const');
        const base = sleb(wasm, at);
        while (wasm[at++] !== 0x0b);
        if (flags === 2 && wasm[at++] !== 0) throw new Error('wasm-crates: an element segment not of functions');
        for (let n = leb(), j = 0; j < n; j++) table[base + j] = leb();
      }
    } else if (id === 10) {
      for (let i = leb(); i > 0; i--) {
        const len = leb();
        bodies.push(wasm.subarray(at, at + len));
        at += len;
      }
    } else if (id === 11) {
      data = wasm.subarray(at, end);
      for (let i = leb(); i > 0; i--) {
        const flags = leb();
        let base = null;
        if (flags !== 1) {
          if (flags === 2) leb();
          if (wasm[at] === 0x41) base = sleb(wasm, at + 1);
          while (wasm[at++] !== 0x0b);
        }
        const len = leb();
        if (base !== null && len) segments.push([base, base + len]);
        at += len;
      }
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
  imports.forEach((field, i) => names.has(i) || names.set(i, `import ${field}`));
  return { bodies, imported, names, data, types, segments, table, memory };
}

/** Every function name in a cpl.wasm built with names. */
export const functionNames = (wasm) => [...sections(wasm).names.values()];

/**
 * crate -> function name -> its body as normalized writes it, for a cpl.wasm built
 * with names: what a crate's code is, whatever index or address the link gave it.
 * Another crate's change renumbers functions and types and moves statics; it does
 * not change a byte here. A function goes by its sharedName; one repeated in a crate
 * gets `#2`, `#3`, ...
 */
export function normalizedFunctions(wasm) {
  const { bodies, imported, names, types, segments, table, memory } = sections(wasm);
  const id = (text) => createHash('sha256').update(text).digest().subarray(0, 4);
  const fn = Array.from({ length: imported + bodies.length }, (_, i) => id(sharedName(names.get(i) ?? `#${i}`)));
  const ty = types.map(id);
  const lo = Math.min(...segments.map(([from]) => from));
  const slots = table.map((f) => Buffer.concat([Buffer.from([0xd2]), fn[f]]));
  const shape = { fn, ty, slots, statics: [lo, Math.max(memory, ...segments.map(([, to]) => to))] };
  const out = new Map();
  bodies.forEach((body, i) => {
    const name = names.get(i + imported) ?? `#${i + imported}`;
    const c = gateCrateOf(name);
    if (!out.has(c)) out.set(c, new Map());
    const fns = out.get(c);
    let key = sharedName(name);
    for (let n = 2; fns.has(key); n++) key = `${sharedName(name)}#${n}`;
    try {
      fns.set(key, normalized(body, shape));
    } catch (e) {
      throw new Error(`wasm-crates: ${name}: ${e.message}`);
    }
  });
  return out;
}

const skipLeb = (buf, at) => {
  while (buf[at++] & 0x80);
  return at;
};
const uleb = (buf, at) => {
  let v = 0, shift = 0, b;
  do {
    b = buf[at++];
    v += (b & 0x7f) * 2 ** shift;
    shift += 7;
  } while (b & 0x80);
  return v;
};
function sleb(buf, at) {
  let v = 0n, shift = 0n, b;
  do {
    b = buf[at++];
    v |= BigInt(b & 0x7f) << shift;
    shift += 7n;
  } while (b & 0x80);
  return Number(b & 0x40 ? v - (1n << shift) : v);
}

// Opcode -> immediates, for the instructions rustc's wasm32 output uses: f a function,
// t a type, a an address-or-integer (i32.const), m a memarg (align, then an offset that
// may be an address), u any other LEB, b a byte, 4/8 raw bytes, v br_table's labels,
// k a blocktype, y select's value types. Anything else is refused, never skipped.
const OPS = new Map([
  ...[0x02, 0x03, 0x04].map((o) => [o, 'k']),
  [0x0c, 'u'], [0x0d, 'u'], [0x0e, 'v'], [0x10, 'f'], [0x11, 'tu'], [0x12, 'f'], [0x13, 'tu'], [0x1c, 'y'],
  ...[0x20, 0x21, 0x22, 0x23, 0x24, 0x25, 0x26].map((o) => [o, 'u']),
  ...Array.from({ length: 0x3e - 0x28 + 1 }, (_, i) => [0x28 + i, 'm']),
  [0x3f, 'b'], [0x40, 'b'], [0x41, 'a'], [0x42, 'u'], [0x43, '4'], [0x44, '8'], [0xd0, 'b'], [0xd2, 'f'],
  ...[0x00, 0x01, 0x05, 0x0b, 0x0f, 0x1a, 0x1b, 0xd1].map((o) => [o, '']),
  ...Array.from({ length: 0xc4 - 0x45 + 1 }, (_, i) => [0x45 + i, '']),
]);
// 0xfc's sub-opcodes: the saturating truncations, then bulk memory and tables.
const FC = ['', '', '', '', '', '', '', '', 'ub', 'u', 'bb', 'b', 'uu', 'u', 'uu', 'u', 'u', 'u'];
const VALTYPE = new Set([0x40, 0x6f, 0x70, 0x7b, 0x7c, 0x7d, 0x7e, 0x7f]);

/**
 * `body` with each function it names written as 4 bytes of its name's hash, each type
 * as 4 of its signature's, and each relocated constant (wasm-ld leaves those as padded
 * 5-byte LEBs) as what it points at: a table slot as 0xd2 and its function's hash, a
 * static address as 4 bytes of its first-use order in the body, so the rest of the
 * link moving them changes nothing. Throws on an instruction it cannot read and on a
 * body whose outermost `end` is not its last byte, so a misread immediate can't pass.
 */
export function normalized(body, { fn, ty, slots, statics: [lo, hi] }) {
  const out = [];
  const addrs = new Map();
  let at = 0;
  const copy = (to) => {
    if (to > body.length) throw new Error(`instruction runs past the body's ${body.length} B`);
    for (; at < to; at++) out.push(body[at]);
  };
  const named = (table, v, what) => {
    if (!table[v]) throw new Error(`${what} ${v} does not exist`);
    out.push(...table[v]);
  };
  const addr = (v) => {
    if (!addrs.has(v)) addrs.set(v, addrs.size);
    const n = addrs.get(v);
    out.push(n & 0xff, (n >> 8) & 0xff, (n >> 16) & 0xff, n >>> 24);
  };
  const imm = (k) => {
    if (k === 'f') (named(fn, uleb(body, at), 'function'), (at = skipLeb(body, at)));
    else if (k === 't') (named(ty, uleb(body, at), 'type'), (at = skipLeb(body, at)));
    else if (k === 'u') copy(skipLeb(body, at));
    else if (k === 'b') copy(at + 1);
    else if (k === '4' || k === '8') copy(at + +k);
    else if (k === 'v') for (let n = uleb(body, at) + 1, i = 0; i <= n; i++) copy(skipLeb(body, at));
    else if (k === 'y') for (let n = uleb(body, at), i = 0; i <= n; i++) copy(i ? at + 1 : skipLeb(body, at));
    else if (k === 'k') {
      if (VALTYPE.has(body[at])) copy(at + 1);
      else (named(ty, sleb(body, at), 'type'), (at = skipLeb(body, at)));
    } else if (k === 'a' || k === 'm') {
      if (k === 'm') {
        if (body[at] & 0x40) throw new Error('a memarg naming a memory');
        copy(skipLeb(body, at));
      }
      const v = k === 'a' ? sleb(body, at) : uleb(body, at);
      const end = skipLeb(body, at);
      const padded = end - at === 5 && Math.abs(v) < 2 ** 27;
      if (padded && v >= lo && v <= hi) (addr(v), (at = end));
      else if (padded && k === 'a' && slots[v]) (out.push(...slots[v]), (at = end));
      else copy(end);
    }
  };
  for (let n = uleb(body, 0), i = 0; i <= n; i++) copy(i ? skipLeb(body, at) + 1 : skipLeb(body, at));
  for (let depth = 0; ; ) {
    if (at >= body.length) throw new Error('the body ends inside a block');
    const op = body[at];
    copy(at + 1);
    if (op === 0x02 || op === 0x03 || op === 0x04) depth++;
    else if (op === 0x0b && !depth--) break;
    if (op === 0xfc) {
      const sub = uleb(body, at);
      if (sub >= FC.length) throw new Error(`unknown instruction 0xfc ${sub}`);
      copy(skipLeb(body, at));
      for (const k of FC[sub]) imm(k);
    } else if (OPS.has(op)) for (const k of OPS.get(op)) imm(k);
    else throw new Error(`unknown instruction 0x${op.toString(16)}`);
  }
  if (at !== body.length) throw new Error(`the body's last \`end\` is ${body.length - at} B before its end`);
  return Buffer.from(out);
}

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

/** A names build's gate buckets: crate -> fn -> normalized B (normalizedFunctions), and the data segment's raw B. */
export function normalizedSizes(wasm, rows) {
  const out = Object.fromEntries([...normalizedFunctions(wasm)].map(([c, fns]) => [c, Object.fromEntries([...fns].map(([f, b]) => [f, b.length]))]));
  const data = rows.find(([c]) => c === DATA);
  if (data) out[DATA] = { 'data segment': data[1] };
  return out;
}

/** The buckets of `fns` whose normalized sum grew against `parent`'s: bucket -> {was, now, top: the 3 fns that grew most}. */
export function grown(fns, parent) {
  const sum = (m) => Object.values(m ?? {}).reduce((s, n) => s + n, 0);
  const out = {};
  for (const [c, now] of Object.entries(fns)) {
    const was = parent[c] ?? {};
    if (sum(now) <= sum(was)) continue;
    const top = Object.entries(now).map(([f, n]) => [f, n - (was[f] ?? 0)]).filter(([, d]) => d > 0).sort((a, b) => b[1] - a[1]).slice(0, 3);
    out[c] = { was: sum(was), now: sum(now), top };
  }
  return out;
}

/** The buckets of `fns` whose normalized sum dropped against `parent`'s: bucket -> B dropped. */
export function shrunk(fns, parent) {
  const sum = (m) => Object.values(m ?? {}).reduce((s, n) => s + n, 0);
  return Object.fromEntries(Object.keys(parent).map((c) => [c, sum(parent[c]) - sum(fns[c])]).filter(([, d]) => d > 0));
}

/**
 * A budget transfer (decisions.md 634): against the parent's `was` budgets, a land may
 * raise a bucket's budget only by what it lowers others (the total never rises), and
 * only by what the lowered buckets' normalized code dropped (`drops`, from shrunk), so
 * a budget follows moved code and can't hide new growth. No drops measured: no raise.
 * A raise the coordinator signed (`signed`, from signedRaises) is exact and outside the
 * transfer: a bucket over a budget its code never fit, or code moved before the parent.
 */
export function budgetTransfer(budgets, was, drops = {}, signed = {}) {
  const b = (n) => n.toLocaleString('en-US');
  const raised = Object.entries(budgets).map(([c, n]) => [c, n - (was[c] ?? 0)])
    .filter(([c, d]) => d > 0 && !(signed[c] && signed[c][0] === (was[c] ?? 0) && signed[c][1] === budgets[c]));
  if (!raised.length) return [];
  const cuts = Object.entries(was).map(([c, n]) => [c, n - (budgets[c] ?? 0)]).filter(([, d]) => d > 0);
  const total = (xs) => xs.reduce((s, [, d]) => s + d, 0);
  const up = total(raised);
  const what = `wasm-crate-budgets.json raises ${raised.map(([c, d]) => `${c} +${b(d)}`).join(', ')} (${b(up)} B)`;
  const errors = [];
  if (up > total(cuts)) errors.push(`${what} but lowers others by only ${b(total(cuts))} B: a budget moves with its code and the total never rises`);
  const moved = cuts.reduce((s, [c, d]) => s + Math.min(d, drops[c] ?? 0), 0);
  const from = cuts.map(([c, d]) => `${c} -${b(d)} budget, -${b(drops[c] ?? 0)} B code`).join('; ') || 'nothing lowered';
  if (up > moved) errors.push(`${what} but the lowered buckets' normalized code dropped only ${b(moved)} B of it (${from}): a transfer follows code that moved`);
  return errors;
}

/**
 * The land gate's ratchet against the exact same-tree parent, for every crate and the
 * data segment alike (touched is only a label): one over its budget (gz) after the change
 * is refused if its normalized code (`growth`, from grown) grew, so a crate within budget
 * can't cross it and one over can't grow, while layout-only gz drift never refuses.
 * Normalized growth of GREW_WARN B or more within budget warns.
 */
export function ratchet(rows, budgets, map, growth) {
  const errors = [];
  const held = [];
  const b = (n) => n.toLocaleString('en-US');
  for (const row of rows) {
    const [crate] = row;
    const over = crate === DATA ? dataOverBudget(rows, budgets) : checkBudgets([row], budgets, map).errors[0];
    if (!over) continue;
    const g = growth[crate];
    if (!g) held.push(`${over}, but its normalized code is no larger than its parent's`);
    else {
      const top = g.top.map(([f, d]) => `${f} +${b(d)}`).join(', ');
      errors.push(`${over}; it grew ${b(g.now - g.was)} B against its parent's ${b(g.was)} B of normalized code (now ${b(g.now)} B)${top ? `; grew most: ${top}` : ''}`);
    }
  }
  const grew = rows
    .filter(([c, , gz]) => growth[c] && growth[c].now - growth[c].was >= GREW_WARN && gz <= (budgets[c] ?? -1))
    .map(([c]) => `${c} grew ${b(growth[c].now - growth[c].was)} B of normalized code against its parent's ${b(growth[c].was)} B`);
  return { errors, warnings: [], held, grew };
}

/**
 * The coordinator-signed budget raises in commit trailers' values, one per line:
 * `Budget-raise: cpl_ir 445500->470872, signed off by coordinator` -> {cpl_ir: [445500, 470872]}.
 * A row without the sign-off is no raise.
 */
export function signedRaises(text) {
  const out = {};
  for (const line of text.split('\n')) {
    const m = line.trim().match(/^(\S+) (\d+)->(\d+), signed off by coordinator$/);
    if (m) out[m[1]] = [Number(m[2]), Number(m[3])];
  }
  return out;
}

/** The old pipeline's frozen crates: they only shrink, so a lowered budget has no room. */
export const FROZEN_CRATES = new Set(['cpl_ir', 'cpl_check', 'cpl_interp']);

/** A lowered budget for a crate measured at `gz`: 1% or 256 B of room, to the next 100. */
export const lowerTo = (gz) => Math.ceil((gz + Math.max(gz * 0.01, 256)) / 100) * 100;

/**
 * The budgets after a build within them: each budget with more room than lowerTo
 * gives over its crate's measurement comes down to lowerTo, so slack a cut freed
 * can't be regrown; a frozen crate's comes down to its measurement. Budgets only go down.
 * Null when nothing lowers.
 */
export function lowered(rows, budgets) {
  const next = { ...budgets };
  for (const [crate, , gz] of rows) {
    const to = FROZEN_CRATES.has(crate) ? gz : lowerTo(gz);
    if (budgets[crate] !== undefined && to < budgets[crate]) next[crate] = to;
  }
  return Object.keys(next).some((c) => next[c] !== budgets[c]) ? next : null;
}

/** A crate growing at least this many B of normalized code against its parent warns at land. */
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
// crates over theirs only warn. With `--parent`, a parent build's sizes (and normalized
// sizes, `fns`), every crate and the data segment are held to the ratchet, and the sort copies only grow if they
// grew. `--normalized DIR/cpl.wasm OUT.json` writes normalizedFunctions as
// {crate: {fn: hex}}. `--parent-budgets FILE` (the parent's wasm-crate-budgets.json)
// checks a budget transfer against the code that moved, and `--budgets FILE` alone
// refuses any raise (budgetTransfer) but the coordinator-signed `Budget-raise:` rows land.sh
// passes in CPL_BUDGET_RAISES (signedRaises). `--sizes DIR/cpl.wasm SIZES.json` writes a build's sizes for `--parent`, and
// `--data` takes such a file in place of a build. `--lower-budgets` lowers each budget
// with slack after a pass (lowered).
if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href && process.argv[2] === '--normalized') {
  const crates = normalizedFunctions(readFileSync(process.argv[3]));
  const json = Object.fromEntries([...crates].map(([c, fns]) => [c, Object.fromEntries([...fns].map(([f, b]) => [f, b.toString('hex')]))]));
  writeFileSync(process.argv[4], `${JSON.stringify(json)}\n`);
} else if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href && process.argv[2] === '--budgets') {
  const here = dirname(fileURLToPath(import.meta.url));
  const budgets = JSON.parse(readFileSync(join(here, 'wasm-crate-budgets.json'), 'utf8'));
  const errors = budgetTransfer(budgets, JSON.parse(readFileSync(process.argv[3], 'utf8')), {}, signedRaises(process.env.CPL_BUDGET_RAISES ?? ''));
  for (const e of errors) console.error(`wasm-crates: ${e} (no wasm build measured the moved code; build one: scripts/wasm-warm.sh --wait)`);
  process.exit(errors.length ? 1 : 0);
} else if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href && process.argv[2] === '--sizes') {
  const wasm = readFileSync(process.argv[3]);
  // A sizes file standing in for a build (scripts/test_land.py's builds) is its own sizes.
  const crates = wasm[0] === 0x7b ? null : Object.fromEntries(contributors(wasm).map(([c, , gz]) => [c, gz]));
  const fns = crates && normalizedSizes(wasm, contributors(wasm));
  writeFileSync(process.argv[4], crates ? `${JSON.stringify({ crates, sorts: sortCopies(functionNames(wasm)).length, fns })}\n` : wasm);
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
  if (parent && !parent.fns) {
    console.error(`wasm-crates: ${flag('--parent')} has no normalized sizes to hold the crates to; record the parent again (scripts/wasm-warm.sh --wait)`);
    process.exit(1);
  }
  const { errors, warnings, held = [], grew = [] } = parent
    ? ratchet(rows, budgets, map, grown(sized ? sized.fns ?? {} : normalizedSizes(wasm, rows), parent.fns))
    : touchedOverBudget(rows, budgets, map, touched);
  const parentBudgets = flag('--parent-budgets') && JSON.parse(readFileSync(flag('--parent-budgets'), 'utf8'));
  if (parentBudgets) errors.push(...budgetTransfer(budgets, parentBudgets, shrunk(sized ? sized.fns ?? {} : normalizedSizes(wasm, rows), parent?.fns ?? {}), signedRaises(process.env.CPL_BUDGET_RAISES ?? '')));
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

// What `wasm-size.mjs --why` breaks down: the names build it was given, so an A/B of
// two builds compares them; a fresh build of the checkout only when it was given none.
export const whyWasm = (names, { exists, read, build }) => (names && exists(names) ? read(names) : build());
