'use client';

import { lspTask, type LspSend } from './cpl';

type Reply = { id?: number; result?: unknown };

// The language server behind `cpl lsp`, run in the compiler's worker. Every
// editor on a page is one document on it. `open` is each editor's text as
// last sent; `known` is what the current server has, which is nothing on a
// new one (a worker stopped after a run timed out comes back empty).
const open = new Map<string, string>();
const known = new Map<string, string>();
let ids = 1;
let names = 0;
let versions = 1;
let advertised: Capabilities = {};

/** What the server's `initialize` reply advertises. */
export type Capabilities = Record<string, unknown>;

const notify = (send: LspSend, method: string, params: object) => send({ jsonrpc: '2.0', method, params });

// Every message goes through here: the server is initialized if it is new,
// then given each document's latest text (and told of closed ones) before
// `task` runs, all in one step on the language server's queue.
function session<T>(task: (send: LspSend) => Promise<T>): Promise<T> {
  return lspTask(async (send, fresh) => {
    if (fresh) {
      known.clear();
      const replies = (await send({ jsonrpc: '2.0', id: 0, method: 'initialize', params: {} })) as Reply[];
      const init = replies.find((r) => r.id === 0)?.result as { capabilities?: Capabilities } | undefined;
      advertised = init?.capabilities ?? {};
      await notify(send, 'initialized', {});
    }
    for (const uri of known.keys())
      if (!open.has(uri)) {
        known.delete(uri);
        await notify(send, 'textDocument/didClose', { textDocument: { uri } });
      }
    for (const [uri, text] of open) {
      const had = known.get(uri);
      if (had === text) continue;
      if (had === undefined) await notify(send, 'textDocument/didOpen', { textDocument: { uri, languageId: 'cp', version: 1, text } });
      else await notify(send, 'textDocument/didChange', { textDocument: { uri, version: ++versions }, contentChanges: [{ text }] });
      known.set(uri, text);
    }
    return task(send);
  });
}

async function ask<T>(send: LspSend, method: string, params: object): Promise<T | null> {
  const id = ids++;
  const replies = (await send({ jsonrpc: '2.0', id, method, params })) as Reply[];
  return (replies.find((r) => r.id === id)?.result ?? null) as T | null;
}

/** The server's capabilities, once it is running. */
export const capabilities = (): Promise<Capabilities> => session(async () => advertised);

export class LspDoc {
  readonly uri = `file:///editor-${++names}.cp`;

  constructor(private readonly text: () => string) {}

  /** Sends the editor's current text if the server doesn't have it, then the request; its result or null. */
  request<T>(method: string, params: object): Promise<T | null> {
    open.set(this.uri, this.text());
    return session((send) => ask<T>(send, method, { textDocument: { uri: this.uri }, ...params }));
  }

  /** A request about no document, such as `completionItem/resolve`. */
  static request<T>(method: string, params: object): Promise<T | null> {
    return session((send) => ask<T>(send, method, params));
  }

  close() {
    if (open.delete(this.uri)) session(async () => {}).catch(() => {});
  }
}
