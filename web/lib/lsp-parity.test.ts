// site/R6 (handbook/guidelines/site-checklist.md): the browser editor is the
// language server. Asks the browser build of the server for its capabilities:
// every request the editor sends is one the server advertises, and the
// advertised ones the editor never asks for are the `browser_lsp_gaps` counter
// in scripts/debt-ratchet.py, which only shrinks.
import { existsSync, readFileSync } from 'node:fs';
import { join } from 'node:path';
import { expect, test } from 'vitest';

const repo = join(__dirname, '../..');
const dist = join(repo, 'dist/wasm');
const editor = readFileSync(join(__dirname, '../components/editor/intelligence.ts'), 'utf8');
// `textDocument/fooBar` is answered by the server's `fooBarProvider`.
const asked = [...editor.matchAll(/"(?:textDocument|workspace)\/(\w+)"/g)].map((m) => `${m[1]}Provider`);

test('every request the editor sends is a capability the language server advertises', async () => {
  expect(existsSync(join(dist, 'cpl.wasm')), 'build the browser compiler first: scripts/build-wasm.sh').toBe(true);
  const { load } = await import(/* @vite-ignore */ join(dist, 'cpl.js'));
  const cpl = await load(readFileSync(join(dist, 'cpl.wasm')));
  const [reply] = cpl.lsp({ jsonrpc: '2.0', id: 0, method: 'initialize', params: {} });
  const advertised = Object.keys(reply.result.capabilities);
  expect(asked.length).toBeGreaterThan(0);
  expect(asked.filter((c) => !advertised.includes(c)), 'the editor asks for what the server lacks').toEqual([]);
});
