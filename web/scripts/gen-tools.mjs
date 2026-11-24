import { execFileSync } from 'node:child_process';
import { mkdirSync, writeFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { cplBin } from './cpl-bin.mjs';

const repo = dirname(dirname(dirname(fileURLToPath(import.meta.url))));
const cpl = cplBin();
const run = (...args) => execFileSync(cpl, args, { encoding: 'utf8' });

// The CLI's own contracts (spec/tools/cli.md): `--help --json` and `judges --json`.
const { commands } = JSON.parse(run('--help', '--json'));
const missing = commands.filter((c) => !c.help).map((c) => c.name);
if (missing.length) console.warn(`gen-tools: no --help for ${missing.join(', ')}`);

const judges = JSON.parse(run('judges', '--json')).judges.map((j) => ({
  name: j.name,
  default: j.chosen,
  compiler: j.title,
  details: [
    j.command,
    `${j.os === 'windows' ? 'Windows' : 'Linux'}, ${j.stack}, ${
      j.thread_stack_mb === null ? 'no threads' : `the program on a thread with ${j.thread_stack_mb} MB`
    }, source up to ${j.source_limit_kb} KB`,
    `usual limits ${j.time_limit_ms} ms, ${j.memory_limit_mb} MB`,
    `from: ${j.evidence}`,
  ],
}));

const version = run('--version').trim();
mkdirSync('content/generated', { recursive: true });
writeFileSync('content/generated/cli.json', `${JSON.stringify({ version, commands, judges }, null, 2)}\n`);
console.log(`gen-tools: ${commands.length} commands, ${judges.length} judges (${version})`);
