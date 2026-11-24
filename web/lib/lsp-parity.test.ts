// site/R6 (handbook/guidelines/site-checklist.md): the browser editor is the
// language server. Every request the editor (components/editor/) sends, the
// browser build of the server answers rather than calling it unknown; the
// advertised capabilities the editor never asks for are the
// `browser_lsp_gaps` counter in scripts/debt-ratchet.py, which only shrinks.
import { existsSync, readdirSync, readFileSync } from 'node:fs';
import { join } from 'node:path';
import { expect, test } from 'vitest';

const repo = join(__dirname, '../..');
const dist = join(repo, 'dist/wasm');
const dir = join(__dirname, '../components/editor');
const editor = readdirSync(dir)
  .filter((f) => f.endsWith('.ts') && !f.endsWith('.test.ts'))
  .map((f) => readFileSync(join(dir, f), 'utf8'))
  .join('\n');
const asked = [...new Set([...editor.matchAll(/"((?:textDocument|workspace|callHierarchy)\/[\w/]+)"/g)].map((m) => m[1]))];

test('the language server answers every request the editor sends', async () => {
  expect(existsSync(join(dist, 'cpl.wasm')), 'build the browser compiler first: scripts/build-wasm.sh').toBe(true);
  const { load } = await import(/* @vite-ignore */ join(dist, 'cpl.js'));
  const cpl = await load(readFileSync(join(dist, 'cpl.wasm')));
  cpl.lsp({ jsonrpc: '2.0', id: 0, method: 'initialize', params: {} });
  const uri = 'file:///parity.cp';
  const text = 'fn twice(n: Int) -> Int:\n    n * 2\nprint(twice(3))\n';
  cpl.lsp({ jsonrpc: '2.0', method: 'textDocument/didOpen', params: { textDocument: { uri, languageId: 'cp', version: 1, text } } });
  const position = { line: 2, character: 7 };
  const range = { start: { line: 0, character: 0 }, end: position };
  const item = { name: 'twice', kind: 12, uri, range, selectionRange: range };
  expect(asked.length).toBeGreaterThan(10);
  const unknown = asked.filter((method, id) => {
    const params = { textDocument: { uri }, position, range, item, query: '', newName: 'thrice', ch: '\n', options: { tabSize: 4, insertSpaces: true }, context: { diagnostics: [], includeDeclaration: true } };
    const replies = cpl.lsp({ jsonrpc: '2.0', id: id + 1, method, params }) as { id?: number; error?: { code: number } }[];
    return replies.find((r) => r.id === id + 1)?.error?.code === -32601;
  });
  expect(unknown, 'the editor sends what the server does not answer').toEqual([]);
});
