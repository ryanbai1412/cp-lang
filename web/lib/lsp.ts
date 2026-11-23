'use client';

import { compiler, cpl } from './cpl';

type Reply = { id?: number; result?: unknown };

// The language server behind `cpl lsp`, run in the compiler's worker. Every
// editor on a page is one document on it.
const open = new Map<string, string>();
let session: { worker: unknown; ready: Promise<void> } | null = null;
let ids = 1;
let names = 0;

const notify = (method: string, params: object) => cpl.lsp({ jsonrpc: '2.0', method, params });
const didOpen = (uri: string, text: string) => notify('textDocument/didOpen', { textDocument: { uri, languageId: 'cp', version: 1, text } });

// A worker that was stopped (a run that timed out) comes back with an empty
// server, so the first message to a new one initializes it and reopens every
// document.
async function ready() {
  const worker = compiler();
  if (session?.worker !== worker) {
    const start = async () => {
      await cpl.lsp({ jsonrpc: '2.0', id: 0, method: 'initialize', params: {} });
      await notify('initialized', {});
      for (const [uri, text] of open) await didOpen(uri, text);
    };
    session = { worker, ready: start() };
  }
  try {
    await session.ready;
  } catch (error) {
    session = null;
    throw error;
  }
}

export class LspDoc {
  readonly uri = `file:///editor-${++names}.cp`;
  private version = 1;

  constructor(private readonly text: () => string) {}

  /** Sends the editor's current text if it changed, then the request; its result or null. */
  async request<T>(method: string, params: object): Promise<T | null> {
    await ready();
    const text = this.text();
    if (!open.has(this.uri)) {
      open.set(this.uri, text);
      await didOpen(this.uri, text);
    } else if (open.get(this.uri) !== text) {
      open.set(this.uri, text);
      await notify('textDocument/didChange', { textDocument: { uri: this.uri, version: ++this.version }, contentChanges: [{ text }] });
    }
    const id = ids++;
    const replies = (await cpl.lsp({ jsonrpc: '2.0', id, method, params: { textDocument: { uri: this.uri }, ...params } })) as Reply[];
    return (replies.find((r) => r.id === id)?.result ?? null) as T | null;
  }

  close() {
    if (open.delete(this.uri) && session) notify('textDocument/didClose', { textDocument: { uri: this.uri } }).catch(() => {});
  }
}
