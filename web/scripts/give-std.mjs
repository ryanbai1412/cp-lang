// cpl.wasm carries only std's prelude and mod (scripts/build-wasm.sh); a
// script that loads it in Node gives it every other module from std/ up front.
import { readdirSync, readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const std = join(dirname(dirname(dirname(fileURLToPath(import.meta.url)))), 'std');

export function giveStd(instance) {
  for (const f of readdirSync(std)) {
    if (!f.endsWith('.cp')) continue;
    const given = instance.provideStd(f.slice(0, -3), readFileSync(join(std, f), 'utf8'));
    if (given.error) throw new Error(`give-std: ${f}: ${given.error}`);
  }
  return instance;
}
