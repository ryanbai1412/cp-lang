import { existsSync, readFileSync } from 'node:fs';
import { join } from 'node:path';

const repo = join(process.cwd(), '..');

export interface Test {
  input: string;
  expected: string;
}

interface Base {
  id: string;
  title: string;
  prompt: string;
  hint: string;
  solution: string;
}

export type Exercise =
  | (Base & { kind: 'output'; code: string; tests: Test[] })
  | (Base & { kind: 'choice'; code?: string; options: string[]; answer: number; explanation: string })
  | (Base & { kind: 'code'; starter: string; reference: string; tests: Test[] });

export interface Track {
  slug: string;
  title: string;
  description: string;
  duration: string;
  level: string;
  exercises: Exercise[];
}

type Raw = Record<string, unknown> & { fixture?: string; code?: string };

function fixture(e: Raw): string | undefined {
  if (e.code) return e.code as string;
  if (!e.fixture) return undefined;
  const p = join(repo, 'site/examples', `${e.fixture}.cp`);
  return existsSync(p) ? readFileSync(p, 'utf8') : undefined;
}

let cache: Track[] | null = null;

export function tracks(): Track[] {
  if (cache) return cache;
  cache = (['foundations', 'advanced'] as const).flatMap((level) => {
    const raw = JSON.parse(readFileSync(join(repo, 'site/course', `${level}.json`), 'utf8')) as (Omit<
      Track,
      'exercises' | 'level'
    > & { exercises: Raw[] })[];
    return raw.map((t) => ({
      ...t,
      slug: t.slug.replace(/^course-/, ''),
      level,
      exercises: t.exercises.map((e) => ({ ...e, code: fixture(e) }) as unknown as Exercise),
    }));
  });
  return cache;
}

export function findExercise(track: string, id: string) {
  const t = tracks().find((x) => x.slug === track);
  const i = t?.exercises.findIndex((e) => e.id === id) ?? -1;
  if (!t || i < 0) return null;
  return { track: t, exercise: t.exercises[i], index: i, prev: t.exercises[i - 1], next: t.exercises[i + 1] };
}
