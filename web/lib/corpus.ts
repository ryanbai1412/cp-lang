import { existsSync, readdirSync, readFileSync } from 'node:fs';
import { join } from 'node:path';
import type { Status } from '@/components/status-badge';

const repo = join(process.cwd(), '..');

interface Submission {
  id: string;
  language: string;
  result: string;
  time: number;
  memory_kb: number;
  file: string;
}

interface Meta {
  problem: string;
  source: string;
  submissions: Submission[];
  artifacts: { dream?: string[]; spec?: string[] };
  tags?: string[];
  difficulty?: string;
  spec?: string;
  dream?: string;
  notes?: string;
}

export interface Entry {
  problem: string;
  source: string;
  tags: string[];
  difficulty: string;
  review: string;
  notes: string;
  statement: string;
  spec: string;
  status: Status;
  cpp: string;
  cppFile: string;
  cppTime: number;
}

const read = (p: string) => readFileSync(join(repo, p), 'utf8');
const lines = (s: string) => s.replace(/\n+$/, '').split('\n').length;

let cache: Entry[] | null = null;

export function corpus(): Entry[] {
  if (cache) return cache;
  const dir = join(repo, 'corpus/problems');
  const status = JSON.parse(readFileSync(join(process.cwd(), '.gate/corpus.json'), 'utf8')) as Record<string, Status>;
  cache = readdirSync(dir)
    .filter((p) => existsSync(join(dir, p, 'meta.json')))
    .map((p) => JSON.parse(readFileSync(join(dir, p, 'meta.json'), 'utf8')) as Meta)
    .filter((m) => m.spec && m.spec !== 'none' && m.artifacts.spec?.length)
    .flatMap((m) => {
      const ac = m.submissions.find((s) => s.language.startsWith('CPP') && s.result === 'AC');
      const spec = m.artifacts.spec![0];
      if (!ac || !existsSync(join(repo, ac.file)) || !existsSync(join(repo, spec))) return [];
      return [
        {
          problem: m.problem,
          source: m.source,
          tags: m.tags ?? [],
          difficulty: m.difficulty ?? 'unrated',
          review: m.spec!,
          notes: m.notes ?? '',
          statement: `https://dmoj.ca/problem/${m.problem}`,
          spec: read(spec),
          status: status[m.problem] ?? 'spec',
          cpp: read(ac.file),
          cppFile: ac.file,
          cppTime: ac.time,
        },
      ];
    })
    .sort((a, b) => a.problem.localeCompare(b.problem));
  return cache;
}

export const lineCount = lines;
