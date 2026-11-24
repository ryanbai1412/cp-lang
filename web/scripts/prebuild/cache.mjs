// The content-hash cache behind prebuild.mjs. A stage's key hashes its own
// script (with the local modules it imports), every file under its declared
// inputs and the environment variables it reads; when the key and the hash of
// its outputs both match the last successful run, the stage is skipped.
// Paths are repo-relative: a directory (`std`), a file, or `dir/**/*.ext`;
// `!path` excludes.
import { createHash } from 'node:crypto';
import { existsSync, mkdirSync, readdirSync, readFileSync, statSync, writeFileSync } from 'node:fs';
import { dirname, join, relative, sep } from 'node:path';
import { fileURLToPath } from 'node:url';

export const repo = dirname(dirname(dirname(dirname(fileURLToPath(import.meta.url)))));
export const web = join(repo, 'web');
const dir = join(web, 'node_modules/.cache/prebuild');
const SKIP = new Set(['node_modules', '.git', 'target', '.next']);

const memoFile = join(dir, 'files.json');
const memo = existsSync(memoFile) ? JSON.parse(readFileSync(memoFile, 'utf8')) : {};

function fileHash(path) {
  const st = statSync(path);
  const m = memo[path];
  if (m && m[0] === st.size && m[1] === st.mtimeMs) return m[2];
  const h = createHash('sha256').update(readFileSync(path)).digest('hex').slice(0, 24);
  memo[path] = [st.size, st.mtimeMs, h];
  return h;
}

/** One parsed path spec: the directory or file it is rooted at, and an optional suffix. */
export function parse(spec) {
  const neg = spec.startsWith('!');
  const s = neg ? spec.slice(1) : spec;
  const star = s.indexOf('**/*');
  const root = join(repo, star < 0 ? s : s.slice(0, star));
  return { neg, root: root.replace(/[/\\]$/, ''), suffix: star < 0 ? '' : s.slice(star + 4) };
}

const under = (path, root) => path === root || path.startsWith(root + sep);

/** Whether `path` (absolute) falls under the specs. */
export function covers(specs, path) {
  let hit = false;
  for (const p of specs.map(parse)) if (under(path, p.root) && path.endsWith(p.suffix)) hit = !p.neg;
  return hit;
}

function* walk(path) {
  if (!existsSync(path)) return;
  if (!statSync(path).isDirectory()) return yield path;
  for (const e of readdirSync(path, { withFileTypes: true }).sort((a, b) => (a.name < b.name ? -1 : 1))) {
    if (SKIP.has(e.name)) continue;
    const p = join(path, e.name);
    if (e.isDirectory()) yield* walk(p);
    else if (e.isFile()) yield p;
  }
}

/** Every file the specs name, sorted, as [repo-relative path, hash] lines. */
export function hashFiles(specs) {
  const files = new Set();
  for (const p of specs.map(parse).filter((p) => !p.neg))
    for (const f of walk(p.root)) if (f.endsWith(p.suffix) && covers(specs, f)) files.add(f);
  const h = createHash('sha256');
  for (const f of [...files].sort()) h.update(`${relative(repo, f)} ${fileHash(f)}\n`);
  return h.digest('hex');
}

/** A script and the local modules it imports, transitively. */
export function scriptClosure(file, seen = new Set()) {
  if (seen.has(file) || !existsSync(file)) return seen;
  seen.add(file);
  for (const m of readFileSync(file, 'utf8').matchAll(/(?:from|import\(|new URL\()\s*['"](\.\.?\/[^'"]+)['"]/g))
    scriptClosure(join(dirname(file), m[1]), seen);
  // A script it runs by its path, such as `join(web, 'scripts/stress-worker.mjs')`.
  for (const m of readFileSync(file, 'utf8').matchAll(/['"](scripts\/[\w./-]+\.mjs)['"]/g)) scriptClosure(join(web, m[1]), seen);
  return seen;
}

export function stageKey(stage) {
  const h = createHash('sha256');
  h.update(`${stage.name}\n`);
  for (const f of [...scriptClosure(join(web, 'scripts', `${stage.name}.mjs`))].sort()) h.update(`${relative(repo, f)} ${fileHash(f)}\n`);
  h.update(hashFiles(stage.inputs));
  for (const v of stage.env ?? []) h.update(`${v}=${process.env[v] ?? ''}\n`);
  return h.digest('hex');
}

const recordFile = (name) => join(dir, `${name}.json`);
export const recorded = (name) => (existsSync(recordFile(name)) ? JSON.parse(readFileSync(recordFile(name), 'utf8')) : null);
export function record(name, value) {
  mkdirSync(dir, { recursive: true });
  writeFileSync(recordFile(name), JSON.stringify(value));
}
export function saveMemo() {
  mkdirSync(dir, { recursive: true });
  writeFileSync(memoFile, JSON.stringify(memo));
}

// Reads that never make a stage stale: tooling, build caches and the stage's own code.
const ignored = (stage) => [
  ...['web/node_modules', 'node_modules', '.git', 'target', 'web/.next'].map((d) => join(repo, d)),
  ...scriptClosure(join(web, 'scripts', `${stage.name}.mjs`)),
  join(web, 'scripts/prebuild'),
];

/** The repo paths among `reads` (absolute) that the stage's inputs and outputs don't cover. */
export function undeclared(stage, reads) {
  const skip = ignored(stage);
  const declared = [...stage.inputs, ...stage.outputs];
  const bad = new Set();
  for (const path of reads) {
    if (!under(path, repo)) continue;
    if (skip.some((s) => under(path, s))) continue;
    if (covers(declared, path)) continue;
    // A directory listing on the way to a declared input.
    if (declared.some((d) => !d.startsWith('!') && join(repo, d).startsWith(path + sep))) continue;
    bad.add(relative(repo, path) || '.');
  }
  return [...bad].sort();
}
