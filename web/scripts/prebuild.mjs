// Runs the prebuild steps as two independent chains at once: the wasm build
// shares nothing with the generated docs and the snippet gate.
import { spawn } from 'node:child_process';

const chains = [
  ['wasm', 'wasm-size', 'lint-inputs', 'practice-mutants'],
  ['stamp', 'releases', 'practice', 'downloads', 'gen-ref', 'gen-tools', 'gen-debug', 'facts', 'gate'],
];

// A stage that hangs fails by name instead of running into CI's job timeout
// with no log; each stage's time is printed so slow ones show.
const limit = Number(process.env.PREBUILD_STAGE_SECONDS ?? 240);
const run = (name) =>
  new Promise((resolve, reject) => {
    const start = Date.now();
    const p = spawn(process.execPath, [`scripts/${name}.mjs`], { stdio: 'inherit', detached: true });
    const timer = setTimeout(() => {
      console.error(`prebuild: ${name} ran over ${limit} s; killing it`);
      process.kill(-p.pid, 'SIGKILL');
    }, limit * 1000);
    p.on('error', reject);
    p.on('exit', (code, signal) => {
      clearTimeout(timer);
      const took = ((Date.now() - start) / 1000).toFixed(1);
      console.log(`prebuild: ${name} ${took} s`);
      code === 0 ? resolve() : reject(new Error(`prebuild: ${name} failed (${signal ?? code}) after ${took} s`));
    });
  });

const results = await Promise.allSettled(
  chains.map(async (steps) => {
    for (const step of steps) await run(step);
  }),
);
const failed = results.filter((r) => r.status === 'rejected');
for (const f of failed) console.error(f.reason.message);
process.exit(failed.length ? 1 : 0);
