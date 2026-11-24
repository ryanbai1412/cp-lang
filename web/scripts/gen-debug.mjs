// The landing page's debugging evidence, from today's compiler: `cpl stress`
// on a planted bug, and the side-by-side trace it saves for the shrunk case.
import { execFileSync, spawnSync } from 'node:child_process';
import { copyFileSync, cpSync, existsSync, mkdirSync, mkdtempSync, readFileSync, readdirSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { cplBin } from './cpl-bin.mjs';

const web = dirname(dirname(fileURLToPath(import.meta.url)));
const repo = dirname(web);
const cpl = cplBin();
if (!existsSync(cpl)) execFileSync('cargo', ['build', '--release', '-q', '-p', 'cpl-cli'], { cwd: repo, stdio: 'inherit' });

const dir = mkdtempSync(join(tmpdir(), 'cp-debug-'));
copyFileSync(join(web, 'content/landing/debug/buy.cp'), join(dir, 'buy.cp'));
const args = ['stress', 'buy.cp', '--cases', '200', '--seed', '1', '--trace', '--explain'];
const r = spawnSync(cpl, args, { cwd: dir, encoding: 'utf8', timeout: 120000 });
const output = (r.stdout + r.stderr).replace(/\x1b\[[0-9;]*m/g, '');
const trace = readdirSync(join(dir, 'buy.tests')).find((f) => f.endsWith('.trace.html'));
if (!/FAIL/.test(output) || !trace) throw new Error(`gen-debug: cpl stress found no failing case:\n${output}`);

mkdirSync(join(web, 'content/generated'), { recursive: true });
writeFileSync(
  join(web, 'content/generated/stress.json'),
  JSON.stringify({ command: `cpl ${args.filter((a) => a !== '--seed' && a !== '1').join(' ')}`, output }, null, 2),
);
cpSync(join(dir, 'buy.tests', trace), join(web, 'public/trace-demo.html'));
// The viewer opens in its own tab, so it loads the compiler itself to replay a call.
const compiler = `<script>
let cplWorker;
globalThis.CpCompiler = {
  call: async (method, ...args) => {
    const cpl = await (cplWorker ??= import("/wasm/cpl.js").then((m) => m.spawn({ timeoutMs: 10000 })));
    if (typeof args[0] === "string")
      for (let need = await cpl.stdNeeded(args[0]); need.length; need = await cpl.stdNeeded(args[0]))
        for (const m of need) await cpl.provideStd(m, await (await fetch("/wasm/std/" + m + ".cp")).text());
    return cpl[method](...args);
  },
};
</script>`;
const viewer = readFileSync(join(repo, 'tools/trace/index.html'), 'utf8');
if (!viewer.includes('</head>')) throw new Error('gen-debug: tools/trace/index.html has no </head>');
// On the site the viewer shows the site's nav and its brand goes home; exported traces reset to standalone.
const nav = /<nav hidden data-site-nav([^>]*)>[\s\S]*?<\/nav>/;
const siteNav = `<nav data-site-nav$1>
    <a href="/">Home</a><a href="/learn">Learn</a><a href="/docs">Docs</a><a href="/play">Playground</a><a href="/trace.html" aria-current="page">Trace</a>
  </nav>`;
const brand = '<a class="brand" href="#main">';
const context = '<html lang="en" data-context="standalone">';
for (const [what, found] of [['site nav', nav.test(viewer)], ['brand link', viewer.includes(brand)], ['standalone context', viewer.includes(context)]])
  if (!found) throw new Error(`gen-debug: tools/trace/index.html has no ${what}`);
const site = viewer
  .replace(context, '<html lang="en" data-context="site">')
  .replace(brand, '<a class="brand" href="/">')
  .replace(nav, siteNav)
  .replace('</head>', `${compiler}\n</head>`);
writeFileSync(join(web, 'public/trace.html'), site);
console.log(`debug: stress output and ${trace}`);
