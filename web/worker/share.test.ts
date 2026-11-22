import { describe, expect, it } from 'vitest';
import { type Bucket, create, MAX_BYTES, read } from './share';

function memory(): Bucket & { size(): number } {
  const store = new Map<string, string>();
  return {
    async get(k) {
      const v = store.get(k);
      return v === undefined ? null : { text: async () => v };
    },
    async head(k) {
      return store.has(k) ? {} : null;
    },
    async put(k, v) {
      store.set(k, v);
      return {};
    },
    size: () => store.size,
  };
}

const post = (body: string) => new Request('https://example.com/api/share', { method: 'POST', body });

describe('share links', () => {
  it('stores a snapshot under a stable 10-character id and reads it back', async () => {
    const bucket = memory();
    const body = JSON.stringify({ src: 'print(1)\n', in: '', version: '0.2.2' });
    const first = await (await create(post(body), bucket)).json();
    const second = await (await create(post(body), bucket)).json();
    expect(first.id).toMatch(/^[0-9A-Za-z]{10}$/);
    expect(second.id).toBe(first.id);
    expect(bucket.size()).toBe(1);
    expect(await read(first.id, bucket)).toEqual({ v: 1, src: 'print(1)\n', in: '', version: '0.2.2' });
  });

  it('rejects oversized, malformed and unknown snapshots', async () => {
    const bucket = memory();
    const big = JSON.stringify({ src: 'x'.repeat(MAX_BYTES), in: '', version: '0.2.2' });
    expect((await create(post(big), bucket)).status).toBe(413);
    expect((await create(post('nope'), bucket)).status).toBe(400);
    expect((await create(post('{"src":1}'), bucket)).status).toBe(400);
    expect(await read('../etc', bucket)).toBeNull();
    expect(await read('AAAAAAAAAA', bucket)).toBeNull();
  });
});
