// Runs the prebuild stages as two independent chains at once (the wasm build
// shares nothing with the generated docs and the snippet gate), skipping each
// stage whose inputs and outputs are unchanged since its last successful run
// (prebuild/cache.mjs). Every stage declares what it reads; a run that reads
// a repo file outside its declaration fails, because a cache keyed on the
// declaration would serve stale output after that file changes.
// Then the timed stages run alone (stages.mjs `alone`). PREBUILD_CACHE=0
// runs every stage.
import { spawn } from 'node:child_process';
import { existsSync, readFileSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { pathToFileURL } from 'node:url';
import { hashFiles, record, recorded, saveMemo, stageKey, undeclared, web } from './prebuild/cache.mjs';
import { alone, chains, stages } from './prebuild/stages.mjs';

// A stage that hangs fails by name instead of running into CI's job timeout
// with no log; each stage's time is printed so slow ones show.
const limit = Number(process.env.PREBUILD_STAGE_SECONDS ?? 240);
const useCache = process.env.PREBUILD_CACHE !== '0';
const tracer = pathToFileURL(join(web, 'scripts/prebuild/trace-reads.mjs')).href;
const only = process.argv.slice(2);

const run = (stage) =>
  new Promise((resolve, reject) => {
    const { name } = stage;
    const key = stage.always ? null : stageKey(stage);
    const last = recorded(name);
    if (useCache && key && last?.key === key && last.outputs === hashFiles(stage.outputs)) {
      console.log(`prebuild: ${name} cached`);
      return resolve();
    }
    const start = Date.now();
    const traceFile = join(tmpdir(), `prebuild-trace-${process.pid}-${name}`);
    rmSync(traceFile, { force: true });
    const env = { ...process.env, PREBUILD_TRACE: traceFile, NODE_OPTIONS: `${process.env.NODE_OPTIONS ?? ''} --import=${tracer}`.trim() };
    const p = spawn(process.execPath, [`scripts/${name}.mjs`], { stdio: 'inherit', detached: true, env, cwd: web });
    const timer = setTimeout(() => {
      console.error(`prebuild: ${name} ran over ${limit} s; killing it`);
      process.kill(-p.pid, 'SIGKILL');
    }, limit * 1000);
    p.on('error', reject);
    p.on('exit', (code, signal) => {
      clearTimeout(timer);
      const took = ((Date.now() - start) / 1000).toFixed(1);
      console.log(`prebuild: ${name} ${took} s`);
      if (code !== 0) return reject(new Error(`prebuild: ${name} failed (${signal ?? code}) after ${took} s`));
      const bad = existsSync(traceFile) ? undeclared(stage, readFileSync(traceFile, 'utf8').split('\n').filter(Boolean)) : [];
      rmSync(traceFile, { force: true });
      if (bad.length)
        return reject(new Error(`prebuild: ${name} read ${bad.join(', ')}, which scripts/prebuild/stages.mjs does not list among its inputs`));
      if (key) record(name, { key: stageKey(stage), outputs: hashFiles(stage.outputs) });
      resolve();
    });
  });

const byName = new Map(stages.map((s) => [s.name, s]));
const results = await Promise.allSettled(
  chains.map(async (steps) => {
    for (const step of steps) if (!only.length || only.includes(step)) await run(byName.get(step));
  }),
);
// After the chains, even when one failed: a red docs stage must not hide
// whether the browser keeps stress parity.
for (const step of alone)
  if (!only.length || only.includes(step))
    await run(byName.get(step)).catch((e) => results.push({ status: 'rejected', reason: e }));
saveMemo();
const failed = results.filter((r) => r.status === 'rejected');
for (const f of failed) console.error(f.reason.message);
process.exit(failed.length ? 1 : 0);
