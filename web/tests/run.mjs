// Serves the built site (web/out) on a free port and runs the browser smoke
// tests against it (or the tests named as arguments). BASE points them at a
// server that is already running.
import { spawn } from 'node:child_process';
import { existsSync, readFileSync, statSync } from 'node:fs';
import { createServer } from 'node:http';
import { dirname, extname, join, normalize } from 'node:path';
import { fileURLToPath } from 'node:url';

const tests = dirname(fileURLToPath(import.meta.url));
const out = join(dirname(tests), 'out');
const types = {
  '.html': 'text/html; charset=utf-8',
  '.js': 'text/javascript',
  '.css': 'text/css',
  '.json': 'application/json',
  '.wasm': 'application/wasm',
  '.svg': 'image/svg+xml',
  '.txt': 'text/plain; charset=utf-8',
};

function resolve(url) {
  const path = join(out, normalize(decodeURIComponent(new URL(url, 'http://localhost').pathname)));
  if (!path.startsWith(out)) return null;
  for (const p of [path, `${path}.html`, join(path, 'index.html')]) if (existsSync(p) && statSync(p).isFile()) return p;
  return null;
}

let server;
let base = process.env.BASE;
if (!base) {
  if (!existsSync(join(out, 'index.html'))) {
    console.error('smoke: web/out is missing; run `pnpm --dir web build` first');
    process.exit(1);
  }
  server = createServer((req, res) => {
    const file = resolve(req.url);
    res.writeHead(file ? 200 : 404, { 'content-type': types[extname(file ?? '.html')] ?? 'application/octet-stream' });
    res.end(readFileSync(file ?? join(out, '404.html')));
  });
  await new Promise((done) => server.listen(0, '127.0.0.1', done));
  base = `http://127.0.0.1:${server.address().port}`;
}

let status = 0;
const named = process.argv.slice(2);
for (const test of named.length ? named : ['smoke.mjs', 'learn-smoke.mjs']) {
  // Not spawnSync: this process is also the server the test loads pages from.
  const child = spawn(process.execPath, [join(tests, test)], { stdio: 'inherit', env: { ...process.env, BASE: base } });
  if ((await new Promise((done) => child.on('exit', done))) !== 0) status = 1;
}
server?.close();
process.exit(status);
