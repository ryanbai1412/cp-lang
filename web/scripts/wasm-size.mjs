// Fails when the browser compiler is over its download budget. The cap is on
// the gzipped size, which is what a visitor waits for.
import { existsSync, readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { gzipSync } from 'node:zlib';

// A shrink-only ratchet: when a cut leaves more than 10% headroom, lower the
// cap to the new size plus about 5% in the same commit.
const cap = 2_000_000;
const repo = dirname(dirname(dirname(fileURLToPath(import.meta.url))));
const file = process.argv[2] ?? join(repo, 'dist/wasm/cpl.wasm');
if (!existsSync(file)) {
  console.error(`wasm-size: ${file} is missing; run scripts/build-wasm.sh`);
  process.exit(1);
}
const raw = readFileSync(file);
const gz = gzipSync(raw, { level: 9 }).length;
const line = `wasm-size: cpl.wasm is ${raw.length.toLocaleString('en-US')} bytes, ${gz.toLocaleString('en-US')} gzipped (cap ${cap.toLocaleString('en-US')})`;
if (gz > cap) {
  console.error(`${line}: over the cap`);
  process.exit(1);
}
console.log(line);
if (gz * 1.1 < cap) console.log(`wasm-size: over 10% headroom; lower the cap to about ${Math.ceil((gz * 1.05) / 1e4) * 1e4}`);
