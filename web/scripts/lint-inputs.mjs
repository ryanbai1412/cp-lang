// Gives every lint example that reads input one to run on: a small random
// case from its `read()` types, made by cpl-build.wasm the way `cpl stress`
// makes one. lib/lints.ts reads content/generated/lint-inputs.json.
import { execFileSync } from 'node:child_process';
import { mkdirSync, writeFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

const web = dirname(dirname(fileURLToPath(import.meta.url)));
const { load, BUILD_WASM } = await import(pathToFileURL(join(web, 'public/wasm/cpl.js')));
const { giveStd } = await import('./give-std.mjs');
const { cplBin } = await import('./cpl-bin.mjs');
const cpl = giveStd(await load(BUILD_WASM));
export const reads = (src) => /\bread\w*\s*(\(|<|::)/.test(src);

const inputs = {};
let made = 0;
let none = 0;
// The examples come from the compiler's registry, `cpl lint --list --json`.
const listed = JSON.parse(execFileSync(cplBin(), ['lint', '--list', '--json'], { encoding: 'utf8', maxBuffer: 64 << 20 }));
for (const lint of listed.lints) {
  for (const [key, file] of [['before', 'before.cp'], ['after', 'after.cp'], ['near_miss', 'near_miss.cp']]) {
    const src = lint[key];
    if (!src || !reads(src)) continue;
    const r = cpl.sample(src, { seed: 1, size: 5 });
    if ('input' in r) (inputs[`${lint.dir}/${file}`] = r.input), made++;
    else none++;
  }
}
const out = join(web, 'content/generated/lint-inputs.json');
mkdirSync(dirname(out), { recursive: true });
writeFileSync(out, JSON.stringify(inputs, null, 1) + '\n');
console.log(`lint-inputs: ${made} examples get a random input, ${none} read input their types can't generate`);
