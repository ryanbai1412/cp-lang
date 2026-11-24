import { spawnSync } from 'node:child_process';
import { existsSync, mkdtempSync, readFileSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';

// Lint pages come from the compiler's own registry, `cpl lint --list --json`
// (spec/tools/cli.md): every record of spec/lints/ and spec/checker-lints/.
const repoRoot = join(process.cwd(), '..');

export const groups = {
  correctness: {
    title: 'Correctness',
    practice: 'Code that is wrong or almost certainly a bug. Denied by `cpl lint`.',
  },
  perf: { title: 'Performance', practice: 'Code that does more work than it needs to.' },
  idiom: {
    title: 'Idioms',
    practice: 'A hand-written pattern the language or the standard library already says in one step.',
  },
  style: { title: 'Style', practice: 'Spelling that hides what the code means.' },
  pedantic: { title: 'Pedantic', practice: 'Stricter checks, off by default.' },
} as const;

export type Group = keyof typeof groups;

export type Principle = { n: number; title: string };

export type Lint = {
  name: string;
  /** The record's directory under the repo root, e.g. `spec/lints/search_loop`. */
  dir: string;
  code: string;
  level: 'allow' | 'warn' | 'deny';
  group: Group;
  title: string;
  summary: string;
  practice?: string;
  family?: string;
  subfamily?: string;
  principles: Principle[];
  explain: string;
  before: string;
  after: string;
  nearMiss: string;
  /** Which examples can run: they pass `cpl check`, and read no input or have one. */
  runs: { before: boolean; after: boolean; nearMiss: boolean };
  /** A random input from each example's `read()` types (scripts/lint-inputs.mjs). */
  inputs: { before: string; after: string; nearMiss: string };
};

// Each lint record names its family (`family = "S.seq: reductions"`,
// a subfamily of the catalog heading `## S. ...`) and the practices it serves
// (`practice = "#2 #7"`), titled in spec/idioms/principles.md.
const idioms = join(process.cwd(), '../spec/idioms');

export function principles(): Principle[] {
  const text = readFileSync(join(idioms, 'principles.md'), 'utf8');
  return [...text.matchAll(/^\*\*(\d+)\. ([^*]+?)\.?\*\*/gm)].map((m) => ({ n: Number(m[1]), title: m[2] }));
}

const headings = () =>
  readFileSync(join(idioms, 'lints.md'), 'utf8')
    .split('\n')
    .filter((l) => l.startsWith('## '))
    .map((l) => l.slice(3).replace(/\s*\(.*\)\s*$/, '').trim());

function placed(value: string | undefined, families: string[]): { family?: string; subfamily?: string } {
  const sub = value && /^([A-Z])\.\S/.exec(value);
  if (!sub) return { family: value };
  return { family: families.find((h) => h.startsWith(`${sub[1]}. `)) ?? value, subfamily: value };
}

let cache: Lint[] | null = null;

const cpl =
  process.env.CPL ?? ['target/release/cpl', 'target/debug/cpl'].map((p) => join(repoRoot, p)).find(existsSync) ?? 'cpl';
let scratch: string | null = null;

const generated = join(process.cwd(), 'content/generated/lint-inputs.json');
const sampled: Record<string, string> = existsSync(generated) ? JSON.parse(readFileSync(generated, 'utf8')) : {};

function runs(src: string, input: string): boolean {
  if (!src || (/\bread\w*\s*(\(|<|::)/.test(src) && !input)) return false;
  scratch ??= mkdtempSync(join(tmpdir(), 'cpl-lints-'));
  const file = join(scratch, 'main.cp');
  writeFileSync(file, `${src}\n`);
  return spawnSync(cpl, ['check', file], { encoding: 'utf8' }).status === 0;
}

/** One record of `cpl lint --list --json`. */
type LintRecord = {
  code: string;
  name: string;
  dir: string;
  level: Lint['level'];
  group: Group;
  title: string;
  summary: string;
  family: string | null;
  practice: string | null;
  explain: string;
  before: string;
  after: string;
  near_miss: string;
};

export function lints(): Lint[] {
  if (cache) return cache;
  const families = headings();
  const byNumber = new Map(principles().map((p) => [p.n, p]));
  const listed = spawnSync(cpl, ['lint', '--list', '--json'], { encoding: 'utf8', maxBuffer: 64 << 20 });
  if (listed.status !== 0) throw new Error(`lints: \`${cpl} lint --list --json\` failed: ${listed.stderr}`);
  const input = (r: LintRecord, f: string) => sampled[`${r.dir}/${f}`] ?? '';
  cache = (JSON.parse(listed.stdout).lints as LintRecord[]).map((r) => ({
    name: r.name,
    dir: r.dir,
    code: r.code,
    level: r.level,
    group: r.group,
    title: r.title,
    summary: r.summary,
    practice: r.practice ?? undefined,
    ...placed(r.family ?? undefined, families),
    principles: [...(r.practice ?? '').matchAll(/#(\d+)/g)].flatMap((m) => byNumber.get(Number(m[1])) ?? []),
    explain: r.explain,
    before: r.before,
    after: r.after,
    nearMiss: r.near_miss,
    inputs: { before: input(r, 'before.cp'), after: input(r, 'after.cp'), nearMiss: input(r, 'near_miss.cp') },
    runs: {
      before: runs(r.before, input(r, 'before.cp')),
      after: runs(r.after, input(r, 'after.cp')),
      nearMiss: runs(r.near_miss, input(r, 'near_miss.cp')),
    },
  }));
  return cache;
}
