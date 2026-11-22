import { ExternalLink } from 'lucide-react';
import type { Metadata } from 'next';
import Link from 'next/link';
import { notFound } from 'next/navigation';
import { StatusBadge } from '@/components/status-badge';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { corpus, lineCount } from '@/lib/corpus';
import { highlight } from '@/lib/highlight';

type Props = { params: Promise<{ problem: string }> };

export function generateStaticParams() {
  return corpus().map((e) => ({ problem: e.problem }));
}

export async function generateMetadata({ params }: Props): Promise<Metadata> {
  const { problem } = await params;
  return { title: `${problem} · Real problems`, description: `${problem}: accepted C++ and its CP Lang rewrite.` };
}

export default async function Page({ params }: Props) {
  const { problem } = await params;
  const e = corpus().find((x) => x.problem === problem);
  if (!e) notFound();
  const [cp, cpp] = await Promise.all([
    highlight(e.spec, 'cp'),
    highlight(e.cpp, 'cpp'),
  ]);
  return (
    <main className="container mx-auto max-w-7xl px-4 py-10">
      <Link href="/corpus" className="text-sm text-muted-foreground hover:text-foreground">
        ← Real problems
      </Link>
      <div className="mt-3 flex flex-wrap items-center gap-3">
        <h1 className="font-mono text-3xl font-semibold tracking-tight">{e.problem}</h1>
        <Badge variant="secondary">{e.difficulty}</Badge>
        {e.tags.map((t) => (
          <Badge key={t} variant="outline">{t}</Badge>
        ))}
        <Button size="sm" variant="outline" className="ml-auto" asChild>
          <a href={e.statement} target="_blank" rel="noreferrer">
            Statement on DMOJ <ExternalLink />
          </a>
        </Button>
      </div>
      <div className="mt-6 grid gap-4 lg:grid-cols-2">
        <Pane title="CP Lang" sub={`${lineCount(e.spec)} lines`} badge={<StatusBadge status={e.status} />} html={cp} />
        <Pane title="Accepted C++" sub={`${lineCount(e.cpp)} lines · ${e.cppTime.toFixed(2)} s on the judge`} html={cpp} />
      </div>
    </main>
  );
}

function Pane({ title, sub, html, badge }: { title: string; sub: string; html: string; badge?: React.ReactNode }) {
  return (
    <section className="min-w-0 overflow-hidden rounded-xl border bg-card shadow-sm">
      <header className="flex items-center gap-2 border-b px-4 py-2">
        <h2 className="text-sm font-medium">{title}</h2>
        <span className="text-xs text-muted-foreground">{sub}</span>
        <span className="ml-auto">{badge}</span>
      </header>
      <div
        className="max-h-[70vh] overflow-auto text-xs [&_pre]:!bg-transparent [&_pre]:p-4 dark:[&_.shiki]:!text-[var(--shiki-dark)] dark:[&_.shiki_span]:!text-[var(--shiki-dark)]"
        dangerouslySetInnerHTML={{ __html: html }}
      />
    </section>
  );
}
