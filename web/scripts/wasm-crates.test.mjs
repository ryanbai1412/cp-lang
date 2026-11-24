import { describe, expect, test } from 'vitest';
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { budgetFor, checkBudgets, crateOf, DATA, ownerOf, owners, readCharters } from './wasm-crates.mjs';

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
