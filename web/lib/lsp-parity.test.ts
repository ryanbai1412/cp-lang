// site/R6 (handbook/guidelines/site-checklist.md): the browser editor is the
// language server. Asks the browser build of the server for its capabilities
// and checks each is accounted for in lib/lsp-features.ts.
import { existsSync, readFileSync } from 'node:fs';
import { join } from 'node:path';
import { expect, test } from 'vitest';
import { lspFeatures } from './lsp-features';

const repo = join(__dirname, '../..');
const dist = join(repo, 'dist/wasm');

test('every capability the language server advertises is used or a named gap', async () => {
  expect(existsSync(join(dist, 'cpl.wasm')), 'build the browser compiler first: scripts/build-wasm.sh').toBe(true);
  const { load } = await import(/* @vite-ignore */ join(dist, 'cpl.js'));
  const cpl = await load(readFileSync(join(dist, 'cpl.wasm')));
  const [reply] = cpl.lsp({ jsonrpc: '2.0', id: 0, method: 'initialize', params: {} });
  const advertised = Object.keys(reply.result.capabilities).sort();
  expect(advertised.filter((c) => !(c in lspFeatures)), 'add these to lib/lsp-features.ts').toEqual([]);
  expect(Object.keys(lspFeatures).filter((c) => !advertised.includes(c)), 'no longer advertised').toEqual([]);
});

test('every request lib/lsp-features.ts lists is sent by the editor', () => {
  const source = readFileSync(join(__dirname, '../components/editor/intelligence.ts'), 'utf8');
  const uses = Object.values(lspFeatures).flatMap((f) => ('uses' in f ? f.uses : []));
  expect(uses.filter((m) => !source.includes(`"${m}"`))).toEqual([]);
});
