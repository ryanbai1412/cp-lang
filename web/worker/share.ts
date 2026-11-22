export interface Bucket {
  get(key: string): Promise<{ text(): Promise<string> } | null>;
  head(key: string): Promise<unknown | null>;
  put(key: string, value: string, options?: { httpMetadata?: { contentType?: string } }): Promise<unknown>;
}

export interface Snapshot {
  v: 1;
  src: string;
  in: string;
  version: string;
}

export const MAX_BYTES = 64 * 1024;
const alphabet = '0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz';
export const hashPattern = /^[0-9A-Za-z]{10}$/;

export async function shareId(body: string): Promise<string> {
  const digest = new Uint8Array(await crypto.subtle.digest('SHA-256', new TextEncoder().encode(body)));
  let n = 0n;
  for (const b of digest.slice(0, 8)) n = (n << 8n) | BigInt(b);
  let id = '';
  for (let i = 0; i < 10; i++) {
    id = alphabet[Number(n % 62n)] + id;
    n /= 62n;
  }
  return id;
}

export function parseSnapshot(raw: unknown): Snapshot | null {
  if (typeof raw !== 'object' || raw === null) return null;
  const r = raw as Record<string, unknown>;
  if (typeof r.src !== 'string' || typeof (r.in ?? '') !== 'string' || typeof r.version !== 'string') return null;
  return { v: 1, src: r.src, in: (r.in as string | undefined) ?? '', version: r.version };
}

const json = (data: unknown, status = 200) =>
  new Response(JSON.stringify(data), { status, headers: { 'Content-Type': 'application/json' } });

export async function create(request: Request, bucket: Bucket): Promise<Response> {
  const text = await request.text();
  if (new TextEncoder().encode(text).length > MAX_BYTES) return json({ error: 'program is over 64 KB' }, 413);
  let raw: unknown;
  try {
    raw = JSON.parse(text);
  } catch {
    return json({ error: 'body is not JSON' }, 400);
  }
  const snapshot = parseSnapshot(raw);
  if (!snapshot) return json({ error: 'expected {src, in, version}' }, 400);
  const body = JSON.stringify(snapshot);
  const id = await shareId(body);
  if (!(await bucket.head(id))) await bucket.put(id, body, { httpMetadata: { contentType: 'application/json' } });
  return json({ id, url: `/p/${id}` }, 201);
}

export async function read(id: string, bucket: Bucket): Promise<Snapshot | null> {
  if (!hashPattern.test(id)) return null;
  const object = await bucket.get(id);
  return object ? parseSnapshot(JSON.parse(await object.text())) : null;
}
