import type { Metadata } from 'next';
import Link from 'next/link';
import { InlineMd } from '@/components/inline-md';
import { decisions } from '@/lib/decisions';

export const metadata: Metadata = {
  title: 'Design decisions',
  description: 'Every CP Lang design decision: the call, the reason, the alternatives rejected, and how to reverse it.',
};

export default function Page() {
  const all = decisions();
  return (
    <main className="container mx-auto max-w-4xl px-4 py-10 [&_code]:rounded [&_code]:bg-muted [&_code]:px-1 [&_code]:text-[0.9em]">
      <h1 className="text-3xl font-semibold tracking-tight">Design decisions</h1>
      <p className="mt-3 text-muted-foreground">
        The {all.length} design calls behind CP Lang, generated from <code>decisions.md</code>: what was decided, why,
        what was rejected and how to reverse it. Docs and Language chapters cite them by number. The rules themselves are
        in the spec; where a decision and the spec disagree, the spec wins.
      </p>
      <ol className="mt-8 space-y-1.5">
        {all.map((d) => (
          <li key={d.slug} className="flex gap-3">
            <Link prefetch={false} className="w-14 shrink-0 text-right font-mono text-sm underline" href={`/decisions/${d.slug}`}>
              {d.number}
            </Link>
            <span className="min-w-0">
              <InlineMd text={d.title} />
            </span>
          </li>
        ))}
      </ol>
    </main>
  );
}
