// The landing page's debugging evidence, from today's compiler: `cpl stress`
// on a planted bug, and the side-by-side trace it saves for the shrunk case.
import { execFileSync, spawnSync } from 'node:child_process';
import { copyFileSync, cpSync, existsSync, mkdirSync, mkdtempSync, readdirSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const web = dirname(dirname(fileURLToPath(import.meta.url)));
const repo = dirname(web);
const cpl = process.env.CPL ?? join(repo, 'target/release/cpl');
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
copyFileSync(join(repo, 'tools/trace/index.html'), join(web, 'public/trace.html'));
console.log(`debug: stress output and ${trace}`);
