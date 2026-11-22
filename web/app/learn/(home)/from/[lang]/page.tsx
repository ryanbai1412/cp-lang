import type { Metadata } from 'next';
import Link from 'next/link';
import { notFound } from 'next/navigation';
import { Solve } from '@/components/exercise';
import { StatusBadge } from '@/components/status-badge';
import { Button } from '@/components/ui/button';
import { highlight } from '@/lib/highlight';
import { type Lang, languages, quickstart } from '@/lib/quickstart';

type Props = { params: Promise<{ lang: string }> };

export function generateStaticParams() {
  return Object.keys(languages).map((lang) => ({ lang }));
}

export async function generateMetadata({ params }: Props): Promise<Metadata> {
  const { lang } = await params;
  const l = languages[lang as Lang];
  if (!l) notFound();
  return { title: `CP Lang for ${l.name} programmers`, description: `Six problems, side by side with ${l.name}.` };
}

export default async function Page({ params }: Props) {
  const { lang } = await params;
  if (!(lang in languages)) notFound();
  const l = lang as Lang;
  const problems = quickstart();
  const checked = await Promise.all(problems.map((p) => (p.runs ? null : highlight(p.source.trimEnd(), 'cp'))));
  const theirs = await Promise.all(problems.map((p) => highlight(p.yours[l], languages[l].shiki)));
  return (
    <main className="container mx-auto max-w-4xl px-4 py-12">
      <p className="text-sm text-muted-foreground">Quickstart · about 15 minutes</p>
      <h1 className="mt-2 text-3xl font-semibold tracking-tight">CP Lang for {languages[l].name} programmers</h1>
      <p className="mt-2 text-muted-foreground">
        Six short problems. Each shows how you&apos;d write it in {languages[l].name}, then the CP Lang solution. Edit
        it, break it, submit it: your browser judges it. A solution the compiler does not run yet is shown without a judge.
      </p>
      <div className="mt-4 flex flex-wrap gap-2 text-sm">
        {Object.entries(languages).map(([k, v]) => (
          <Button key={k} size="sm" variant={k === l ? 'default' : 'outline'} asChild>
            <Link href={`/learn/from/${k}`}>{v.name}</Link>
          </Button>
        ))}
      </div>
      <ol className="mt-12 space-y-16">
        {problems.map((p, i) => (
          <li key={p.id} id={p.id} className="scroll-mt-20">
            <p className="font-mono text-xs text-muted-foreground">
              {i + 1} / {problems.length} · {p.teaches}
            </p>
            <h2 className="mt-1 text-2xl font-semibold tracking-tight">{p.title}</h2>
            <details className="mt-4 rounded-xl border bg-muted/30">
              <summary className="cursor-pointer px-4 py-2 text-sm text-muted-foreground">
                How you&apos;d write it in {languages[l].name}
              </summary>
              <div
                className="overflow-x-auto border-t text-xs [&_pre]:!bg-transparent [&_pre]:p-4"
                dangerouslySetInnerHTML={{ __html: theirs[i] }}
              />
            </details>
            <div className="mt-4">
              <p className="mb-4 text-base leading-7">{p.statement}</p>
              {p.runs ? (
                <Solve id={`quickstart-${p.id}`} starter={p.source} reference={p.source} tests={p.tests} />
              ) : (
                <div className="overflow-hidden rounded-xl border bg-card shadow-sm">
                  <div className="flex items-center gap-2 border-b px-3 py-2 text-sm text-muted-foreground">
                    <StatusBadge status={p.status} />
                  </div>
                  <div
                    className="overflow-x-auto p-4 text-[13px] leading-relaxed [&_pre]:!bg-transparent"
                    dangerouslySetInnerHTML={{ __html: checked[i]! }}
                  />
                </div>
              )}
            </div>
            <div className="mt-4 rounded-xl border p-4">
              <p className="text-sm font-medium">What&apos;s different from {languages[l].name}</p>
              <ul className="mt-2 list-disc space-y-1 pl-5 text-sm text-muted-foreground">
                {p.notes[l].map((n) => (
                  <li key={n} dangerouslySetInnerHTML={{ __html: n.replace(/`([^`]+)`/g, '<code>$1</code>') }} />
                ))}
              </ul>
            </div>
          </li>
        ))}
      </ol>
      <section className="mt-16 rounded-xl border bg-card p-6">
        <h2 className="text-xl font-semibold">You&apos;ve read six programs</h2>
        <p className="mt-1 text-muted-foreground">Next: problems with verdicts, or look things up.</p>
        <div className="mt-4 flex flex-wrap gap-2">
          <Button asChild>
            <Link href="/learn">Start the course</Link>
          </Button>
          <Button variant="outline" asChild>
            <Link href="/docs/guides/tour">Read the tour</Link>
          </Button>
          <Button variant="outline" asChild>
            <Link href="/setup">Set up your editor</Link>
          </Button>
        </div>
      </section>
    </main>
  );
}
