import { type Bucket, create, read } from './share';

interface Env {
  SHARES: Bucket;
  ASSETS: { fetch(request: Request | string): Promise<Response> };
  SHARE_LIMIT: { limit(options: { key: string }): Promise<{ success: boolean }> };
}

const escape = (s: string) => s.replace(/[&<>"]/g, (c) => `&#${c.charCodeAt(0)};`);

export default {
  async fetch(request: Request, env: Env): Promise<Response> {
    const url = new URL(request.url);
    if (url.pathname === '/api/share' && request.method === 'POST') {
      const { success } = await env.SHARE_LIMIT.limit({ key: request.headers.get('cf-connecting-ip') ?? '' });
      if (!success) return new Response('Too many shares; try again in a minute.', { status: 429 });
      return create(request, env.SHARES);
    }
    const api = /^\/api\/p\/([^/]+)$/.exec(url.pathname);
    if (api) {
      const snapshot = await read(api[1], env.SHARES);
      return snapshot
        ? new Response(JSON.stringify(snapshot), {
            headers: { 'Content-Type': 'application/json', 'Cache-Control': 'public, max-age=31536000, immutable' },
          })
        : new Response('not found', { status: 404 });
    }
    const page = /^\/p\/([^/]+)\/?$/.exec(url.pathname);
    if (page) {
      const snapshot = await read(page[1], env.SHARES);
      if (!snapshot) return new Response('This share link does not exist.', { status: 404 });
      const html = await (await env.ASSETS.fetch(new URL('/play/', url).toString())).text();
      const preview = snapshot.src.split('\n').slice(0, 6).join(' / ');
      const meta = `<meta property="og:title" content="A CP Lang program"><meta property="og:description" content="${escape(preview)}">`;
      return new Response(html.replace('</head>', `${meta}</head>`), {
        headers: { 'Content-Type': 'text/html; charset=utf-8' },
      });
    }
    return env.ASSETS.fetch(request);
  },
};
