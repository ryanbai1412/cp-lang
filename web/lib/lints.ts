import { spawnSync } from 'node:child_process';
import { existsSync, mkdtempSync, readdirSync, readFileSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';

// Lint pages are generated from lint-fmt's records in spec/lints/<name>/, the
// same files crates/cpl-lint/build.rs builds the registry from, and from the
// checker's lints' records in spec/checker-lints/<name>/ (L06xx).
const repoRoot = join(process.cwd(), '..');
const roots = ['spec/lints', 'spec/checker-lints'];

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

function toml(text: string): Record<string, string> {
  const out: Record<string, string> = {};
  for (const line of text.split('\n')) {
    const m = /^(\w+)\s*=\s*("(?:[^"\\]|\\.)*")\s*$/.exec(line);
    if (m) out[m[1]] = JSON.parse(m[2]);
  }
  return out;
}

// Each lint record's lint.toml names its family (`family = "S.seq: reductions"`,
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

export function lints(): Lint[] {
  const families = headings();
  const byNumber = new Map(principles().map((p) => [p.n, p]));
  cache ??= roots
    .flatMap((root) =>
      existsSync(join(repoRoot, root))
        ? readdirSync(join(repoRoot, root), { withFileTypes: true }).map((e) => ({ e, root }))
        : [],
    )
    .filter(({ e, root }) => e.isDirectory() && existsSync(join(repoRoot, root, e.name, 'lint.toml')))
    .map(({ e, root }) => {
      const at = join(repoRoot, root, e.name);
      const file = (f: string) => (existsSync(join(at, f)) ? readFileSync(join(at, f), 'utf8').trimEnd() : '');
      const t = toml(file('lint.toml'));
      return {
        name: e.name,
        dir: `${root}/${e.name}`,
        code: t.code,
        level: t.level as Lint['level'],
        group: t.group as Group,
        title: t.title,
        summary: t.summary,
        practice: t.practice,
        ...placed(t.family, families),
        principles: [...(t.practice ?? '').matchAll(/#(\d+)/g)].flatMap((m) => byNumber.get(Number(m[1])) ?? []),
        explain: file('explain.md'),
        before: file('before.cp'),
        after: file('after.cp'),
        nearMiss: file('near_miss.cp'),
        inputs: {
          before: sampled[`${root}/${e.name}/before.cp`] ?? '',
          after: sampled[`${root}/${e.name}/after.cp`] ?? '',
          nearMiss: sampled[`${root}/${e.name}/near_miss.cp`] ?? '',
        },
        runs: {
          before: runs(file('before.cp'), sampled[`${root}/${e.name}/before.cp`] ?? ''),
          after: runs(file('after.cp'), sampled[`${root}/${e.name}/after.cp`] ?? ''),
          nearMiss: runs(file('near_miss.cp'), sampled[`${root}/${e.name}/near_miss.cp`] ?? ''),
        },
      };
    })
    .sort((a, b) => a.code.localeCompare(b.code));
  return cache;
}
