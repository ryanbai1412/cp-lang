import { describe, expect, test } from 'vitest';
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { budgetFor, budgetTransfer, checkBudgets, checkSortCopies, crateOf, gateCrateOf, grown, normalizedFunctions, normalizedSizes, shrunk, sharedName, DATA, dataOverBudget, lowered, lowerTo, ownerOf, signedRaises, FROZEN_CRATES, ratchet, owners, readCharters, sortCopies, touchedOverBudget, whyWasm } from './wasm-crates.mjs';

const charters = {
  compiler: '| Kind | Metric |\n| Describes | `crates/cpl-syntax`, `crates/cpl-check`, `crates/cpl-interp` |\n',
  tools: '| Describes | `crates/cpl-cli`, `crates/cpl-lsp/`, `editors/` |\n',
  debugger: '| Describes | `crates/cpl-interp/src/trace.rs`, `crates/cpl-stress/` |\n',
  site: 'cpl-new is mentioned here, but not in a Describes row.\n',
};

describe('wasm crate budgets', () => {
  test('crate names come from v0 mangling', () => {
    expect(crateOf('_RNvCs1a2b_9cpl_check4main')).toBe('cpl_check');
    expect(crateOf('memcpy')).toBe('(runtime)');
    expect(crateOf('_RNvMs2_NtNtCsiErZIu7dPQ0_4core3fmt8buildersNtB5_11DebugStruct9is_pretty')).toBe('core');
  });

  test('a std generic is charged to the cpl crate of its type, not the crate the linker kept', () => {
    const drop = '_RINvNtCsiErZIu7dPQ0_4core3ptr9drop_glueTNtNtCsd7qzcshS4qx_5alloc6string6StringNtNtCs3AcHtDQUFWa_9cpl_check2ty2TyReE';
    expect(crateOf(drop)).toBe('cpl_check');
    expect(crateOf(`${drop}Csamg1HCxCxBg_7cpl_lsp`)).toBe('cpl_check');
    expect(crateOf('_RNvMNtCsiErZIu7dPQ0_4core5slice6reverseCsamg1HCxCxBg_7cpl_doc')).toBe('core');
  });

  test('the data segment over its budget is named, and within it is not', () => {
    const rows = [['cpl_check', 10, 5], [DATA, 900, 300]];
    expect(dataOverBudget(rows, { [DATA]: 300 })).toBeNull();
    expect(dataOverBudget(rows, { [DATA]: 299 })).toMatch(/300 B gzipped, over its budget of 299/);
    expect(dataOverBudget([['cpl_check', 10, 5]], { [DATA]: 1 })).toBeNull();
  });

  test('owners come from each charter\'s Describes row', () => {
    const map = owners(charters);
    expect(ownerOf('cpl_check', map)).toBe('compiler');
    expect(ownerOf('cpl_lsp', map)).toBe('tools');
    expect(ownerOf('cpl_interp', map)).toBe('compiler');
    expect(ownerOf('cpl_stress', map)).toBe('debugger');
    expect(ownerOf('core', map)).toBe('infra');
    expect(ownerOf(DATA, map)).toBe('stdlib');
    expect(ownerOf('cpl_new', map)).toBe(null);
  });

  test('a crate over its budget names itself and its lane', () => {
    const map = owners(charters);
    const rows = [['cpl_check', 900, 300], ['cpl_lsp', 500, 100], ['cpl_new', 50, 20]];
    const { errors, lower } = checkBudgets(rows, { cpl_check: 250, cpl_lsp: 5000, gone: 5 }, map);
    expect(errors).toEqual([
      'cpl_check is 300 B gzipped, over its budget of 250: the compiler lane (handbook/domains/compiler.md) grew it',
      'cpl_new (no domain: add crates/cpl-new to the Describes row of its charter in handbook/domains/) has no budget in web/scripts/wasm-crate-budgets.json; it is 20 B gzipped',
    ]);
    expect(lower).toEqual([`cpl_lsp: lower its budget to ${budgetFor(100)}`, 'gone is no longer in cpl.wasm: delete its budget']);
  });

  test('the commit gate fails a touched crate over budget and warns for the rest', () => {
    const map = owners(charters);
    const rows = [['cpl_check', 900, 300], ['cpl_lsp', 500, 200], [DATA, 900, 50]];
    const budgets = { cpl_check: 250, cpl_lsp: 100, [DATA]: 40 };
    const { errors, warnings } = touchedOverBudget(rows, budgets, map, ['cpl_lsp']);
    expect(errors).toHaveLength(2);
    expect(errors[0]).toMatch(/^data .* 50 B gzipped, over its budget of 40/);
    expect(errors[1]).toMatch(/^cpl_lsp is 200 B gzipped/);
    expect(warnings).toEqual([expect.stringMatching(/^cpl_check is 300 B gzipped/)]);
    expect(touchedOverBudget(rows, { ...budgets, [DATA]: 50 }, map, []).errors).toEqual([]);
  });

  test('the ratchet refuses a bucket over budget whose normalized code grew, touched or not', () => {
    const map = owners(charters);
    const budgets = { cpl_check: 250, cpl_lsp: 100, [DATA]: 40 };
    const gate = (rows, fns, parent) => ratchet(rows, budgets, map, grown(fns, parent));
    const was = { cpl_check: { a: 600, b: 300 }, cpl_lsp: { c: 500 }, [DATA]: { 'data segment': 900 } };
    // Renumbering only: normalized bytes equal, gz drifts past the budget; held, not refused.
    const drift = gate([['cpl_check', 900, 300], ['cpl_lsp', 500, 101], [DATA, 900, 50]], was, was);
    expect(drift.errors).toEqual([]);
    expect(drift.held).toHaveLength(3);
    // One fn +1 B in an over-budget crate, nothing else shrinking: refused, with the numbers.
    const plus = gate([['cpl_check', 900, 300]], { ...was, cpl_check: { a: 601, b: 300 } }, was);
    expect(plus.errors).toEqual([expect.stringMatching(/^cpl_check is 300 B gzipped, over its budget of 250.*; it grew 1 B against its parent's 900 B of normalized code \(now 901 B\); grew most: a \+1$/)]);
    // Bytes moving between a crate's fns aren't growth.
    expect(gate([['cpl_check', 900, 300]], { ...was, cpl_check: { a: 599, b: 301 } }, was).errors).toEqual([]);
    // A within-budget crate that grows and crosses its budget is refused; one that grows within it passes.
    const lsp = { ...was, cpl_lsp: { c: 500, d: 7 } };
    expect(gate([['cpl_lsp', 507, 101]], lsp, was).errors).toEqual([expect.stringMatching(/^cpl_lsp is 101 B gzipped.*grew most: d \+7$/)]);
    expect(gate([['cpl_lsp', 507, 100]], lsp, was).errors).toEqual([]);
    // The data segment by its raw bytes.
    expect(gate([[DATA, 901, 50]], { ...was, [DATA]: { 'data segment': 901 } }, was).errors).toEqual([expect.stringMatching(/^data .* it grew 1 B/)]);
    // A crate with no parent record grew from nothing.
    expect(gate([['cpl_check', 900, 300]], was, {}).errors).toEqual([expect.stringMatching(/^cpl_check is 300 B.*it grew 900 B/)]);
    const big = { ...was, cpl_lsp: { c: 700 } };
    expect(ratchet([['cpl_lsp', 700, 90]], budgets, map, grown(big, was)).grew).toEqual(['cpl_lsp grew 200 B of normalized code against its parent\'s 500 B']);
    expect(ratchet([['cpl_lsp', 699, 90]], budgets, map, grown({ ...was, cpl_lsp: { c: 699 } }, was)).grew).toEqual([]);
  });

  test('a budget moves with code that moved, and the total never rises', () => {
    // VM-memory's narrow storage: IntWidth's code moves cpl_ir -> cpl_rt.
    const was = { cpl_ir: 445800, cpl_rt: 500 };
    const parentFns = { cpl_ir: { a: 9000, w: 5550 }, cpl_rt: { r: 811 } };
    const fns = { cpl_ir: { a: 9000 }, cpl_rt: { r: 811, w: 5550 } };
    const drops = shrunk(fns, parentFns);
    expect(drops).toEqual({ cpl_ir: 5550 });
    expect(budgetTransfer({ cpl_ir: 443300, cpl_rt: 3000 }, was, drops)).toEqual([]);
    const map = owners(charters);
    expect(ratchet([['cpl_rt', 6361, 2900]], { cpl_ir: 443300, cpl_rt: 3000 }, map, grown(fns, parentFns)).errors).toEqual([]);
    // A raise with no matching cut: the total rises.
    expect(budgetTransfer({ cpl_ir: 445800, cpl_rt: 3000 }, was, drops))
      .toEqual([expect.stringMatching(/raises cpl_rt \+2,500 \(2,500 B\) but lowers others by only 0 B/), expect.stringMatching(/dropped only 0 B/)]);
    // A transfer bigger than the code that moved.
    expect(budgetTransfer({ cpl_ir: 439800, cpl_rt: 6500 }, was, drops))
      .toEqual([expect.stringMatching(/^wasm-crate-budgets.json raises cpl_rt \+6,000 \(6,000 B\) but the lowered buckets' normalized code dropped only 5,550 B of it \(cpl_ir -6,000 budget, -5,550 B code\)/)]);
    // A cut from a bucket whose code didn't drop funds nothing.
    expect(budgetTransfer({ cpl_ir: 445800, cpl_check: 100, cpl_rt: 600 }, { ...was, cpl_check: 200 }, drops)).toHaveLength(1);
    // Lowering alone, and no measurement, are fine; a raise with nothing measured refuses.
    expect(budgetTransfer({ cpl_ir: 440000, cpl_rt: 500 }, was)).toEqual([]);
    expect(budgetTransfer({ cpl_ir: 440000, cpl_rt: 600 }, was)).toHaveLength(1);
  });

  test('a raise the coordinator signed lands exactly, and an unsigned or inexact one refuses', () => {
    const was = { cpl_ir: 445500, cpl_rt: 500 };
    const signed = signedRaises('cpl_ir 445500->470872, signed off by coordinator\ncpl_rt 500->900\n');
    expect(signed).toEqual({ cpl_ir: [445500, 470872] });
    expect(signedRaises('data (std source, strings) 206300->209300, signed off by coordinator'))
      .toEqual({ 'data (std source, strings)': [206300, 209300] });
    expect(budgetTransfer({ cpl_ir: 470872, cpl_rt: 500 }, was, {}, signed)).toEqual([]);
    expect(budgetTransfer({ cpl_ir: 470872, cpl_rt: 900 }, was, {}, signed)).not.toEqual([]);
    expect(budgetTransfer({ cpl_ir: 471000, cpl_rt: 500 }, was, {}, signed)).not.toEqual([]);
    expect(budgetTransfer({ cpl_ir: 470872, cpl_rt: 500 }, { ...was, cpl_ir: 445000 }, {}, signed)).not.toEqual([]);
  });

  test("a frozen crate's budget lowers to its measurement, with no room", () => {
    expect(FROZEN_CRATES.has('cpl_ir')).toBe(true);
    expect(lowered([['cpl_ir', 1, 470000], ['cpl_rt', 1, 100]], { cpl_ir: 470872, cpl_rt: 500 }))
      .toEqual({ cpl_ir: 470000, cpl_rt: 400 });
  });

  test('an over-budget crate that grows through a dependency it calls is refused', () => {
    // cpl_lint's change inlines into cpl_lsp (over budget): cpl_lsp's own fn grows 1 B.
    const prev = build({ fns: [[fn('cpl_lsp', 'f'), call(1)], [fn('cpl_lint', 'g'), [0x01]]] });
    const next = build({ fns: [[fn('cpl_lint', 'g'), [0x01, 0x01]], [fn('cpl_lsp', 'f'), [...call(0), 0x01]]] });
    const sizes = (w) => normalizedSizes(w, []);
    const growth = grown(sizes(next), sizes(prev));
    expect(Object.keys(growth).sort()).toEqual(['cpl_lint', 'cpl_lsp']);
    const r = ratchet([['cpl_lsp', 9, 101], ['cpl_lint', 3, 1]], { cpl_lsp: 100, cpl_lint: 100 }, owners(charters), growth);
    expect(r.errors).toEqual([expect.stringMatching(/^cpl_lsp is 101 B gzipped.*it grew 1 B/)]);
  });

  test('a build lowers each budget with slack to its measurement and some room, never up', () => {
    const budgets = { core: 71100, cpl_json: 19600, cpl_rt: 500, cpl_check: 250, gone: 5 };
    expect(lowered([['core', 1, 62220], ['cpl_json', 1, 14865], ['cpl_rt', 1, 100], ['cpl_check', 1, 300]], budgets))
      .toEqual({ core: 62900, cpl_json: 15200, cpl_rt: 400, cpl_check: 250, gone: 5 });
    expect(lowered([['core', 1, 70300]], budgets)).toBeNull();
  });

  test('each element type and crate a sort is compiled for is one copy of core\'s sort', () => {
    const unstable = (t, c) => `_RINvNtNtNtCsiErZIu7dPQ0_4core5slice4sort8unstable7ipnsort${t}NvYB1b_NtNtBa_3cmp10PartialOrd2ltE${c}`;
    const stable = '_RINvNtNtNtCsiErZIu7dPQ0_4core5slice4sort6stable14driftsort_mainjNvYjNtNtBa_3cmp10PartialOrd2ltECsjqR3RLP4pKN_6cpl_ir';
    const names = [
      unstable('Re', 'Cs7ddMHjwixPc_7cpl_fmt'),
      unstable('Re', 'CsigG2x4UxwvU_8cpl_lint'),
      unstable('Re', 'CsigG2x4UxwvU_8cpl_lint'),
      stable,
      '_RINvNtNtNtNtCsiErZIu7dPQ0_4core5slice4sort8unstable9quicksort9quicksortReNvYB17_NtNtBa_3cmp10PartialOrd2ltECsigG2x4UxwvU_8cpl_lint',
      '_RNvCs1a2b_9cpl_check4sort',
    ];
    const copies = sortCopies(names);
    expect(copies).toHaveLength(3);
    expect(checkSortCopies(copies, 3)).toEqual({ error: null, lower: null });
    const over = checkSortCopies(copies, 2);
    expect(over.error).toMatch(/^cpl.wasm has 3 copies of core's sort, over the 2 in web\/scripts\/wasm-sort-copies.json/);
    expect(over.error).toContain('\n  cpl_fmt: ');
    expect(over.error).toContain('\n  cpl_ir: ');
    expect(checkSortCopies(copies, 5).lower).toMatch(/lower "max" in web\/scripts\/wasm-sort-copies.json to 3$/);
  });

  // A names build: `fns` ([name, body instructions] in index order), `types` signatures,
  // a table of `slots` (function indices) and `pad` bytes of data at 1024 ahead of a static.
  const uleb = (v, pad = 0) => {
    const out = [];
    do out.push((v & 0x7f) | (v >= 128 || out.length + 1 < pad ? 0x80 : 0)), (v = Math.floor(v / 128)); while (v || out.length < pad);
    out[out.length - 1] &= 0x7f;
    return out;
  };
  const str = (s) => [...uleb(s.length), ...Buffer.from(s)];
  const vec = (items) => [...uleb(items.length), ...items.flat()];
  const section = (id, bytes) => [id, ...uleb(bytes.length), ...bytes];
  const build = ({ fns, types = [[0x60, 0, 0]], slots = [], data = 8 }) => Buffer.from([
    0, 0x61, 0x73, 0x6d, 1, 0, 0, 0,
    ...section(1, vec(types)),
    ...section(5, [1, 0, 1]),
    ...section(9, vec([[0, 0x41, 1, 0x0b, ...vec(slots.map((f) => uleb(f)))]])),
    ...section(10, vec(fns.map(([, code]) => [...uleb(code.length + 2), 0, ...code, 0x0b]))),
    ...section(11, vec([[0, 0x41, ...uleb(1024, 2), 0x0b, ...uleb(data + 4), ...Array(data + 4).fill(7)]])),
    ...section(0, [...str('name'), ...section(1, vec(fns.map(([n], i) => [...uleb(i), ...str(n)])))]),
  ]);
  const fn = (crate, f) => `_RNvCs1_${crate.length}${crate}${f.length}${f}`;
  // wasm-ld's relocated immediates: padded 5-byte LEBs.
  const call = (i) => [0x10, ...uleb(i, 5)];
  const indirect = (t) => [0x41, 0, 0x11, ...uleb(t, 5), 0];
  const slot = (i) => [0x41, ...uleb(i, 5), 0x1a];
  const load = (addr) => [0x41, 0, 0x28, 2, ...uleb(addr, 5), 0x1a];
  const fnsOf = (wasm, crate) => normalizedFunctions(wasm).get(crate);

  test('a crate normalizes the same when other crates renumber its calls, types, table slots and statics', () => {
    // cpl_a calls itself and cpl_b, calls indirectly through type (i32) -> (), takes cpl_b's
    // address and reads a static that sits after `data` bytes of other crates' statics.
    const a = (self, b, type, at, data) => [
      [fn('cpl_a', 'f'), [...call(self + 1), ...call(b), ...indirect(type), ...slot(at), ...load(1024 + data), 0x02, 0x40, 0x0b]],
      [fn('cpl_a', 'g'), call(self)],
    ];
    const sig = [0x60, 1, 0x7f, 0];
    const before = build({ fns: [...a(0, 2, 1, 1, 8), [fn('cpl_b', 'h'), [0x01]]], types: [[0x60, 0, 0], sig], slots: [2] });
    const after = build({
      fns: [[fn('cpl_b', 'x'), [0x01]], [fn('cpl_b', 'y'), call(0)], [fn('cpl_b', 'h'), [0x01]], ...a(3, 2, 2, 2, 40)],
      types: [[0x60, 0, 0], [0x60, 0, 1, 0x7f], sig],
      slots: [0, 2],
      data: 40,
    });
    expect(fnsOf(after, 'cpl_a')).toEqual(fnsOf(before, 'cpl_a'));
    expect([...fnsOf(after, 'cpl_b').keys()]).toEqual([fn('cpl_b', 'x'), fn('cpl_b', 'y'), fn('cpl_b', 'h')]);
  });

  test('a real change to a body changes its normalized bytes, and growth makes them longer', () => {
    const base = (extra, callee = 1) => build({ fns: [[fn('cpl_a', 'f'), [...call(callee), ...extra]], [fn('cpl_a', 'g'), [0x01]], [fn('cpl_a', 'k'), [0x01]]] });
    const was = fnsOf(base([]), 'cpl_a');
    const grown = fnsOf(base([0x41, 7, 0x1a]), 'cpl_a');
    expect(grown.get(fn('cpl_a', 'f')).length).toBe(was.get(fn('cpl_a', 'f')).length + 3);
    expect(grown.get(fn('cpl_a', 'g'))).toEqual(was.get(fn('cpl_a', 'g')));
    expect(fnsOf(base([], 2), 'cpl_a').get(fn('cpl_a', 'f'))).not.toEqual(was.get(fn('cpl_a', 'f')));
  });

  test("the gate charges a generic instantiated on a closure to the closure's crate", () => {
    // cpl_diag's sort_by_key closure, instantiated on cpl_lint::library::find's closure,
    // the second time through a backref to cpl_diag's Span.
    const sort = '_RNCINvCs92dA4jsxNq4_8cpl_diag11sort_by_keyTjjNtB4_4SpanENCNvNtCsigG2x4UxwvU_8cpl_lint7library4finds7_0E0BY_';
    expect(crateOf(sort)).toBe('cpl_diag');
    expect(gateCrateOf(sort)).toBe('cpl_lint');
    // A closure whose path is a backref (`B<base62>_`, an offset after `_R`) to cpl_lint's module.
    const head = '_RINvCs1_8cpl_diag4sortNCNvNtCs2_8cpl_lint1m1g0NCNvB';
    const back = '_RINvCs1_8cpl_diag4sortNCNv'.length - 2 - 1;
    expect(gateCrateOf(`${head}${back.toString(36)}_1h0E`)).toBe('cpl_lint');
    expect(gateCrateOf('_RINvCs1_8cpl_diag4sortNCNvCs1_8cpl_diag1g0E')).toBe('cpl_diag');
    expect(gateCrateOf('_RINvNtCsiErZIu7dPQ0_4core3mem4dropINtNtCsd7qzcshS4qx_5alloc3vec3VecjEE')).toBe('core');
    expect(gateCrateOf(fn('cpl_a', 'f'))).toBe('cpl_a');
    expect(gateCrateOf('memcpy')).toBe('(runtime)');
    const user = (closure) => build({ fns: [[`_RINvCs1_8cpl_diag4sortNC${closure}0E`, [0x01]]] });
    expect([...normalizedFunctions(user('NvCs2_8cpl_lint1g')).keys()]).toEqual(['cpl_lint']);
  });

  test('a std generic goes by its name without the crate the linker kept it from', () => {
    const drop = '_RNvNtCsiErZIu7dPQ0_4core3ptr4dropCs8Jhg9rW6Oc3_10cpl_interp';
    expect(sharedName(drop)).toBe('_RNvNtCsiErZIu7dPQ0_4core3ptr4drop');
    expect(sharedName(fn('cpl_a', 'f'))).toBe(fn('cpl_a', 'f'));
    const user = (kept) => build({ fns: [[fn('cpl_a', 'f'), call(1)], [`_RNvNtCsiErZIu7dPQ0_4core3ptr4dropCs1_${kept.length}${kept}`, [0x01]]] });
    expect(fnsOf(user('cpl_b'), 'cpl_a')).toEqual(fnsOf(user('cpl_check'), 'cpl_a'));
  });

  test('an instruction the normalizer cannot read fails it instead of misreading the crate', () => {
    expect(() => normalizedFunctions(build({ fns: [[fn('cpl_a', 'f'), [0xfd, 0x0c]]] }))).toThrow(/cpl_a1f: unknown instruction 0xfd/);
    expect(() => normalizedFunctions(build({ fns: [[fn('cpl_a', 'f'), [0x0b, 0x01]]] }))).toThrow(/last `end` is 2 B before its end/);
    expect(() => normalizedFunctions(build({ fns: [[fn('cpl_a', 'f'), call(9)]] }))).toThrow(/function 9 does not exist/);
  });

  test('within budget passes', () => {
    expect(checkBudgets([['cpl_check', 1, 1000]], { cpl_check: budgetFor(1000) }, owners(charters)).errors).toEqual([]);
  });
});

test('every crate in cpl.wasm has a charter that describes it', () => {
  const map = owners(readCharters(join(dirname(fileURLToPath(import.meta.url)), '../../handbook/domains')));
  const budgets = JSON.parse(readFileSync(join(dirname(fileURLToPath(import.meta.url)), 'wasm-crate-budgets.json'), 'utf8'));
  const unowned = Object.keys(budgets).filter((c) => ownerOf(c, map) === null);
  expect(unowned).toEqual([]);
});

test('--why breaks down the WASM_NAMES build it is given, building only without one', () => {
  const built = [];
  const io = { exists: (f) => f === 'a.wasm', read: (f) => `read ${f}`, build: () => (built.push(1), 'fresh') };
  expect(whyWasm('a.wasm', io)).toBe('read a.wasm');
  expect(built).toEqual([]);
  expect(whyWasm(undefined, io)).toBe('fresh');
  expect(whyWasm('missing.wasm', io)).toBe('fresh');
});
