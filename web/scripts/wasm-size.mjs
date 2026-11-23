// Enforces site/R7 (handbook/guidelines/site-checklist.md).
// Fails when the browser compiler is over its download budget. The cap is on
// the gzipped size of cpl.wasm, which is what a visitor waits for before any
// editor on the site has diagnostics; cpl-build.wasm (adds the C++ back end,
// explain and stress) loads on first use and is reported, not capped.
//
//   node scripts/wasm-size.mjs [dist/wasm/cpl.wasm]
//   node scripts/wasm-size.mjs --why   the crates cpl.wasm's bytes come from
//
// Over the cap, it prints --why's table too, so a lane sees what grew.
import { execFileSync } from 'node:child_process';
import { existsSync, mkdtempSync, readFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { gzipSync } from 'node:zlib';

// A shrink-only ratchet: after each cut, lower the cap to the new size plus
// about 2% in the same commit. It went up once, from 1,720,000, when the VM
// came into cpl.wasm (+153 KB gzipped, the browser runs ~5x faster); the
// commit that deletes the tree-walker lowers it to the new size plus slack.
// Building the interpreter at "s" cut it to 1,804,697; the in-browser TLE
// shrink (55e93f978) brought it to 1,841,845.
const cap = 1_850_000;
const repo = dirname(dirname(dirname(fileURLToPath(import.meta.url))));
const args = process.argv.slice(2);
const why = args.includes('--why');
const file = args.find((a) => !a.startsWith('--')) ?? join(repo, 'dist/wasm/cpl.wasm');
const n = (x) => x.toLocaleString('en-US');

// Code bytes per crate from the module's name section, plus its data.
export function contributors(wasm) {
  let at = 8;
  const leb = () => {
    let v = 0, shift = 0, b;
    do {
      b = wasm[at++];
      v += (b & 0x7f) * 2 ** shift;
      shift += 7;
    } while (b & 0x80);
    return v;
  };
  const str = () => {
    const len = leb();
    const s = wasm.toString('utf8', at, at + len);
    at += len;
    return s;
  };
  let imported = 0;
  const bodies = [];
  const names = new Map();
  let data = 0;
  while (at < wasm.length) {
    const id = wasm[at++];
    const size = leb();
    const end = at + size;
    if (id === 2) {
      for (let i = leb(); i > 0; i--) {
        str();
        str();
        const kind = wasm[at++];
        if (kind === 0) (leb(), imported++);
        else if (kind === 1) (at++, wasm[at++] & 1 ? (leb(), leb()) : leb());
        else if (kind === 2) (wasm[at++] & 1 ? (leb(), leb()) : leb());
        else if (kind === 3) at += 2;
      }
    } else if (id === 10) {
      for (let i = leb(); i > 0; i--) {
        const len = leb();
        bodies.push(len);
        at += len;
      }
    } else if (id === 11) {
      data = size;
    } else if (id === 0 && str() === 'name') {
      while (at < end) {
        const sub = wasm[at++];
        const subEnd = leb() + at;
        if (sub === 1) for (let i = leb(); i > 0; i--) names.set(leb(), str());
        at = subEnd;
      }
    }
    at = end;
  }
  // Rust's v0 mangling (`_R`): the first crate root, `C`, a disambiguator
  // `s<base62>_`, and the crate's name, length-prefixed; for an impl, the
  // crate it is in. Anything else is libc or compiler-rt.
  const crate = (name) => {
    const m = /^_R.*?C(?:s[0-9A-Za-z]*_)?(\d+)/.exec(name);
    return m ? name.substr(m.index + m[0].length, +m[1]) : '(runtime)';
  };
  const by = new Map([['data (std source, strings)', data]]);
  bodies.forEach((len, i) => {
    const c = crate(names.get(i + imported) ?? '');
    by.set(c, (by.get(c) ?? 0) + len);
  });
  return [...by].sort((a, b) => b[1] - a[1]);
}

function explain() {
  const out = mkdtempSync(join(tmpdir(), 'wasm-why-'));
  console.error('wasm-size: building cpl.wasm with function names to see what is in it');
  try {
    execFileSync(join(repo, 'scripts/build-wasm.sh'), ['--names', out], { stdio: 'pipe' });
  } catch (error) {
    console.error(String(error.stderr ?? error));
    return;
  }
  const rows = contributors(readFileSync(join(out, 'cpl.wasm')));
  const total = rows.reduce((s, [, b]) => s + b, 0);
  console.error('wasm-size: raw bytes by crate (gzip shrinks each about 3x):');
  for (const [c, b] of rows.slice(0, 15)) console.error(`  ${n(b).padStart(10)}  ${((100 * b) / total).toFixed(1).padStart(5)}%  ${c}`);
  console.error('  Leave what the editors do not need out of cpl.wasm (crates/cpl-wasm feature `build`, or cfg on wasm32).');
}

if (!existsSync(file)) {
  console.error(`wasm-size: ${file} is missing; run scripts/build-wasm.sh`);
  process.exit(1);
}
const raw = readFileSync(file);
const gz = gzipSync(raw, { level: 9 }).length;
const line = `wasm-size: cpl.wasm is ${n(raw.length)} bytes, ${n(gz)} gzipped (cap ${n(cap)})`;
const full = join(dirname(file), 'cpl-build.wasm');
const fullLine = existsSync(full)
  ? `wasm-size: cpl-build.wasm (loaded on the first build, explain or stress) is ${n(gzipSync(readFileSync(full), { level: 9 }).length)} gzipped`
  : null;
if (gz > cap) {
  console.error(`${line}: over the cap`);
  explain();
  process.exit(1);
}
console.log(line);
if (fullLine) console.log(fullLine);
if (gz * 1.05 < cap) console.log(`wasm-size: over 5% headroom; lower the cap to about ${Math.ceil((gz * 1.02) / 1e4) * 1e4}`);
if (why) explain();
