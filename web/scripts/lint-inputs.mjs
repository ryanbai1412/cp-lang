// Gives every lint example that reads input one to run on: a small random
// case from its `read()` types, made by cpl-build.wasm the way `cpl stress`
// makes one. lib/lints.ts reads content/generated/lint-inputs.json.
import { existsSync, mkdirSync, readdirSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

const web = dirname(dirname(fileURLToPath(import.meta.url)));
const repo = dirname(web);
const { load, BUILD_WASM } = await import(pathToFileURL(join(web, 'public/wasm/cpl.js')));
const cpl = await load(BUILD_WASM);
export const reads = (src) => /\bread\w*\s*(\(|<|::)/.test(src);

const inputs = {};
let made = 0;
let none = 0;
for (const root of ['spec/lints', 'spec/checker-lints']) {
  for (const name of readdirSync(join(repo, root))) {
    for (const file of ['before.cp', 'after.cp', 'near_miss.cp']) {
      const path = join(repo, root, name, file);
      if (!existsSync(path)) continue;
      const src = readFileSync(path, 'utf8');
      if (!reads(src)) continue;
      const r = cpl.sample(src, { seed: 1, size: 5 });
      if ('input' in r) (inputs[`${root}/${name}/${file}`] = r.input), made++;
      else none++;
    }
  }
}
const out = join(web, 'content/generated/lint-inputs.json');
mkdirSync(dirname(out), { recursive: true });
writeFileSync(out, JSON.stringify(inputs, null, 1) + '\n');
console.log(`lint-inputs: ${made} examples get a random input, ${none} read input their types can't generate`);
