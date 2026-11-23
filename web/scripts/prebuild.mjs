// Runs the prebuild steps as two independent chains at once: the wasm build
// shares nothing with the generated docs and the snippet gate.
import { spawn } from 'node:child_process';

const chains = [
  ['wasm', 'wasm-size', 'lint-inputs'],
  ['stamp', 'releases', 'practice', 'downloads', 'gen-ref', 'gen-tools', 'gen-debug', 'facts', 'gate'],
];

const run = (name) =>
  new Promise((resolve, reject) => {
    const p = spawn(process.execPath, [`scripts/${name}.mjs`], { stdio: 'inherit' });
    p.on('error', reject);
    p.on('exit', (code, signal) =>
      code === 0 ? resolve() : reject(new Error(`prebuild: ${name} failed (${signal ?? code})`)),
    );
  });

const results = await Promise.allSettled(
  chains.map(async (steps) => {
    for (const step of steps) await run(step);
  }),
);
const failed = results.filter((r) => r.status === 'rejected');
for (const f of failed) console.error(f.reason.message);
process.exit(failed.length ? 1 : 0);
