import { existsSync, readdirSync, readFileSync } from 'node:fs';
import { join } from 'node:path';

// Lint pages are generated from lint-fmt's records in spec/lints/<name>/, the
// same files crates/cpl-lint/build.rs builds the registry from.
const root = join(process.cwd(), '../spec/lints');

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

export type Lint = {
  name: string;
  code: string;
  level: 'allow' | 'warn' | 'deny';
  group: Group;
  title: string;
  summary: string;
  practice?: string;
  explain: string;
  before: string;
  after: string;
  nearMiss: string;
};

function toml(text: string): Record<string, string> {
  const out: Record<string, string> = {};
  for (const line of text.split('\n')) {
    const m = /^(\w+)\s*=\s*("(?:[^"\\]|\\.)*")\s*$/.exec(line);
    if (m) out[m[1]] = JSON.parse(m[2]);
  }
  return out;
}

let cache: Lint[] | null = null;

export function lints(): Lint[] {
  cache ??= readdirSync(root, { withFileTypes: true })
    .filter((e) => e.isDirectory() && existsSync(join(root, e.name, 'lint.toml')))
    .map((e) => {
      const file = (f: string) =>
        existsSync(join(root, e.name, f)) ? readFileSync(join(root, e.name, f), 'utf8').trimEnd() : '';
      const t = toml(file('lint.toml'));
      return {
        name: e.name,
        code: t.code,
        level: t.level as Lint['level'],
        group: t.group as Group,
        title: t.title,
        summary: t.summary,
        practice: t.practice,
        explain: file('explain.md'),
        before: file('before.cp'),
        after: file('after.cp'),
        nearMiss: file('near_miss.cp'),
      };
    })
    .sort((a, b) => a.code.localeCompare(b.code));
  return cache;
}
