import { readdirSync, readFileSync } from 'node:fs';
import { join } from 'node:path';
import { pathToFileURL } from 'node:url';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

// A worker running a toy language server: it keeps each open document's text,
// answers a request with the text it has for that document, and needs the std
// modules a text names with `use std.<m>` given before it will check it.
type Message = { id?: number; method: string; params?: { textDocument?: { uri: string; text?: string }; contentChanges?: { text: string }[] } };

// A few turns of the event loop's microtask queue: an async step a later message could overtake.
const later = () => new Promise<void>((r) => queueMicrotask(() => queueMicrotask(() => queueMicrotask(r))));

class FakeWorker {
  log: string[] = [];
  docs = new Map<string, string>();
  given = new Set<string>();
  dead = false;
  async stdNeeded(source: string) {
    await later();
    return [...source.matchAll(/use std\.(\w+)/g)].map((m) => m[1]).filter((m) => !this.given.has(m));
  }
  async provideStd(module: string) {
    this.given.add(module);
    this.log.push(`provideStd ${module}`);
    return { changed: true };
  }
  async lsp(m: Message) {
    if (this.dead) throw Object.assign(new Error('cpl: the compiler stopped'), { kind: 'terminated' });
    const uri = m.params?.textDocument?.uri;
    const text = m.params?.textDocument?.text ?? m.params?.contentChanges?.at(-1)?.text;
    for (const n of [...(text ?? '').matchAll(/use std\.(\w+)/g)].map((x) => x[1]))
      if (!this.given.has(n)) throw new Error(`${m.method} before std.${n} was given`);
    if (m.method === 'textDocument/didOpen' || m.method === 'textDocument/didChange') this.docs.set(uri!, text!);
    if (m.method === 'textDocument/didClose') this.docs.delete(uri!);
    this.log.push(uri ? `${m.method} ${short(uri)}` : m.method);
    return m.id === undefined ? [] : [{ id: m.id, result: uri ? this.docs.get(uri) ?? null : `${m.method} answered` }];
  }
  terminate() {
    this.dead = true;
  }
}

const short = (uri: string) => uri.replace(/^file:\/\/\/editor-(\d+)\.cp$/, '#$1');

let workers: FakeWorker[];
let fetched: string[];

