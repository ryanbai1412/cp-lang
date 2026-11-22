import type { Metadata } from 'next';
import { CorpusGrid } from './grid';
import { corpus, lineCount } from '@/lib/corpus';

export const metadata: Metadata = {
  title: 'Real problems',
  description: 'Real contest problems solved in C++ and rewritten in CP Lang, side by side.',
};

export default function Page() {
  const entries = corpus().map((e) => ({
    problem: e.problem,
    source: e.source,
    tags: e.tags,
    difficulty: e.difficulty,
    notes: e.notes,
    cppLines: lineCount(e.cpp),
    cpLines: lineCount(e.spec),
  }));
  return (
    <main className="container mx-auto max-w-6xl px-4 py-12">
      <h1 className="text-3xl font-semibold tracking-tight">Real problems</h1>
      <p className="mt-2 max-w-3xl text-muted-foreground">
        {entries.length} problems from real contests. Each has an accepted C++ submission and a CP Lang rewrite written
        against the 0.3 spec. The rewrites type-check against the spec but most don&apos;t run on today&apos;s compiler yet;
        statements stay on the judge.
      </p>
      <CorpusGrid entries={entries} />
    </main>
  );
}
