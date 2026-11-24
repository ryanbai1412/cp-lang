// A worker thread for web/scripts/gate.mjs's browser differential: loads the
// browser build once and answers { source, stdin } with its C++ and its run.
import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { parentPort, workerData } from 'node:worker_threads';

const { giveStd } = await import('./give-std.mjs');
const { load } = await import(join(workerData.dir, 'cpl.js'));
const wasm = giveStd(await load(readFileSync(join(workerData.dir, 'cpl-build.wasm'))));
parentPort.on('message', ({ source, stdin, run }) => {
  try {
    const cpp = wasm.build(source, { name: 'main.cp' }).cpp;
    const r = run ? wasm.run(source, stdin, { name: 'main.cp' }) : null;
    parentPort.postMessage({ cpp, stdout: r?.stdout ?? null, stderr: r?.stderr ?? '', ok: r ? r.exitCode === 0 : null });
  } catch (e) {
    parentPort.postMessage({ error: `${e.kind ?? 'error'}: ${e.message}` });
  }
});
