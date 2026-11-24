import { describe, expect, test } from 'vitest';
import { budgetFor, checkBudgets, crateOf, DATA, ownerOf, owners } from './wasm-crates.mjs';

const readme = [
  '| Domain | Owns | Excellent means |',
  '|---|---|---|',
  '| [compiler](compiler.md) | `crates/cpl-syntax`, `cpl-check` | x |',
  '| [tools](tools.md) | `crates/cpl-cli`, `cpl-lsp`, `editors/` | y |',
].join('\n');

describe('wasm crate budgets', () => {
  test('crate names come from v0 mangling', () => {
    expect(crateOf('_RNvCs1a2b_9cpl_check4main')).toBe('cpl_check');
    expect(crateOf('memcpy')).toBe('(runtime)');
  });

  test('owners come from the domains table', () => {
    const map = owners(readme);
    expect(ownerOf('cpl_check', map)).toBe('compiler');
    expect(ownerOf('cpl_lsp', map)).toBe('tools');
    expect(ownerOf('core', map)).toBe('infra');
    expect(ownerOf(DATA, map)).toBe('stdlib');
    expect(ownerOf('cpl_new', map)).toBe(null);
  });

  test('a crate over its budget names itself and its lane', () => {
    const map = owners(readme);
    const rows = [['cpl_check', 900, 300], ['cpl_lsp', 500, 100], ['cpl_new', 50, 20]];
    const { errors, lower } = checkBudgets(rows, { cpl_check: 250, cpl_lsp: 5000, gone: 5 }, map);
    expect(errors).toEqual([
      'cpl_check is 300 B gzipped, over its budget of 250: the compiler lane (handbook/domains/compiler.md) grew it',
      'cpl_new (no domain: add crates/cpl-new to handbook/domains/README.md) has no budget in web/scripts/wasm-crate-budgets.json; it is 20 B gzipped',
    ]);
    expect(lower).toEqual([`cpl_lsp: lower its budget to ${budgetFor(100)}`, 'gone is no longer in cpl.wasm: delete its budget']);
  });

  test('within budget passes', () => {
    expect(checkBudgets([['cpl_check', 1, 1000]], { cpl_check: budgetFor(1000) }, owners(readme)).errors).toEqual([]);
  });
});
