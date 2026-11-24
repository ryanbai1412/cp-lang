// Serves the built site (web/out) on a free port and runs the browser smoke
// tests against it (or the tests named as arguments), all in one shared
// chromium. BASE points them at a server that is already running.
//
// Each suite ends with browser.mjs `done()`, which prints its verdict line. A
// suite fails if it runs past SUITE_SECONDS (default 150), exits without that
// line, or is still running EXIT_GRACE_MS after it (a hang after the result
// once hid a real red behind CI's job limit).
import { chromium } from '@playwright/test';
import { spawn } from 'node:child_process';
import { existsSync, readFileSync, statSync } from 'node:fs';
import { createServer } from 'node:http';
import { dirname, extname, join, normalize } from 'node:path';
import { fileURLToPath } from 'node:url';
import { RESULT, until } from './browser.mjs';

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
const limit = Number(process.env.SUITE_SECONDS ?? 150);
const grace = Number(process.env.EXIT_GRACE_MS ?? 5000);

function resolve(url) {
  const path = join(out, normalize(decodeURIComponent(new URL(url, 'http://localhost').pathname)));
  if (!path.startsWith(out)) return null;
  for (const p of [path, `${path}.html`, join(path, 'index.html')]) if (existsSync(p) && statSync(p).isFile()) return p;
  return null;
}

const named = process.argv.slice(2).map((t) => (t.endsWith('.mjs') ? t : `${t}.mjs`));
const suites = named.length ? named : ['smoke.mjs', 'learn-smoke.mjs', 'intelligence.mjs'];
for (const s of suites) {
  if (!existsSync(join(tests, s))) {
    console.error(`run: no suite tests/${s}`);
    process.exit(2);
  }
  if (!/\bdone\(/.test(readFileSync(join(tests, s), 'utf8'))) {
    console.error(`run: tests/${s} never calls done() from browser.mjs, so its verdict can't be told from a hang`);
    process.exit(2);
  }
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
  await new Promise((ok) => server.listen(0, '127.0.0.1', ok));
  base = `http://127.0.0.1:${server.address().port}`;
}
const browser = process.env.CDP ? null : await chromium.launchServer();

// Not spawnSync: this process is also the server the test loads pages from.
function runSuite(test) {
  const start = Date.now();
  const env = { ...process.env, BASE: base, ...(browser && { PW_WS: browser.wsEndpoint() }) };
  const child = spawn(process.execPath, [join(tests, test)], {
    stdio: ['ignore', 'pipe', 'inherit'],
    env,
    timeout: limit * 1000,
    killSignal: 'SIGKILL',
  });
  return new Promise((ok) => {
    let exited = false;
    let verdict = null;
    let why = null;
    let line = '';
    const kill = (reason) => {
      why ??= reason;
      child.kill('SIGKILL');
    };
    child.stdout.on('data', (chunk) => {
      process.stdout.write(chunk);
      line += chunk;
      const lines = line.split('\n');
      line = lines.pop();
      for (const l of lines)
        if (l.startsWith(RESULT) && verdict === null) {
          verdict = l.slice(RESULT.length).trim();
          until(() => exited, grace).then((gone) => gone || kill(`printed its result but was still running ${grace} ms later`));
        }
    });
    child.on('exit', (code, signal) => {
      exited = true;
      if (signal === 'SIGKILL') why ??= `still running after ${limit} s (SUITE_SECONDS)`;
      const took = ((Date.now() - start) / 1000).toFixed(1);
      why ??= verdict === null ? `exited (${signal ?? code}) without a result line` : null;
      why ??= verdict !== 'pass' || code !== 0 ? `failed (${signal ?? code})` : null;
      console.log(`run: ${test} ${why ? `FAILED: ${why}` : 'passed'} in ${took} s`);
      ok(!why);
    });
  });
}

let status = 0;
for (const test of suites) if (!(await runSuite(test))) status = 1;
await browser?.close();
server?.close();
process.exit(status);