beforeEach(() => {
  vi.resetModules();
  workers = [];
  fetched = [];
  vi.stubGlobal('location', { origin: pathToFileURL(`${__dirname}/testing`).href });
  vi.stubGlobal('spawnFakeCpl', () => {
    const w = new FakeWorker();
    workers.push(w);
    return w;
  });
  vi.stubGlobal(
    'fetch',
    vi.fn(async (url: string) => {
      fetched.push(url.replace(/.*\/wasm\/std\//, ''));
      await later();
      return { ok: true, text: async () => 'std text' };
    }),
  );
});

afterEach(() => vi.unstubAllGlobals());

async function load() {
  const { LspDoc } = await import('./lsp');
  const cpl = await import('./cpl');
  return { LspDoc, cpl };
}

describe('LSP client', () => {
  it('initializes the server once, then opens a document before its first request', async () => {
    const { LspDoc } = await load();
    const doc = new LspDoc(() => 'let x = 1');
    expect(await doc.request('textDocument/hover', {})).toBe('let x = 1');
    expect(await doc.request('textDocument/hover', {})).toBe('let x = 1');
    expect(workers).toHaveLength(1);
    expect(workers[0].log).toEqual(['initialize', 'initialized', 'textDocument/didOpen #1', 'textDocument/hover #1', 'textDocument/hover #1']);
  });

  it('answers requests sent back to back against the latest text, even when it needs std fetched first', async () => {
    const { LspDoc } = await load();
    let text = 'let x = 1';
    const doc = new LspDoc(() => text);
    await doc.request('textDocument/hover', {});
    text = 'use std.sparse\nlet x = twice(';
    const replies = await Promise.all([doc.request('textDocument/completion', {}), doc.request('textDocument/signatureHelp', {})]);
    expect(replies).toEqual([text, text]);
    expect(workers[0].log.slice(4)).toEqual([
      'provideStd sparse',
      'textDocument/didChange #1',
      'textDocument/completion #1',
      'textDocument/signatureHelp #1',
    ]);
  });

  it('gives std a document needs before opening it, fetching each module once', async () => {
    const { LspDoc } = await load();
    const a = new LspDoc(() => 'use std.graph\nuse std.math');
    const b = new LspDoc(() => 'use std.math');
    await Promise.all([a.request('textDocument/hover', {}), b.request('textDocument/hover', {})]);
    expect(fetched.sort()).toEqual(['graph.cp', 'math.cp']);
    const log = workers[0].log;
    expect(log.indexOf('provideStd graph')).toBeLessThan(log.indexOf(`textDocument/didOpen ${short(a.uri)}`));
  });

  it('reopens every open document, with its std, on the worker that replaces a stopped one', async () => {
    const { LspDoc, cpl } = await load();
    const a = new LspDoc(() => 'use std.sparse');
    const b = new LspDoc(() => 'let b = 2');
    const c = new LspDoc(() => 'let c = 3');
    for (const d of [a, b, c]) await d.request('textDocument/hover', {});
    workers[0].terminate();
    await expect(b.request('textDocument/hover', {})).rejects.toMatchObject({ kind: 'terminated' });
    c.close();
    expect(cpl.compilerState()).toBe('idle');
    expect(await b.request('textDocument/hover', {})).toBe('let b = 2');
    expect(workers).toHaveLength(2);
    const [ia, ib] = [a, b].map((d) => short(d.uri));
    expect(workers[1].log).toEqual([
      'initialize',
      'initialized',
      'provideStd sparse',
      `textDocument/didOpen ${ia}`,
      `textDocument/didOpen ${ib}`,
      `textDocument/hover ${ib}`,
    ]);
    expect(fetched).toEqual(['sparse.cp']);
    expect(await a.request('textDocument/hover', {})).toBe('use std.sparse');
  });

  it('closes a document on the server and not again', async () => {
    const { LspDoc } = await load();
    const doc = new LspDoc(() => 'let x = 1');
    await doc.request('textDocument/hover', {});
    doc.close();
    doc.close();
    await LspDoc.request('workspace/symbol', {});
    expect(workers[0].log.filter((l) => l.startsWith('textDocument/didClose'))).toHaveLength(1);
  });

  it('sends a request about no document after initializing the server', async () => {
    const { LspDoc } = await load();
    expect(await LspDoc.request('completionItem/resolve', { label: 'x' })).toBe('completionItem/resolve answered');
    expect(workers[0].log).toEqual(['initialize', 'initialized', 'completionItem/resolve']);
  });

  // Read from the editor's source, so a method added there is covered here at once.
  it('sends every method the editor asks for after the text it was asked about', async () => {
    const dir = join(__dirname, '../components/editor');
    const src = readdirSync(dir)
      .filter((f) => /\.tsx?$/.test(f) && !f.includes('.test.'))
      .map((f) => readFileSync(join(dir, f), 'utf8'))
      .join('\n');
    const named = (re: RegExp) => new Set([...src.matchAll(re)].map((m) => m[1]));
    const all = named(/"((?:textDocument|workspace|completionItem|callHierarchy)\/[\w/]+)"/g);
    const static_ = named(/LspDoc\.request(?:<[^>]*>)?\(\s*"([\w/]+)"/g);
    expect(all.size).toBeGreaterThan(20);
    const { LspDoc } = await load();
    let text = '';
    const doc = new LspDoc(() => text);
    const n = doc.uri.match(/\d+/)![0];
    let sync = 'textDocument/didOpen';
    for (const [i, method] of [...all].entries()) {
      if (static_.has(method)) {
        expect(await LspDoc.request(method, {})).toBe(`${method} answered`);
        continue;
      }
      text = `let x = ${i}`;
      expect(await doc.request(method, {})).toBe(text);
      expect(workers[0].log.slice(-2)).toEqual([`${sync} #${n}`, `${method} #${n}`]);
      sync = 'textDocument/didChange';
    }
    expect([...static_]).toEqual(['completionItem/resolve']);
  });
});
