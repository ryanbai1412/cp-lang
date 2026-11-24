import { afterEach, describe, expect, it, vi } from 'vitest';
import { documentText, provideStd } from './cpl';

const texts: Record<string, string> = { sparse: 'sparse text', math: 'math text' };

function worker(needs: string[][]) {
  const given: string[] = [];
  return {
    given,
    stdNeeded: vi.fn(async () => needs.shift() ?? []),
    provideStd: vi.fn(async (m: string) => {
      given.push(m);
      return { changed: true };
    }),
  };
}

afterEach(() => vi.unstubAllGlobals());

describe('lazy std', () => {
  it('fetches what the check needs until nothing is missing, once per module', async () => {
    vi.stubGlobal('location', { origin: 'https://cp-lang.org' });
    const fetch = vi.fn(async (url: string) => {
      const m = url.match(/std\/(\w+)\.cp$/)![1];
      return { ok: true, text: async () => texts[m] };
    });
    vi.stubGlobal('fetch', fetch);
    const w = worker([['sparse'], ['math', 'sparse']]);
    await provideStd(w as never, 'use std.sparse');
    expect(w.given.sort()).toEqual(['math', 'sparse', 'sparse']);
    expect(w.stdNeeded).toHaveBeenCalledTimes(3);
    expect(fetch.mock.calls.map((c) => c[0])).toEqual(['https://cp-lang.org/wasm/std/sparse.cp', 'https://cp-lang.org/wasm/std/math.cp']);
  });

  it('reports a module that would not download as a load error, and retries it next time', async () => {
    vi.stubGlobal('location', { origin: 'https://cp-lang.org' });
    vi.stubGlobal('fetch', vi.fn(async () => ({ ok: false, status: 503 })));
    await expect(provideStd(worker([['graph']]) as never, 'x')).rejects.toMatchObject({
      kind: 'load',
      message: "cpl: can't load std.graph: HTTP 503; check the connection and try again",
    });
    vi.stubGlobal('fetch', vi.fn(async () => ({ ok: true, text: async () => 'g' })));
    const w = worker([['graph']]);
    await provideStd(w as never, 'x');
    expect(w.given).toEqual(['graph']);
  });

  it('does nothing on a build that embeds std', async () => {
    await provideStd({} as never, 'use std.sparse');
  });

  it('reads the text an LSP notification opens or changes', () => {
    expect(documentText({ params: { textDocument: { uri: 'a', text: 'x' } } })).toBe('x');
    expect(documentText({ params: { textDocument: { uri: 'a' }, contentChanges: [{ text: 'y' }] } })).toBe('y');
    expect(documentText({ params: { position: {} } })).toBeUndefined();
  });
});
