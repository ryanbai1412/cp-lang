// One browser-build stress run for web/scripts/stress-parity.mjs, in its own
// process so a slow one can be stopped: `node stress-worker.mjs <file> <json options> <wasm dir>`.
import { readFileSync } from 'node:fs';
import { basename, join } from 'node:path';

const [file, options, dir] = process.argv.slice(2);
const { load } = await import(join(dir, 'cpl.js'));
const wasm = await load(readFileSync(join(dir, 'cpl-build.wasm')));
process.stdout.write(JSON.stringify(wasm.stress(readFileSync(file, 'utf8'), { ...JSON.parse(options), name: basename(file) })));
